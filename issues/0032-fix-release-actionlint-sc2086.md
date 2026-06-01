# release.yml の shellcheck SC2086 (クォート漏れ) を解消する

- Priority: Low
- Created: 2026-06-01
- Model: Opus 4.8

## 目的

`actionlint .github/workflows/release.yml` を実行すると、埋め込み shell スクリプト内の変数クォート漏れにより shellcheck SC2086 (Double quote to prevent globbing and word splitting) が info レベルで 2 件報告される。これを解消し、actionlint がクリーンに通る状態にする。

この問題は issue 0004 (release.yml の権限設定) の対応中に検出されたが、権限設定とは無関係な既存の問題であり、スコープ外として本 issue に分離した。変更前の develop でも同数報告される。

## 優先度根拠

SC2086 は info レベルの shell スタイル警告であり、workflow を invalid にするものではない。`$GITHUB_OUTPUT` への書き込み自体は実害なく動作している。早急な対応は不要だが、actionlint をクリーンに保つことで将来の workflow 変更時に本当の問題を見落とさないようにする。Low。

## 現状

`.github/workflows/release.yml` の以下 2 箇所で `$GITHUB_OUTPUT` がダブルクォートされていない。

- `github-release` ジョブの「Get the version」ステップ:

  ```yaml
  run: echo "VERSION=${GITHUB_REF/refs\/tags\//}" >> $GITHUB_OUTPUT
  ```

- `build-prebuilt` ジョブの「Find OUT_DIR」ステップ:

  ```yaml
  echo "OUT_DIR=${OUT_DIR}" >> $GITHUB_OUTPUT
  ```

いずれも `>> $GITHUB_OUTPUT` の変数がクォートされておらず、actionlint 経由の shellcheck が SC2086 を報告する。

## 設計方針

該当箇所の `$GITHUB_OUTPUT` を `"$GITHUB_OUTPUT"` にクォートする。`GITHUB_OUTPUT` 環境変数のパスにスペースが含まれることは通常ないが、shellcheck の推奨に従いクォートすることで word splitting / globbing のリスクを排し、actionlint をクリーンに通す。

`ci.yml` にも `>> $GITHUB_OUTPUT` のような同種パターンがあれば、あわせて確認・修正することを検討する (本 issue の主対象は release.yml)。

## 完了条件

- `release.yml` の 2 箇所の `$GITHUB_OUTPUT` がダブルクォートされていること
- `actionlint .github/workflows/release.yml` が SC2086 を報告しないこと
- workflow の挙動が変わらないこと (出力変数の値が従来どおり設定されること)
