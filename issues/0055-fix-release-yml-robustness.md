# release.yml のタグトリガーを制限し OUT_DIR 検出の非決定性を修正する

- Priority: Medium
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/fix-release-yml-robustness
- Polished: 2026-06-06

## 目的

1. タグトリガー `tags: "*"` を制限し、無効なタグプッシュで不要なジョブが実行されるのを防ぐ
2. OUT_DIR 検出の `find ... | head -1` が非決定的である問題を修正する
3. `slack_notify` ジョブの `timeout-minutes: 20` を `5` に統一する

## 優先度根拠

- タグトリガーが `*` で広範すぎる。誤ってプッシュしたタグが crates.io publish をトリガーするリスク。
- `find | head -1` はビルドキャッシュ残存時に誤った OUT_DIR を参照する可能性がある。
- Slack 通知に 20 分のタイムアウトは過剰で、`ci.yml` の同ジョブ（5 分）と不整合。

## 現状

| 問題 | 対象 | 現状 |
|---|---|---|
| タグ制限なし | `release.yml:5-6` | `tags: "*"` |
| OUT_DIR 非決定性 | `release.yml:119` | `find ... | head -1` |
| タイムアウト不整合 | `release.yml:169` | `timeout-minutes: 20` |

## 設計方針

1. タグパターンを `[0-9][0-9][0-9][0-9].[0-9]+.[0-9]*` に制限する
2. OUT_DIR 検出を `cargo metadata` ベースの決定論的手法に変更するか、`find` に最新ディレクトリを選択するオプションを追加する
3. タイムアウトを `5` に統一する

## 完了条件

- タグトリガーがバージョン形式タグに制限されていること
- OUT_DIR 検出が決定論的であること
- Slack 通知のタイムアウトが 5 分で統一されていること
