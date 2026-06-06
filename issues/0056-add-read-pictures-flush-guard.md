# read_pictures の flush 後呼び出しを型/状態で安全にガードする

- Priority: Medium
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/add-read-pictures-flush-guard
- Polished: 2026-06-06

## 目的

`read_pictures` の flush 後（両方 `None` で呼んだ後）にさらに `read_pictures` が呼ばれた場合の安全性を、API レベルで強制する。

## 優先度根拠

- ドキュメントには「フラッシュ後はこれ以上 `read_pictures` を呼び出せない」と記載されているが、型や状態で強制されていない。
- libvmaf の flush 後再呼び出しの挙動は未定義の可能性があり、Rust 側でガードすべき。

## 現状

`src/lib.rs:217-219`:
```rust
/// `reference` と `distorted` の両方が `None` の場合、内部バッファをフラッシュする。
/// フラッシュ後はこれ以上 `read_pictures` を呼び出せない。
```

しかし `Context` には flush 済みかどうかを追跡する状態がない。

## 設計方針

`Context` に `flushed: bool` フィールドを追加し、flush 後の `read_pictures` 呼び出し時に `Error::InvalidInput` を返す。シンプルな状態管理で十分。

## 完了条件

- `Context` が flush 状態を追跡していること
- flush 後の `read_pictures` 呼び出しがエラーを返すこと
- 既存のテストが全て通過すること
