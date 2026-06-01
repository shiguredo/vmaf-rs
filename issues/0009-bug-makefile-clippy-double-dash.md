# Makefile の clippy がダブルダッシュで常にエラー終了する

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/fix-makefile-clippy-double-dash

## 目的

`Makefile` の clippy ターゲットが `--` を二重に渡しており、`-D warnings` が clippy-driver / rustc に入力ファイル名として渡されて `error: multiple input filenames provided` となり、clean なコードでも `make clippy` / `make clippy-all` が必ずエラー終了する。lint ゲートとして機能していない。修正する。

## 優先度根拠

ローカルの clippy ターゲットが完全に壊れており、コードが clean でも実行できない（CI / prek 側は正しいのでローカルに限定）。開発時の lint ゲートが使えない。Medium。

## 現状

`Makefile:33`（clippy）と `Makefile:37`（clippy-all）が `-- --` を含む。

```make
cargo clippy --lib --features source-build -- -- -D warnings
```

`cargo clippy` では最初の `--` が cargo 引数とコンパイラ（clippy-driver / rustc）引数の区切りで、それ以降がコンパイラに渡る。`-- --` だと 2 つ目の `--` 以降の `-D warnings` がコンパイラに **入力ファイル名** として渡され、`error: multiple input filenames provided`（`src/lib.rs` と `-D` 等が複数の入力ファイルとして解釈される）で clean なコードでも exit 101 になる。clippy 0.1.95 で実機確認済み。

正しくは単一 `--` の `cargo clippy ... -- -D warnings`。`ci.yml:35` と `prek.toml:32` はこの正しい形を使っている。

## 設計方針

`Makefile:33`（clippy）と `Makefile:37`（clippy-all）の `-- -- -D warnings` を `-- -D warnings` に修正する。

あわせて `Makefile:1` の `.PHONY` に `clippy-all` を追加する（`clippy-all` ターゲットを本 issue で編集するため。現状 `.PHONY` には `clippy` のみで `clippy-all` が抜けている）。`cover` も `.PHONY` 未登録だが本 issue のスコープ外とする。

CHANGES.md への記載は不要とする。`Makefile` は `Cargo.toml:12-18` の `include` に含まれない非配布の開発ツールで、公開 API・配布物に影響しない。

## 関連 issue との切り分け

本 issue は Makefile のダッシュ記法のみを修正する。clippy の対象範囲（`clippy` の `--lib` と `clippy-all` の `--all-targets`）を CI と揃えるかは 0011（CI clippy を `--all-targets` にする）のスコープとし、本 issue では扱わない。

## 完了条件

- `Makefile:33, 37` が単一 `--`（`-- -D warnings`）になっていること
- `clippy-all` が `Makefile:1` の `.PHONY` に登録されていること
- warning のない clean なコードで `make clippy` / `make clippy-all` が成功すること（exit 0）
- 意図的に warning を入れたコードで `make clippy` が失敗すること（deny が効くこと）
- ダッシュ記法が `ci.yml:35` / `prek.toml:32` と一致すること
