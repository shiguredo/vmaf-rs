# I420Frame と DecodedI420 の型重複を解消する

- Priority: Medium
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/refactor-deduplicate-i420-types
- Polished: 2026-06-06

## 目的

`tests/codec_vmaf/types.rs` で定義されている `I420Frame` と `DecodedI420` は完全に同一の 3 フィールド（`y: Vec<u8>, u: Vec<u8>, v: Vec<u8>`）を持ち、不要な clone が発生している。型を一本化する。

## 優先度根拠

- `decoded_to_i420_frame` が 3 つの `Vec<u8>` を毎回 clone するだけの無意味な変換になっている。
- テストコードの保守性が低下している。

## 現状

`types.rs:78-90`:
```rust
pub struct I420Frame { pub y: Vec<u8>, pub u: Vec<u8>, pub v: Vec<u8> }
pub struct DecodedI420 { pub y: Vec<u8>, pub u: Vec<u8>, pub v: Vec<u8> }
```

`pixel.rs:15-21` で `decoded_to_i420_frame` が全フィールドを clone している。

## 設計方針

`type DecodedI420 = I420Frame;` に置き換え、`decoded_to_i420_frame` を除去する。

## 完了条件

- `DecodedI420` が `I420Frame` の型エイリアスになっていること
- `decoded_to_i420_frame` が不要になり除去されていること
- 既存のテストが全て通過すること
