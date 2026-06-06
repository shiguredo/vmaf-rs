# CI に codec_vmaf 統合テストを追加する

- Priority: High
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/add-ci-codec-vmaf-test
- Polished: 2026-06-06

## 目的

CI の test ジョブで `tests/codec_vmaf/` 以下のコーデック統合テストが一切実行されていない品質保証上のギャップを解消する。

## 優先度根拠

- `tests/codec_vmaf/` は libvmaf バインディングの実際の使用パターン（コーデック符号化→VMAF 評価）を検証する唯一のテスト群。
- libvmaf のアップデートや Rust 側のリファクタリングで破損しても CI では検知できない。
- 現在 CI では `cargo test --features source-build --test test_score` と `cargo test -p pbt --features source-build` のみ実行。

## 現状

`.github/workflows/ci.yml:70-71`:
```yaml
- run: cargo test --features source-build --test test_score
- run: cargo test -p pbt --features source-build
```

test ジョブの `matrix` には x86_64 と arm が混在している。NASM は x86_64 でのみ必要であり、`codec_vmaf` の AOM テストは NASM を要求する。

## 設計方針

最低限 1 つの軽量 codec_vmaf テストを CI に追加する。候補: `aom_静止画_高品質は符号化サイズが大きく_vmaf_も高い`（フレーム数 1、解像度 192x108 で軽量）。

`source-build` feature が必要であり、ビルド依存（meson, ninja, NASM for x86_64）が既に CI でインストール済みであることを確認する。NASM が必要な x86_64 環境のみに限定する場合は条件付き実行を検討する。

## 完了条件

- CI の test ジョブに codec_vmaf テストが追加されていること
- CI が全環境で通過すること（NASM 未インストールの arm 環境では skip または対応）
- 実行時間が許容範囲内であること

## 解決方法

`.github/workflows/ci.yml` の test ジョブに以下を追加する:
```yaml
- name: Run codec integration test (x86_64 only, requires NASM)
  if: ${{ !endsWith(matrix.os, '-arm') }}
  run: cargo test --features source-build --test codec_vmaf aom_静止画_高品質
```
