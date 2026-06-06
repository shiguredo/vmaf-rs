# prek.toml に PBT テストフックを追加する

- Priority: Medium
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/add-prek-pbt-hook
- Polished: 2026-06-06
- Completed: 2026-06-07

## 目的

pre-commit フックのテスト範囲を CI と一致させ、PBT の失敗を pre-commit 段階で検出できるようにする。

## 優先度根拠

- CI は `test_score` + `pbt` の両方を実行しているが、pre-commit は `test_score` のみ。
- pre-commit で PBT が実行されないと、CI で初めて PBT の失敗に気づくことになりフィードバックループが遅れる。

## 現状

`prek.toml:39-46`:
```toml
{
  id = "cargo-test",
  name = "cargo test",
  entry = "cargo test --features source-build --test test_score",
  ...
}
```

## 設計方針

既存の `cargo-test` フックに PBT を追加するか、専用の `cargo-pbt` フックを追加する。優先度を適切に設定する。

## 完了条件

- prek.toml に PBT テストが含まれていること
- pre-commit 実行時に PBT が走ること
