# release.yml の権限設定を明示する

- Priority: High
- Created: 2026-05-29
- Completed: 2026-06-01
- Polished: 2026-05-31
- Model: Opus 4.8

## 目的

`release.yml` の `github-release` ジョブに `permissions` ブロックが無く、Organization / リポジトリのデフォルト権限が read-only の環境では `gh release create` が 403 で失敗する。権限を明示する。

## 優先度根拠

リリースパイプラインが動かない可能性が高く、リポジトリの Workflow permissions を read-only に変更した場合に致命的。壊れたリリースパイプラインの不具合であり High。

## 現状

`release.yml` の `github-release` ジョブには `permissions` 指定が無い。`gh release create` は `contents: write` を必要とする（GitHub API の `POST /repos/{owner}/{repo}/releases` が `contents: write` を要求）が、権限が明示されていない。

`github-release` ジョブは `outputs.version` を提供しており、`build-prebuilt` ジョブが `needs.github-release.outputs.version` で参照している。つまり `github-release` が 403 で失敗すると、`build-prebuilt` と `publish` も連鎖的に失敗する。`slack_notify` のみ `always()` で実行される。

また `release.yml` にはトップレベルの `permissions` 指定が無い。トップレベルの `permissions` 指定がないため、権限が環境依存になる。

## 設計方針

トップレベルに deny-by-default の最小権限を置き、`github-release` ジョブで必要なぶんだけ昇格する。

実装例:

```yaml
permissions:
  contents: read

jobs:
  github-release:
    permissions:
      contents: write
```

トップレベル `permissions: contents: read` により、権限ブロックを持たない将来のジョブも Organization デフォルトに依存せず read に固定される。ジョブ単位の `permissions` はトップレベルを上書きするため、既存ジョブ（`build-prebuilt`、`publish`、`slack_notify`）の権限は影響を受けない。

## 完了条件

- `github-release` ジョブに `permissions: contents: write` が記述されていること
- トップレベルに `permissions: contents: read` が記述されていること
- `release.yml` が valid な YAML / workflow であること（`actionlint .github/workflows/release.yml` で確認）
- `CHANGES.md` の `## develop` に `[FIX]` エントリを追記すること

## 解決方法

`.github/workflows/release.yml` に権限設定を明示した。

- トップレベルに `permissions: contents: read`（deny-by-default の最小権限）を追加した。これにより権限ブロックを持たない将来のジョブも Organization デフォルトに依存せず read に固定される。
- `github-release` ジョブに `permissions: contents: write` を追加した。`gh release create` は GitHub Release 作成に `contents: write` を必要とするため、これで read-only 環境でも 403 で失敗しなくなる。

既存ジョブ（`build-prebuilt` は `contents: write`、`publish` は `id-token: write`、`slack_notify` は `actions: read`）はいずれもジョブ単位の `permissions` ブロックを持つ。GitHub Actions ではジョブに `permissions` があるとトップレベルを継承せずそのジョブの権限を完全に置き換えるため、今回のトップレベル追加による影響を受けない。

`actionlint .github/workflows/release.yml` で権限設定が valid な workflow であることを確認した。

### スコープ外の既知事項

`actionlint` は `release.yml` の 30 行目・90 行目で `$GITHUB_OUTPUT` 等のクォート漏れ（shellcheck SC2086, info レベル）を 2 件報告するが、これは変更前の develop でも同数報告される既存の shell スタイル警告であり、本 issue（権限設定）のスコープ外である。workflow を invalid にするエラーではない。別 issue として対応すべき。
