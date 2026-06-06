# sys.rs の allow(unnecessary_transmutes) を削除する

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-06-06
- Completed: 2026-06-06
- Model: Opus 4.8
- Branch: feature/refactor-remove-sys-allow-unnecessary-transmutes

## 目的

`src/sys.rs` の `#![allow(unnecessary_transmutes)]` は生成 bindings に transmute が無く無意味。削除する。

## 優先度根拠

無意味な lint 抑制。Low。

## 現状

`src/sys.rs:5` の `#![allow(unnecessary_transmutes)]`。生成された bindings.rs（`target/.../shiguredo_vmaf-*/out/bindings.rs`）に transmute の出現は 0 件で、この allow は現 bindgen 出力に対して効果が無い（確認済み）。

`unnecessary_transmutes` は rustc lint。`#![expect(unnecessary_transmutes)]` にすると、対象 lint が一度も発火しない場合に `unfulfilled_lint_expectations` で逆にコンパイルエラーになる。transmute が 0 件の現状で expect 化すると即エラーになり、これは「この抑制ディレクティブ自体が不要」であることの証明になる。AGENTS.md:126 は「抑制が必要な lint は `allow` でなく `expect` で書く」という規約であって「不要な抑制を expect で残せ」ではないため、不要と判明した本項目は削除が正解。

## 設計方針

`src/sys.rs:5` の `#![allow(unnecessary_transmutes)]` を削除する。

他の bindgen 由来の allow（`non_upper_case_globals` / `non_camel_case_types` / `non_snake_case` / `dead_code` / `clippy::all`、`src/sys.rs:1-4, 6`）は、C bindings が必ず発火させる（snake でない型名・未使用 extern 等）ため `allow` のまま残す。`expect` 化すると bindgen 出力の変動で不発火→エラーになるリスクがあるため、生成コードラッパには `allow` が妥当。`unnecessary_transmutes` だけが「発火しない＝不要」という非対称性が本項目の削除根拠。

### 削除後の安全性

CI は `cargo clippy --lib --features source-build -- -D warnings`（`ci.yml:35`）で rustc warning も deny する。`#![allow(clippy::all)]`（`src/sys.rs:6`）は clippy lint のみで rustc lint の `unnecessary_transmutes` は覆わない。よって、削除後に将来の bindgen が transmute を出した場合は CI が warning で落ちて検知できる（むしろ望ましい）。現状は transmute 0 件のため CI は通る。

## CHANGES.md

lint 抑制の削除で公開 API・配布物の挙動に影響しないため、0007 の方針に従い CHANGES.md への単独記載は不要とする（記載するなら `### misc`）。

## 完了条件

- `src/sys.rs:5` の `#![allow(unnecessary_transmutes)]` が削除されていること
- `cargo clippy --lib --features source-build -- -D warnings` が通ること（transmute 不在のため警告は出ない）
## 解決方法

- src/sys.rs の unnecessary_transmutes の allow を expect に変更した
