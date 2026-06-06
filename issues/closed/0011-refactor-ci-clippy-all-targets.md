# CI / prek の clippy を --all-targets に拡張してテストコードを lint する

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-06-06
- Completed: 2026-06-06
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

### build.rs は既に lint 対象

`cargo clippy --lib` でも build.rs はビルドスクリプトとして常にコンパイルされ clippy 対象になる（`--lib` / `--all-targets` とは独立）。したがって本 issue で広げる対象は `tests/` である。

## 設計方針

CI（`ci.yml:35`）と prek（`prek.toml:32`）の clippy を `--all-targets` に拡張し、テストコードを lint 対象にする。テストの実行は不要でも lint（コンパイル）はかける。

### CI コストとテスト対象範囲の確定

`--all-targets` は `tests/` 全体をコンパイルするため、dev-dependencies（aom / libvpx / libyuv / macOS では video_toolbox）とそのネイティブビルドツール（cmake, perl, nasm 等）が clippy ジョブでも必要になる。`fmt-clippy` ジョブ（`ci.yml:22-35`）は現状 `--lib` で dev-dependencies をビルドしておらず、かつ `test` ジョブの `needs` 先（クリティカルパス）である。

検討すべき選択肢を以下に列挙する:

| 選択肢 | clippy 対象 | fmt-clippy ジョブへの追加 | ビルド時間 |
|--------|------------|--------------------------|-----------|
| A | `--all-targets`（test_score + test_codec_vmaf 両方） | cmake, perl, nasm | 数分増 |
| B | `--tests --exclude test_codec_vmaf`（test_score のみ） | 追加不要（既存の build-essential, meson, nasm で賄える） | ほぼ不変 |
| C | `--lib --tests`（lib + test_score のみ） | 追加不要 | ほぼ不変 |

**選択肢 B を採用する**: `cargo clippy --all-targets --features source-build -- -D warnings`。`--all-targets` はすべてのテストバイナリを対象にするが、`test_codec_vmaf` の dev-dependencies ビルドツール（cmake, perl 等）は `fmt-clippy` ジョブに `apt install cmake` を追加して対応する。0012 との擦り合わせは必要に応じて行う。

### --workspace の扱い

`Cargo.toml` に現状 `[workspace]` が無いため `--workspace` は no-op。0006（PBT 導入）で workspace 化された場合は `--workspace` を追加し、pbt クレートも lint 対象にする。0006 が導入されていない場合は `--workspace` を付けずに `--all-targets` のみとする。

### prek の扱い

prek の clippy フック（`prek.toml:32`）は pre-commit で実行されるため、実行時間が開発体験に直結する。`--all-targets` 化で `test_codec_vmaf` のネイティブビルドが走ると commit ごとに数分の待ち時間が発生する可能性がある。以下の方針とする:

- prek の clippy は `--lib --features source-build -- -D warnings` のまま維持する（テスト全体のビルドは pre-commit に重すぎる）
- CI の clippy のみ `--all-targets` に拡張する

### Makefile の整合

0009 適用後の Makefile では、`clippy`（`--lib`）と `clippy-all`（`--workspace --all-targets`）の 2 ターゲットが存在する。本 issue ではこれらのターゲット構成は維持し、`clippy-all` のフラグを CI の方針に合わせて調整する（`--workspace` の要否は 0006 の状況次第）。

### clippy.toml との関係

`clippy.toml` の `allow-unwrap-in-tests` 等（4-8 行）は「`Cargo.toml` で deny している場合のみ有効」で、現状 `Cargo.toml` に `[lints]` が無いため `unwrap_used` 等は active でなく no-op。よって `--all-targets` でテストを lint 対象にしてもテスト内の `.unwrap()` で clippy が落ちることはない（落ちる原因にならない）。`[lints]` 整備は本 issue のスコープ外。

## 関連 issue との整合

- 0009（Makefile clippy のダッシュ記法修正）が先に入る前提
- 0012 とは「`tests/test_codec_vmaf/` の CI コンパイル」を共有する。0011 = clippy（lint）、0012 = テスト実行、と役割を分ける

## 完了条件

- `ci.yml:35` の clippy が `--all-targets` になっており、`tests/` を lint 対象にしていること
  - `fmt-clippy` ジョブに dev-dependencies ビルドツール（`cmake`）が追加されていること
- `prek.toml:32` の clippy は `--lib` のままであること（pre-commit 実行時間への配慮）
- Makefile の `clippy-all` が CI のフラグ（`--all-targets`、workspace 化後は `--workspace --all-targets`）と整合していること
- CI clippy が `tests/` の lint 違反を検出して fail すること

## 解決方法

- `ci.yml:35` の clippy を `--lib` から `--workspace --all-targets` に変更し、テストコード全体を lint 対象とした
- `ci.yml:29` の `apt-get install` に `cmake` を追加し、dev-dependencies (aom/libvpx) のビルド要件に対応した
- `prek.toml:32` は pre-commit の実行時間を考慮して `--lib` のまま維持した
- Makefile の `clippy-all` は既に `--workspace --all-targets` であり、CI と整合している
- 変更ファイル: `.github/workflows/ci.yml`（1 ファイル 2 行）
