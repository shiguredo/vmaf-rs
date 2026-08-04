# src/sys.rs の #[allow(...)] を #[expect(...)] に変更する

- Priority: High
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/refactor-sys-allow-to-expect
- Polished: 2026-06-06
- Completed: 2026-06-07

## 目的

AGENTS.md の規約「lint 警告を抑制する必要がある時は `#[allow(...)]` ではなく `#[expect(...)]` を使う」に準拠させる。

## 優先度根拠

- AGENTS.md:126-127 の明示的な規約違反。
- `allow` では lint 項目が不要になっても気づけず、コードベースの品質劣化を招く。
- 特に `#[allow(clippy::all)]` は全 Clippy lint を無条件に抑制しており、bindgen 出力外で将来発生する lint も隠蔽する。

## 現状

`src/sys.rs:1-5`:

```rust
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]
```

## 設計方針

`allow` → `expect` に置き換える。ただし `clippy::all` はグループ指定であり、`expect(clippy::all)` で代替できるか確認する。代替できない場合は個別 lint 単位での抑制に変更する。

## 完了条件

- `src/sys.rs` の全 `#![allow(...)]` が `#![expect(...)]` に置き換わっていること
- `cargo clippy --lib --features source-build -- -D warnings` が通過すること
- 既存のテストが全て通過すること

## 解決方法

`src/sys.rs:1-5` を以下に変更する:

```rust
#![expect(non_upper_case_globals)]
#![expect(non_camel_case_types)]
#![expect(non_snake_case)]
#![expect(dead_code)]
#![expect(clippy::all)]
```

`expect(clippy::all)` が動作しない場合は、`clippy::all` に含まれる個別 lint を列挙してそれぞれ `expect` することを検討する。
