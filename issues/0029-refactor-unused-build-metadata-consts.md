# 未使用の BUILD_REPOSITORY / BUILD_VERSION を削除する

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-06-06
- Model: Opus 4.8
- Branch: feature/refactor-unused-build-metadata-consts

## 目的

公開定数 `BUILD_REPOSITORY` / `BUILD_VERSION` がクレート内・テスト内のどこからも参照されず、用途も示されていない。要否を判断し、本 issue では削除を推奨する。

## 優先度根拠

未使用の公開 API。外部利用の可能性はあるが用途不明。Low。

## 現状

`src/lib.rs:25-29` の `pub const BUILD_REPOSITORY` / `BUILD_VERSION`（`sys::BUILD_METADATA_*` を公開）は定義箇所のみで参照が無い（src / tests / README いずれにも登場しない）。`build.rs:25-37` は両定数を全ビルド経路で無条件に metadata.rs に書き出すコストを払っており、`src/sys.rs:8` が `include!(metadata.rs)` する。

### version() との冗長性

- `BUILD_VERSION`（`lib.rs:29`）は `Cargo.toml:23` の `version = "v3.1.0"` を build.rs で焼き込んだ静的な宣言値。一方 `version()`（`lib.rs:17-22`）は `vmaf_version()` を呼ぶランタイム値。ソースは異なるが、利用者にとって「どの libvmaf か」を知る情報としてはほぼ重複する
- `BUILD_REPOSITORY`（`lib.rs:26` = `https://github.com/Netflix/vmaf`）は `version()` に無い固有情報だが、固定値であり `Cargo.toml:22-23` で誰でも確認できる

未使用・未文書化の provenance を、ランタイム `version()` という代替手段がある中で公開し続ける根拠は弱い。プロジェクトは最小公開を原則とする。

## 設計方針

削除を推奨する。未使用・未公開（version 2026.0.0 は crates.io 未公開）で機能影響がなく、`version()` が provenance を実質的に満たし、build.rs が全ビルドで metadata 書き出しコストを払っているため。

### 削除する場合の範囲

- `src/lib.rs:25-29`: `BUILD_REPOSITORY` / `BUILD_VERSION` の doc コメントと定数定義
- `build.rs:22`: `let output_metadata_path = ...`
- `build.rs:25-37`: metadata 書き出し（コメント + `let (git_url, version) = get_git_url_and_version();` + `fs::write`）。**`get_git_url_and_version()` 関数（`build.rs:376-398`）は `build.rs:338` の clone でも使うため残す**
- `src/sys.rs:8`: `include!(.../metadata.rs)`

### 維持する場合

`src/lib.rs:25, 28` の doc に、ランタイム `version()` との違い（ビルド時に参照した libvmaf のソース URL とバージョンタグ＝ビルド時 provenance）を明記する。

## 関連 issue との整合

削除は `src/sys.rs` と `build.rs` を編集するため、0003（docs.rs ダミー bindings 追従、High、`src/sys.rs` / `build.rs` を編集）と同一ファイルで競合する。metadata.rs は全ビルド経路（source-build / prebuilt / docs.rs）で無条件生成されており、`sys.rs:8` の削除は docs.rs 経路にも影響する。0003 が先に入る前提でリベースする。

## CHANGES.md

削除する場合: 公開 API の削除だが未公開のため後方互換破壊に当たらず、`### misc` に「未使用の BUILD_REPOSITORY / BUILD_VERSION を削除する」を `- @voluntas` 付きで記載する（`[CHANGE]` 不要）。維持 + doc の場合: doc 変更のみで CHANGES.md 対象外。

## 完了条件

- 削除した場合: `src/lib.rs:25-29`・`build.rs:22, 25-37`・`src/sys.rs:8` が削除され、`get_git_url_and_version()` は残り、`cargo build` と `DOCS_RS=1 cargo build --no-default-features` が通ること。`### misc` に削除エントリが追記されていること
- 維持した場合: `BUILD_REPOSITORY` / `BUILD_VERSION` の doc に `version()` との差分（ビルド時 provenance）が明記されていること
