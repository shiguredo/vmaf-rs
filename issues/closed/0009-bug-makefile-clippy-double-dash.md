# Makefile の clippy ダブルダッシュと `.PHONY` 漏れを修正する

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-06-06
- Completed: 2026-06-06
- Model: Opus 4.8
- Branch: feature/fix-makefile-clippy-double-dash

## 目的

`Makefile` の clippy ターゲットが `--` を二重に渡しており、`-D warnings` が clippy-driver / rustc に入力ファイル名として渡されて `error: multiple input filenames provided` となり、clean なコードでも `make clippy` / `make clippy-all` が必ずエラー終了する。lint ゲートとして機能していない。修正する。

## 優先度根拠

ローカルの clippy ターゲットが完全に壊れており、コードが clean でも実行できない（CI / prek 側は正しいのでローカルに限定）。開発時の lint ゲートが使えない。Medium。

## 現状

`Makefile:33`（clippy）と `Makefile:37`（clippy-all）が `-- --` を含む。

```make
# Makefile:33（clippy） — 誤
cargo clippy --lib --features source-build -- -- -D warnings
# Makefile:37（clippy-all） — 誤、--workspace --all-targets はテストとベンチも対象にする意図
cargo clippy --workspace --all-targets --features source-build -- -- -D warnings
```

`cargo clippy` では最初の `--` が cargo 引数とコンパイラ（clippy-driver / rustc）引数の区切りで、それ以降がコンパイラに渡る。`-- --` だと 2 つ目の `--` 以降の `-D warnings` がコンパイラに **入力ファイル名** として渡され、`error: multiple input filenames provided` で clean なコードでも exit 101 になる。

正しくは単一 `--` の `cargo clippy ... -- -D warnings`。`ci.yml:35` と `prek.toml:32` はこの正しい形を使っている。

## 設計方針

`Makefile:33`（clippy）と `Makefile:37`（clippy-all）の `-- -- -D warnings` を `-- -D warnings` に修正する。

あわせて `Makefile:1` の `.PHONY` に `clippy-all` と `cover` を追加する（`.PHONY` 未登録の全ターゲットを一括修正する）。

`Makefile` は非配布の開発ツールのため、CHANGES.md への記載は不要とする。

## 関連 issue との切り分け

本 issue は Makefile のダッシュ記法のみを修正する。clippy の対象範囲（`clippy` の `--lib` と `clippy-all` の `--all-targets`）を CI と揃えるかは 0011（CI clippy を `--all-targets` にする）のスコープとし、本 issue では扱わない。

## 完了条件

- `Makefile:33, 37` が単一 `--`（`-- -D warnings`）になっていること
- `clippy-all` と `cover` が `Makefile:1` の `.PHONY` に登録されていること
- `make clippy` と `make clippy-all` が現在の develop HEAD で成功すること（exit 0）
- `src/lib.rs` に `let unused = 1;` 等の意図的な warning を入れた状態で `make clippy` が失敗すること（`-D warnings` が効くこと）
- ダッシュ記法が `ci.yml:35` / `prek.toml:32` と一致すること

## 解決方法

- `Makefile:33` と `Makefile:37` の `-- -- -D warnings` を `-- -D warnings` に修正した
- `Makefile:1` の `.PHONY` に `clippy-all` と `cover` を追加した
- `make clippy` と `make clippy-all` が正常終了すること、および意図的な warning で失敗することを確認した
- 変更ファイル: `Makefile`（1 ファイル 3 行）
