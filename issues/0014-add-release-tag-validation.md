# Release のタグ発火にバージョン検証と canary publish ガードを入れる

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-06-06
- Model: Opus 4.8
- Branch: feature/add-release-tag-validation

## 目的

`release.yml` が `tags: "*"` で任意タグから本番 `cargo publish` まで走り、バージョン検証も無く、canary タグでも publish しようとする設計矛盾がある。タグ検証と canary publish ガードという安全装置を追加する。CI / Release 設定への追加で公開 API・配布物に影響しないため add とする。

## 優先度根拠

誤ったタグで意図しない crates.io 公開が走るリスク。Medium。

## 現状（行番号は実ファイルと一致を確認済み）

- `release.yml:5-6`: `tags: "*"` で任意タグが発火する
- `release.yml:28-30`: `get_version` は `refs/tags/` を剥がすだけで、バージョン形式や `Cargo.toml:3`（version = `2026.0.0`）との一致を検証しない
- `release.yml:36`: canary 判定は `contains(VERSION, 'canary')` で prerelease にするだけ
- `release.yml:123-135`: publish ジョブは canary でも無条件で走り、canary を crates.io に publish しようとする
- `build.rs:157-160`: prebuilt download URL は `releases/download/{CARGO_PKG_VERSION}/`（= `2026.0.0`、Cargo.toml 由来・タグ非依存）を使う。一方 `release.yml:118` の `gh release upload` はタグ名（`needs.github-release.outputs.version`）にアップロードする。タグ名と `CARGO_PKG_VERSION` が一致しないとアセットの置き場所と取得先が食い違う

## タグ命名規則（本 issue で確定する）

`build.rs` が `CARGO_PKG_VERSION`（`v` なし CalVer `2026.0.0`）で prebuilt URL を組むため、タグも `v` を付けない。

- 通常リリースタグ: `Cargo.toml` の version と完全一致（例: `2026.0.0`）。正規表現 `^[0-9]{4}\.[0-9]+\.[0-9]+$`
- canary タグ: `{Cargo.toml version}-canary.{N}`（例: `2026.0.0-canary.0`）。正規表現 `^[0-9]{4}\.[0-9]+\.[0-9]+-canary\.[0-9]+$`

CalVer 運用ではリリースのたびにタグを打つ前に `Cargo.toml` の version を更新する前提。version 一致検証はこの更新忘れを捕捉する。

## 設計方針

### バージョン検証

`github-release` ジョブの `get_version`（`release.yml:28-30`）直後に検証ステップを追加する。`Cargo.toml` の version を `cargo metadata --format-version 1 | jq -r '.packages[0].version'` 等で取得し、タグと照合する。

- 通常タグ: タグ == `Cargo.toml` version でなければ `exit 1`
- canary タグ: `-canary.N` を除いたベース部分が `Cargo.toml` version と一致しなければ `exit 1`
- 上記いずれの正規表現にも合致しないタグは `exit 1`

`github-release` が最初に落ちることで後続ジョブ（build-prebuilt / publish）は動かない。

### canary publish ガード

publish ジョブ（`release.yml:123-135`）に条件を付けて canary ではスキップする。

```yaml
publish:
    if: ${{ !contains(needs.github-release.outputs.version, 'canary') }}
```

`slack_notify`（`release.yml:137-151`）は `needs: [..., publish]` かつ `if: always()` のため、publish が skip された場合に skip を failure 扱いしないことを確認する。

### canary 時の build-prebuilt（要判断）

canary タグでは `build.rs` の prebuilt URL（`CARGO_PKG_VERSION` = canary を含まないベース version）とアップロード先タグ名（canary 込み）が一致せず、デフォルト経路では取得できない。かつ publish もしないため crates.io 経由の利用者もいない。したがって canary では build-prebuilt をスキップする方針を基本とする（GitHub prerelease のみ作成）。手動テスト用に canary の prebuilt を残したい場合は、0012 で検討する version 上書き機構と合わせて別途決める。

## canary タグの挙動（確定）

| ジョブ | 通常タグ | canary タグ |
|---|---|---|
| github-release | release 作成 | prerelease 作成 |
| build-prebuilt | 実行 | スキップ（基本方針） |
| publish | 実行 | スキップ |
| crates.io 公開 | される | されない |

## CHANGES.md

Release 設定への追加で公開 API・配布物に影響しないため、CHANGES.md への記載は不要とする（0011 / 0013 と同方針）。

## 関連 issue との整合

- 0023（publish ジョブに timeout 追加）は同じ publish ジョブ（`release.yml:116-128`）を編集する。番号順 0014 → 0023 で 0014 が先行し、0023 がリベースで本 issue の `if:` を取り込む
- 0012（prebuilt 経路テスト）の前提を本 issue の version 一致検証が保証する。タグ名 == `CARGO_PKG_VERSION` でないと利用者の prebuilt download が 404（`build.rs:128-130` で panic）になるため、本検証はその整合を守る安全装置

## 完了条件

- タグ形式の正規表現（通常 / canary）に合致しないタグ、または `Cargo.toml` version と不一致のタグで `github-release` ジョブが fail する検証ステップが存在すること（静的に確認）
- publish ジョブに canary をスキップする `if:` 条件が記述されていること（静的に確認）
- canary タグの各ジョブ挙動が上表どおりに制御されていること
