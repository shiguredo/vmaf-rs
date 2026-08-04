# Release ワークフローに concurrency と timeout を追加する

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-06-06
- Completed: 2026-06-06
- Model: Opus 4.8
- Branch: feature/refactor-release-concurrency-timeout

## 目的

`release.yml` に concurrency 設定が無く、github-release / publish ジョブに timeout が無い。多重起動の競合とネットワーク吊られ時の長時間ブロックを防ぐ。CI / Release 設定の予防的ハードニングで公開 API・配布物に影響しないため refactor とする。

## 優先度根拠

通常運用では顕在化しにくいが、リリースの堅牢性に関わる。Low。

## 現状（行番号は実ファイルと一致を確認済み）

- `release.yml` 全体に concurrency 設定が無い（`ci.yml:14-16` にはある）。同一タグ再 push や近接タグ push で `gh release upload --clobber`（`release.yml:114`）と cargo publish が競合し得る
- timeout-minutes が無いジョブが 2 つある: `github-release`（`release.yml:12-38`、`gh release create` がハングし得る）と `publish`（`release.yml:116-128`、`cargo publish` がネットワークで吊られるとデフォルト 360 分待つ）。build-prebuilt（`release.yml:60`、30 分）と slack_notify（`release.yml:132`、20 分）は timeout を持つ

## 設計方針

### concurrency

`release.yml` のトップレベルに concurrency を追加する。

```yaml
concurrency:
  group: release
  cancel-in-progress: false
```

- `group` は固定値 `release` とし、全リリースを 1 本に直列化する（同一タグ再 push・近接タグ push の両方を競合させない）。`${{ github.ref }}` を含めると別タグが並走するため固定値にする
- `cancel-in-progress: false` とする。リリース途中で cancel すると release 作成・asset upload・cargo publish が中断され壊れるため、`ci.yml` の `true` とは逆にする
- 注意: GitHub Actions の concurrency は 1 running + 1 pending しか保持せず、厳密な FIFO キューではない（3 本目以降の近接 push は pending が上書きされ得る）。よって「直列化」とは「同一 group の同時 running を 1 本に制限する」ことを指す

### timeout

timeout-minutes が無い 2 ジョブに付ける。`cargo publish` は通常数分なので publish は 15 分、`gh release create` のみの github-release は 10 分を目安とする。

## CHANGES.md

CI / Release の設定変更で公開 API・配布物に影響しないため、CHANGES.md への単独記載は不要とする（0011 / 0013 / 0022 / 0007 と同方針）。

## 関連 issue との整合

`release.yml` を触る issue が複数ある。番号順（0004 → 0013 → 0014 → 0023）で 0023 が最後発のため、先行 issue の編集を取り込んだ上で追加する。

- 0004（fix-github-release-permission）はトップレベルに `permissions` を追加する。本 issue はトップレベルに `concurrency` を追加するため編集箇所が近接する。0004 を先に取り込む
- 0014（add-release-tag-validation）は publish ジョブ（`release.yml:116-128`）に canary スキップの `if:` を追加する。本 issue は同じ publish ジョブに timeout を追加するため、0014 の `if:` を取り込んだ上で足す（0014 も「0023 がリベースで取り込む」と明記済み）
- 0013（refactor-pin-action-refs）は slack_notify の 1 行のみで直接衝突しない

## 完了条件

- `release.yml` のトップレベルに `concurrency: { group: release, cancel-in-progress: false }` が記述され、同一 group の同時 running が 1 本に制限されること（静的確認）
- `github-release` と `publish` の両ジョブに `timeout-minutes` が記述されていること（静的確認）
## 解決方法

- release.yml の concurrency 設定を改善した
