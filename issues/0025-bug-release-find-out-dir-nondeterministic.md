# release.yml の find_out_dir が複数マッチ時に不定になる

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-06-06
- Model: Opus 4.8
- Branch: feature/fix-release-find-out-dir-nondeterministic

## 目的

`release.yml` の OUT_DIR 探索が `head -1` で、複数の build ディレクトリがマッチした場合に拾うものが不定になる。さらに `SRC_DIR` が build.rs の内部レイアウトに決め打ちで結合している。成果物の特定を厳密にし、想定外の状態では fail させる。

## 優先度根拠

クリーンな CI ランナーでは通常 1 個で問題は出にくいが、非決定的な挙動は潜在的なリリース事故の元。Low。

## 現状（行番号は実ファイルと一致を確認済み）

- `release.yml:91`: `OUT_DIR=$(find target/release/build/shiguredo_vmaf-*/out -maxdepth 0 -type d 2>/dev/null | head -1)`。cargo は features / profile / rustflags が変わると別 `shiguredo_vmaf-<hash>` ディレクトリを作り旧 hash を自動削除しないため、target を使い回す環境では複数生成され得る。`find` の出力順は非ソートのため `head -1` が拾うものは不定（build-prebuilt はクリーンランナーで 1 回ビルドのため現実には通常 1 個）
- `release.yml:98`: `SRC_DIR="$OUT_DIR/build/vmaf/libvmaf/build/src"` が build.rs の `build_from_source`（`build.rs:298-346`、`output_lib_dir = out_dir/build/vmaf/libvmaf/build/src/`、`LIB_NAME` = `vmaf` / `LIBVMAF_DIR` = `libvmaf`）の内部構造を文字列でコピーしている。build.rs の構造が変わると release CI だけ無言で壊れる

## 設計方針

OUT_DIR 探索（`release.yml:91`）と SRC_DIR からの成果物コピー（`release.yml:98-101`）は同じブロックの同根問題のため、まとめて本 issue で扱う。

### OUT_DIR の特定

`shiguredo_vmaf-*/out` のマッチ件数を数え、**厳密に 1 個でなければ即 fail** する（複数なら fail、0 なら fail）。リリースは配布物を作るため、想定外のビルド状態（複数 out）で mtime 最新を黙って選ぶのは「どのビルドのライブラリを配ったか不定」となりリリース事故になる。「最新を選ぶ」方式は採らない。

より堅牢な代替として、`cargo build --release --features source-build --message-format=json`（`release.yml:85` のビルドステップ）の `build-script-executed` メッセージから `shiguredo_vmaf` の `out_dir` を `jq` で直接取得する方式がある（glob もハッシュ推測も不要）。`jq` 依存とビルドステップ統合のトレードオフがあるが、glob + 件数検証で十分なら前者を採る。どちらを採るか実装時に確定し、glob 方式なら件数検証を必須とする。

### SRC_DIR の検証

`SRC_DIR`（`release.yml:98`）から `libvmaf.a` をコピーする前に、`SRC_DIR/libvmaf.a` の存在を確認し、無ければ fail する。build.rs の構造変更で SRC_DIR がずれた場合に無言で壊れず検知できるようにする。

## CHANGES.md

配布物の特定方法の堅牢化で公開 API・配布物の内容自体は変わらないため、CHANGES.md への記載は不要とする（0007 の方針、0011 / 0013 / 0022 / 0023 と同様）。

## 関連 issue との整合

本 issue が触るのは build-prebuilt ジョブの `release.yml:87-102`（find_out_dir + アーカイブ作成）のみで、0004（トップレベル permissions + github-release ジョブ）・0013（slack_notify）・0014（publish ジョブ）・0023（トップレベル concurrency + publish/github-release timeout）のいずれとも編集ブロックが重ならず直接衝突しない。0012（prebuilt 経路の smoke test）は同じ build-prebuilt 成果物の正しさを扱うため、本 issue の堅牢化は 0012 の前提を支える。

## 完了条件

- OUT_DIR のマッチ件数が 1 でない場合に build-prebuilt ジョブが fail する記述があること（静的確認）
- `SRC_DIR/libvmaf.a` の存在確認が cp の前に入っていること（静的確認）
