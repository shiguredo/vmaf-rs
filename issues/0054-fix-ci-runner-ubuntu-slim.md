# CI の ubuntu-slim ランナーを確認し標準ランナーに統一する

- Priority: High
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/fix-ci-runner-ubuntu-slim
- Polished: 2026-06-06

## 目的

`ci.yml` と `release.yml` の `slack_notify` ジョブで使用されている `runs-on: ubuntu-slim` が GitHub-hosted ランナーの標準ラベルではないため、組織のカスタムランナーの存在を確認し、なければ標準ランナーに修正する。

## 優先度根拠

- `ubuntu-slim` が GitHub-hosted 標準ランナーでなければジョブが永久待機状態になり、Slack 通知が機能しない。
- Slack 通知は CI の最終段階であり、失敗が他ジョブに波及しないため見逃されやすい。

## 現状

`.github/workflows/ci.yml:92`:
```yaml
runs-on: ubuntu-slim
```

`.github/workflows/release.yml:170`:
```yaml
runs-on: ubuntu-slim
```

GitHub-hosted ランナーの標準ラベルは `ubuntu-24.04`、`ubuntu-22.04`、`ubuntu-latest` 等。

## 設計方針

1. 組織のセルフホストランナー `ubuntu-slim` が実際に存在するか確認する
2. 存在しない場合は `ubuntu-24.04` に統一する
3. 存在する場合は、その旨をコメントで明記する

## 完了条件

- `ubuntu-slim` ランナーの存在が確認されていること
- 存在しない場合は標準ランナーに修正されていること
- CI の Slack 通知ジョブが正常に動作すること
