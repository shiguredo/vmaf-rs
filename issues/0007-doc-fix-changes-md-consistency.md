# CHANGES.md の現コードとの矛盾・規約違反を修正する

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/fix-changes-md-consistency

## 目的

`CHANGES.md` の `## develop` セクションが現コードと矛盾し、AGENTS.md の変更履歴規約にも複数違反している。`## develop` を 2026.0.0（初回・未リリース）の正しいリリースノートに整える。

## 優先度根拠

公開物のリリースノートの正確性に関わる。規約の明白な違反であり Medium。修正対象は機械的なものと編集判断を要するものが混在する。

## 判定基準

`## develop` は初回リリース 2026.0.0 の内容そのものになる。AGENTS.md:111-112 の「派生元ブランチとの最終的な差分のみを記載」「開発ブランチ内の中間状態の修正は記載しない」を一貫適用する。すなわち、同じ未リリースサイクル内で追加したコード / 設定に対する後追いの `[FIX]` や、追加後に取り消した項目は中間状態として記載しない。

## 現状の問題

### obsolete / 中間状態のエントリ（削除対象）

- `CHANGES.md:20-21` `[ADD] Picture::from_nv12 を追加する`: 6b71110 で `from_nv12` は削除済み（`src` に痕跡なし、`from_nv12` のヒットは CHANGES.md と issue のみ）。追加後に取り消した項目で最終差分に現れない
- `CHANGES.md:16-17` `[FIX] docs.rs 向け CI で nv12 feature 分離`: 分離した nv12 feature 自体が現 `Cargo.toml` に存在しない（features は `default` / `source-build` のみ）。obsolete かつ中間状態
- `CHANGES.md:18-19` `[FIX] cargo fmt 差分を修正する`: 同サイクルで追加したコードへの fmt 修正で中間状態
- `CHANGES.md:14-15` `[FIX] macOS CI で brew install xxd をやめ標準の xxd を使う`: CI は初回コミット 890b66e で導入されたもので、その CI への後追い修正。上記の判定基準を一貫適用すると、これも同サイクル設定への中間状態の修正であり削除対象。issue 当初は本エントリを残す想定だったが、3 件だけ削除して本エントリを残すのは判定基準が非一貫になるため、`[FIX]` 4 件すべてを削除する

### misc への再分類（AGENTS.md:104）

ライブラリの公開 API に影響しないテスト / ベンチ / CI のエントリは `### misc` に記載する。以下が該当する。

- `CHANGES.md:22` 統合テスト追加、`CHANGES.md:24, 26` その UPDATE
- `CHANGES.md:28, 30` ローカルベンチ追加、`CHANGES.md:32, 34` その UPDATE
- `CHANGES.md:36` CI / prek を `--lib` 対象にする UPDATE

これらは現状 `## develop` 直下にあるが misc が正しい。

### 同サイクル内の UPDATE 畳み込み（AGENTS.md:112）

`CHANGES.md:22` で追加した統合テストへの `24, 26` の UPDATE、`28, 30` で追加したベンチへの `32, 34` の UPDATE は、いずれも同一未リリースサイクル内の中間状態。最終差分としては「完成形の統合テストを追加」「完成形のベンチを追加」という `[ADD]` に畳み込む。

### 事実誤り・文言の不整合

- `CHANGES.md:22` のファイルパス `tests/test_codec_vmaf.rs` は実在しない。実体はディレクトリ `tests/test_codec_vmaf/`（`bench.rs` 等 12 ファイル、`Cargo.toml:32,34` のコメントもスラッシュ付き）。`tests/test_codec_vmaf/` に修正する
- `CHANGES.md:43` `[ADD] shiguredo_libyuv 依存を追加する`: libyuv は 6b71110 で dev-dependency に降格済み（`Cargo.toml:33`、`[dependencies]` は空）。直上 41 行は `dev-dependency` と明記しているのに 43 行だけ不揃い。`shiguredo_libyuv dev-dependency を追加する` に修正する

### 種別順序（AGENTS.md:103）

再分類・削除後、各セクション内のエントリを `CHANGE → ADD → UPDATE → FIX` の順に並べ替える。

### 初回リリースの基幹エントリ（要編集判断）

上記を適用すると `## develop` 直下の機能エントリが実質ゼロになる（ライブラリ本体の追加が itemize されていない）。初回リリースとして、libvmaf バインディング本体の `[ADD]` を `## develop` に追記すべきか、編集判断として検討する。

## 完了条件

- obsolete / 中間状態のエントリ（`[FIX]` 4 件、from_nv12 `[ADD]`）が削除されていること
- テスト / ベンチ / CI のエントリが `### misc` に再分類されていること
- 同サイクル内の UPDATE が `[ADD]` に畳み込まれていること
- `tests/test_codec_vmaf.rs` のパス誤りと misc の libyuv 文言（dev-dependency）が修正されていること
- 各セクションのエントリが `CHANGE → ADD → UPDATE → FIX` 順であること
- 担当者インデント（各エントリ次行 2 文字下げ `- @voluntas`）が維持されていること
- `## develop` が現コードと矛盾しないこと
- 引用元の AGENTS.md 行番号が正しいこと（最終差分=111、中間状態=112）
