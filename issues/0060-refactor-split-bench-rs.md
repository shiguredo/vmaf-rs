# bench.rs を責務別に分割する

- Priority: Low
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/refactor-split-bench-rs
- Polished: 2026-06-06

## 目的

`tests/codec_vmaf/bench.rs`（394 行）に 3 つの異なる関心事が混在しているのを責務別に分割する。

## 優先度根拠

- 394 行の単一ファイルはメンテナンス性が低下する閾値に近い。
- ビットレート探索ロジック、レポート整形、目標 VMAF 探索の 3 関心が混在している。

## 現状

`tests/codec_vmaf/bench.rs` の構成:
- ビットレート探索ロジック（`find_bitrate_for_target_vmaf`, `measure_codec_at_bitrate_cached`）: ~90 行
- レポート出力整形（`run_local_bench_report`, `run_y4m_bench_report`）: ~85 行
- 目標 VMAF 探索の統合（`run_matched_vmaf_for_frames`, `run_y4m_matched_vmaf_report`）: ~80 行
- テストエントリポイント（`run_synthetic_bench`, `run_y4m_bench`, `run_y4m_match`）: ~15 行
- `ENCODER_PROFILE_LABEL` 定数もこのファイルに定義

## 設計方針

以下のように分割する:
- `bench/search.rs` — ビットレート探索ロジック
- `bench/report.rs` — レポート出力整形
- `bench/match.rs` — 目標 VMAF 探索の統合
- `bench.rs` — テストエントリポイント

または `codec_vmaf/` の既存モジュール（`scenario.rs` 等）に一部統合する。

## 完了条件

- `bench.rs` が適切に分割されていること
- 既存のテストが全て通過すること
