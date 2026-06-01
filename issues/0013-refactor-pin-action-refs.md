# GitHub Actions の @main 参照を commit SHA に固定する

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/refactor-pin-action-refs

## 目的

CI / Release で `shiguredo/github-actions` の複数アクションを `@main` で参照しており、ミュータブルな参照がサプライチェーンリスクになる。commit SHA に固定する。CI / Release 設定の予防的ハードニングで公開 API・配布物に影響しないため refactor とする（0011 が同種の CI 設定変更を refactor と判定したのに揃える）。

## 優先度根拠

シークレット（SLACK_WEBHOOK / GH_TOKEN）を扱うアクションを含むため、固定が望ましい。自社リポジトリのため緊急度は中。Medium。

## 現状（行番号は実ファイルと一致を確認済み）

- `ci.yml:30, 63`: `shiguredo/github-actions/.github/actions/rust-cache@main`
- `ci.yml:95`, `release.yml:146`: `shiguredo/github-actions/.github/actions/slack-notify@main`
- slack-notify は `secrets.SLACK_WEBHOOK`（`ci.yml:98`, `release.yml:149`）と `GH_TOKEN`（`ci.yml:102`）を扱う
- `actions/checkout`（`ci.yml:27,53,76`, `release.yml:26,70,130`）と `rust-lang/crates-io-auth-action`（`release.yml:131`）は既に `<SHA> # vX.Y.Z` 形式で SHA 固定済み
- `@main` 参照は上記 4 箇所のみ（`grep '@main' .github/` で確認）

## 設計方針

`@main` 参照を **commit SHA に固定する**（タグは force-push で移動可能でありミュータブルなので採らない。シークレットを扱う slack-notify には特に不適）。`actions/checkout@<SHA> # v6.0.2` と同じ方針・記法に揃える。

`shiguredo/github-actions` は monorepo の複合アクション（`.github/actions/rust-cache` 等のサブパス）で独立したリリースタグが無いため、リポジトリの commit SHA に固定する。`@main` の SHA は次のように解決する。

```
gh api repos/shiguredo/github-actions/commits/main --jq '.sha'
```

固定後は `shiguredo/github-actions/.github/actions/rust-cache@<40 桁 SHA> # main YYYY-MM-DD` のように、参照した時点が分かるコメントを併記する。

SHA 固定すると rust-cache / slack-notify の修正が自動で入らず手動更新が必要になる。追従を自動化するなら `.github/dependabot.yml` に `package-ecosystem: github-actions` を設定する（ただしサブパス複合アクションの更新追従に制約があるため効果は要確認）。

## CHANGES.md

CI / Release の設定変更で公開 API・配布物に影響しないため、CHANGES.md への記載は不要とする（0011 と同方針）。

## 関連 issue との整合

- 0010（closed、対応見送り。build.rs の prebuilt ダウンロード経路のハードニング）と同じサプライチェーン強化の思想だが、本 issue の対象は CI のアクション参照で別物
- release.yml で本 issue が触るのは slack_notify ジョブの 1 行（`release.yml:146`）のみ。0004（トップレベル permissions + github-release ジョブ）・0023（トップレベル concurrency + publish ジョブ）とは編集箇所が離れ、行レベルの直接衝突は無い。番号順（0004 → 0011 → 0013 → 0023）で進める

## 完了条件

- `ci.yml:30, 63, 95` と `release.yml:146` の `@main` がすべて 40 桁 commit SHA + バージョン/日付コメントに固定されていること
- `grep -rn '@main' .github/` がヒット 0 件であること
- すべての `uses:` 参照が SHA 固定（既存の checkout / crates-io-auth-action と同形式）であること
