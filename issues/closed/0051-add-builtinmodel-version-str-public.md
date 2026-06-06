# BuiltinModel::version_str() を pub に変更する

- Priority: Low
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/add-builtinmodel-version-str-public
- Polished: 2026-06-06
- Completed: 2026-06-07

## 目的

`BuiltinModel::version_str()` の可視性を `fn` (private) から `pub fn` に変更し、利用者が選択したモデルのバージョン文字列を取得できるようにする。

## 優先度根拠

- 外部から有意義な情報であり、公開しても害はない。
- 変更は 1 行の `pub` 追加のみでリスクはゼロ。

## 現状

`src/lib.rs:51`:
```rust
fn version_str(self) -> &'static str {
```

## 解決方法

`src/lib.rs:51` を以下に変更する:
```rust
pub fn version_str(self) -> &'static str {
```
