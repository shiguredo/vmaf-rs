# CI で統合テストのコンパイルと prebuilt 経路を検証する

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-06-06
- Completed: 2026-06-06
- Model: Opus 4.8
- Branch: feature/add-ci-cover-integration-and-prebuilt

## 目的

CI が `--test test_score` のみを実行し、統合テスト `tests/test_codec_vmaf/` をコンパイルすらせず、利用者が最も使う prebuilt ダウンロード経路も一度もテストしていない。これらのリグレッションをリリース前に検知できるようにする。

本 issue は達成タイミングの異なる 2 つの作業を含む（後述）。

## 優先度根拠

「CI が緑でも利用者のビルドが壊れている」状態を生む構造的な穴。Medium。

## 現状（行番号は実ファイルと一致を確認済み）

- `ci.yml:67`: 全マトリクスで `cargo test --features source-build --test test_score` のみ。`tests/test_codec_vmaf/`（dev-dependencies の aom / libvpx / libyuv / video_toolbox を使用）はコンパイルされず、API 破壊や依存破壊を検知できない
- CI の全ジョブが `--features source-build` か `--no-default-features` で、デフォルトの prebuilt download 経路（`build.rs:157-179`）を実行するジョブが存在しない
- `release.yml:65` の macOS prebuilt は macos-15 のみだが `ci.yml:48` の test は macos-26 も対象。アーカイブ名 `vmaf-macos_arm64.tar.gz`（`build.rs:161`）は OS バージョン非依存で 404 にはならない

## 設計方針

### 作業 A: 統合テストのコンパイル検証（初回リリース前から実施可能）

`tests/test_codec_vmaf/` を CI でコンパイルし、API 破壊・依存破壊を検知する。

- コマンドは `cargo test --no-run --test test_codec_vmaf --features source-build` を用い、コンパイルのみで実行はしない。`main.rs` の `local_*` テスト群は `local_videos/` 配下の実 Y4M クリップ（リポジトリに同梱されない）を要求し、実行すると不在で失敗するため、`--no-run` で実行を回避する
- `tests/test_codec_vmaf/` は dev-dependencies の aom / libvpx を **ソースからネイティブビルド** するため、`fmt-clippy` / `test` ジョブが現状入れている `meson ninja-build nasm xxd`（`ci.yml:57`）に加え、aom / libvpx のビルドに必要なツール（cmake / perl 等）が要る可能性がある。各 dev-dependency crate のビルド要件を確認して CI ジョブに追加する
- 0011 と役割を分ける。0011 が clippy `--all-targets` で `tests/test_codec_vmaf/` をコンパイル（lint）する方針なら、同じ重いネイティブビルドが CI で二重に走らないよう、コンパイル検証は 0011 の lint ジョブに集約し、本 issue はその前提で実行系（作業 B）に専念するか、逆に本 issue のコンパイルジョブを 0011 が再利用するかを擦り合わせる

### 作業 B: prebuilt 経路の smoke test（初回リリース後にのみ有効）

prebuilt download は `releases/download/{CARGO_PKG_VERSION}/`（`build.rs:157-160`）から取得する。version 2026.0.0 はまだリリースされておらず（git tag なし）、対応するアセットが存在しないため、**初回リリースが完了するまで prebuilt の smoke test は物理的に実行できない**（404 で `build.rs:179` が panic する）。

実現手段は次のいずれか。

- リリース済みタグを checkout してデフォルト feature でビルドするワークフローを別途持つ。タグ上では `CARGO_PKG_VERSION` が公開済みアセットと一致するため、build.rs を変更せず prebuilt 経路を検証できる。`workflow_dispatch` か scheduled で起動する
- もしくは `build.rs` にダウンロード元 version を上書きする env（`VMAF_TARGET`（`build.rs:352`）の version 版）を新設し、任意の released version を指す。こちらは build.rs 改修を伴う

初回リリース前にこのジョブを develop の通常 CI に入れると必ず失敗するため、初回リリース後に有効化する（それまでは追加しない、または `workflow_dispatch` 限定）。

### macOS バージョン整合（結論）

macos-15 でビルドした静的ライブラリ `libvmaf.a` を macos-26 で消費しても、同一 arm64 アーキテクチャの静的リンクであり問題なく通る想定（実害は deployment-target 警告程度）。smoke test は prebuilt をビルドした macos-15 を対象にすれば十分で、macos-26 向けに別 prebuilt を用意する必要はない。

## 関連 issue との整合

- 0011（clippy `--all-targets`）が `tests/test_codec_vmaf/` を lint のためにコンパイルする。作業 A と重複するため、重い dev-dependencies ビルドを CI で二重実行しない分担（0011 = lint コンパイル、0012 = 実行系 / prebuilt smoke）を擦り合わせる
- 0010（closed、対応見送り。prebuilt 経路のサプライチェーン強化）と同じ prebuilt 経路を扱うが、本 issue は「ダウンロード→展開→ビルド成功」という利用者視点の smoke test であり、0010 の検証ロジック改修とは独立に進められる。0010 は対応見送り（closed）だが本 issue はその判断に関係なく進められる
- 0006（PBT / fuzzing 導入）の CI 実行分担とも整合させる

## CHANGES.md

CI のテストカバレッジ追加で公開 API・配布物に影響しないため、CHANGES.md への記載は不要とする。

## 完了条件

作業 A（初回リリース前から達成可能）:

- `tests/test_codec_vmaf/` が CI で `cargo test --no-run --test test_codec_vmaf --features source-build` 相当によりコンパイルされ、API 破壊・依存破壊が検知されること
- 必要な dev-dependencies ビルドツールが該当 CI ジョブに追加されていること
- 0011 とコンパイル分担が整理され、重い dev-dependencies ビルドが CI で不必要に二重実行されないこと

作業 B（初回リリース後に有効化）:

- リリース済みバージョンに対する prebuilt デフォルトビルドの smoke test ワークフローが存在すること
- 初回リリース前は develop の通常 CI を失敗させない形（別ワークフロー / `workflow_dispatch` 等）で組まれていること

## 解決方法

- 作業 A（統合テストのコンパイル検証）: 0011 (`--workspace --all-targets`) により fmt-clippy ジョブで達成済みのため、本 issue では追加作業なし
- 作業 B（prebuilt 経路の smoke test）: `.github/workflows/prebuilt-smoke.yml` を新規作成し、`workflow_dispatch` で手動起動可能な prebuilt 経路検証ワークフローを追加した
  - 指定されたリリースタグを checkout してデフォルト feature でビルドし、prebuilt download が機能することを検証する
  - source-build の確認ステップも含む
- 変更ファイル: `.github/workflows/prebuilt-smoke.yml`（新規 1 ファイル）
