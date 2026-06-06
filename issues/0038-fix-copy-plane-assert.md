# copy_plane の debug_assert! を assert! に変更する

- Priority: High
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/fix-copy-plane-assert
- Polished: 2026-06-06

## 目的

`copy_plane` 内のバッファサイズ検証と null ポインタチェックを `debug_assert!` から `assert!` に昇格させ、リリースビルドでも安全性を確保する。

## 優先度根拠

- `debug_assert!` はリリースビルドで無効化される。`copy_plane` が不正なサイズで呼ばれた場合、`src[row * width..(row + 1) * width]`（`src/lib.rs:432`）でバッファオーバーフローが発生し UB になる。
- AGENTS.md:118「性能より堅牢性を優先すること」に反する。

## 現状

`src/lib.rs:420-429`:

```rust
let expected_len = width * height;
debug_assert!(
    src.len() >= expected_len,
    "plane {plane} buffer too small: expected at least {expected_len}, got {}",
    src.len()
);

let stride = pic.stride[plane] as usize;
let dst_ptr = pic.data[plane] as *mut u8;
debug_assert!(!dst_ptr.is_null());
```

## 設計方針

`debug_assert!` → `assert!` に変更するのみ。

## 完了条件

- `src/lib.rs` の `copy_plane` 内の 2 つの `debug_assert!` が `assert!` に変更されていること
- 既存のテストが全て通過すること

## 解決方法

`src/lib.rs:421-429` の `debug_assert!` を `assert!` に変更する。
