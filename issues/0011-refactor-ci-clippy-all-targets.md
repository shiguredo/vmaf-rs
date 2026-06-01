# CI / prek の clippy を --all-targets に拡張してテストコードを lint する

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/refactor-ci-clippy-all-targets

## 目的

CI と prek の clippy が `--lib` のみで、`tests/` 配下のテストコードに lint が当たっていない。検査範囲をテストコードまで広げる。CI / 開発ゲートの設定変更で公開 API・配布物には影響しないため refactor とする。

## 優先度根拠

テストコード（`tests/test_score.rs`、`tests/test_codec_vmaf/`）が lint されておらず品質ゲートの穴。Medium。

## 現状

- `ci.yml:35`: `cargo clippy --lib --features source-build -- -D warnings`
- `prek.toml:32`: 同上
- `--lib` 指定により `tests/test_score.rs`、`tests/test_codec_vmaf/`（dev-dependencies の aom / libvpx / libyuv / video_toolbox をリンクするマルチファイルのテストバイナリ）が clippy 対象外
- `Makefile:37` に `clippy-all`（`--workspace --all-targets`）が既にあるが CI / prek では使われていない

### build.rs は既に lint されている（当初の前提を訂正）

当初 issue は「build.rs が lint されていない」を主目的にしていたが、これは誤り。clippy 0.1.95 で実機確認した結果、`cargo clippy --lib` でも build.rs はビルドスクリプトとして常にコンパイルされ clippy 対象になる（ターゲット選択フラグ `--lib` / `--all-targets` とは独立）。`--all-targets` が新たに追加するのは tests / examples / benches であって build.rs ではない。したがって本 issue で広げる対象は build.rs ではなく `tests/`。

なお `cargo clippy -- -D warnings` の `-D warnings` がビルドスクリプト（build.rs）の警告まで deny するかは、ビルドスクリプトへのフラグ伝播の挙動に依存するため、build.rs の警告を CI で fail させたい場合は別途 `Cargo.toml` の `[lints]` 設定が要るか実装時に確認する（本 issue のスコープは tests/ の lint だが、確認事項として記す）。

## 設計方針

CI（`ci.yml:35`）と prek（`prek.toml:32`）の clippy を `--all-targets` に拡張し、テストコードを lint 対象にする。テストの実行は不要でも lint（コンパイル）はかける。

### CI コストの評価（要確定）

`--all-targets` は `tests/test_codec_vmaf/` をコンパイルするため、dev-dependencies（aom / libvpx / libyuv / macOS では video_toolbox）とそのネイティブビルドツールが clippy ジョブでも必要になる。`fmt-clippy` ジョブ（`ci.yml:22-35`）は現状 `--lib` で dev-dependencies をビルドしておらず、かつ `test` ジョブの `needs` 先（クリティカルパス）。`--all-targets` 化で数分規模のコスト増があり得る。`tests/test_codec_vmaf/` まで lint 対象に含めるか、軽量な `test_score.rs` のみに絞るかを 0012（統合テストの CI カバレッジ）と整合させて確定する。重い統合テストを含めるなら、必要な dev-dependencies ビルドツールを `fmt-clippy` ジョブに追加する。

### --workspace の扱い

`Cargo.toml` に現状 `[workspace]` が無いため `--workspace` は no-op。0006（PBT 導入で workspace 化 + pbt member 追加）が先に入った場合のみ `--workspace` が意味を持ち pbt も lint 対象になる。0006 の workspace 化を前提にするか否かを明記して `--workspace` の要否を決める。

### clippy.toml との関係

`clippy.toml` の `allow-unwrap-in-tests` 等（4-8 行）は「`Cargo.toml` で deny している場合のみ有効」で、現状 `Cargo.toml` に `[lints]` が無いため `unwrap_used` 等は active でなく no-op。よって `--all-targets` でテストを lint 対象にしてもテスト内の `.unwrap()` で clippy が落ちることはない（落ちる原因にならない）。`[lints]` 整備は本 issue のスコープ外。

## 関連 issue との整合

- 0009（Makefile clippy のダッシュ記法修正）が先に入る前提。0009 が `Makefile` の clippy ターゲットの `-- --` を `--` に直し `.PHONY` に `clippy-all` を追加する。本 issue は 0009 完了後に着手し、Makefile の clippy（`--lib`）/ clippy-all（`--all-targets`）の 2 ターゲットを CI 方針に合わせて最終形を決める（行番号は 0009 適用後に確認）
- 0012 とは「`tests/test_codec_vmaf/` の CI コンパイル」を共有する。0011 = clippy（lint）、0012 = テスト実行、と役割を分け、重い dev-dependencies のビルドを CI に持ち込む方針を擦り合わせる
- CHANGES.md:36 に `[UPDATE] CI / prek の clippy と test をライブラリ (--lib, --test test_score) のみ対象にする` という本 issue が打ち消す向きの既存エントリがある。本 issue で範囲を広げると矛盾するため、このエントリの更新を 0007（CHANGES.md 整合）と調整する

## CHANGES.md

CI / prek の lint 設定変更で公開 API・配布物に影響しないため、CHANGES.md への新規記載は不要とする。ただし上記のとおり既存の CHANGES.md:36 エントリとの整合は 0007 と調整する。

## 完了条件

- `ci.yml:35` と `prek.toml:32` の clippy が `tests/` を lint 対象にしていること（`--all-targets` 等）
- `tests/test_codec_vmaf/` を lint 対象に含めるか否かが確定し、含めるなら必要な dev-dependencies ビルドツールが `fmt-clippy` ジョブに追加されていること
- CI / prek / Makefile の clippy 引数の最終形（各ファイルの具体的なコマンド、Makefile の clippy / clippy-all 2 ターゲットを統合するか残置するか）が列挙され整合していること
- 意図的に lint 違反を入れたテストコードで CI clippy が fail すること（ゲートが機能することの確認）
