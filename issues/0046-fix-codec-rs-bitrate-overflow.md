# codec.rs のビットレート乗算オーバーフローと UV サイズ計算不一致を修正する

- Priority: Medium
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/fix-codec-rs-bitrate-overflow
- Polished: 2026-06-06

## 目的

1. `target_bitrate` 計算で `* 1000` の乗算がオーバーフローする可能性を修正する
2. AOM と VPX で UV サイズ計算に `div_ceil` と `/2` の不一致があるのを統一する
3. VP8 `cpu_used` の符号付き→u32→usize 変換の意図を明確にする

## 優先度根拠

- ビットレートのオーバーフローは debug ビルドでパニック、release で wrapping して意図しない低ビットレートになる
- UV サイズ計算の不一致は将来の変更でバグを誘発しやすい

## 現状

| 問題 | 行 | 現状 |
|---|---|---|
| 乗算オーバーフロー | `codec.rs:47,64` | `usize::try_from(bitrate_kbps).expect(...) * 1000` |
| UV 計算不一致 | `codec.rs:85-86` vs `:120-121` | `div_ceil(2)` vs `/2` |
| VP8 cpu_used | `codec.rs:68` | `(-6i32 as u32) as usize` |

## 設計方針

1. `usize::checked_mul(1000).expect(...)` を使用する
2. 両方で `div_ceil` に統一する
3. VP8 の `cpu_used` は libvpx の FFI 層の型定義を確認し、適切な変換方法を選択する

## 完了条件

- ビットレート計算でオーバーフロー時に明確なエラーメッセージでパニックすること
- AOM/VPX 両方で `div_ceil` が使用されていること
- 既存の codec_vmaf テストが通過すること
