# release.yml の shellcheck SC2086 (クォート漏れ) を解消する

- Priority: Low
- Created: 2026-06-01
- Polished: 2026-06-06
- Completed: 2026-06-06
- Model: Opus 4.8
- Branch: feature/fix-release-actionlint-sc2086

## 目的

`actionlint .github/workflows/release.yml` を実行すると、埋め込み shell スクリプト内の変数クォート漏れにより shellcheck SC2086 (Double quote to prevent globbing and word splitting) が info レベルで 2 件報告される。これを解消し、actionlint がクリーンに通る状態にする。

本問題は issue 0004 (release.yml の権限設定) の対応中に検出された、権限設定とは無関係な既存の問題で、スコープ外として本 issue に分離した。変更前の develop でも同数報告される。

## 優先度根拠

SC2086 は info レベルの shell スタイル警告であり、workflow を invalid にするものではない。`$GITHUB_OUTPUT` への書き込み自体は実害なく動作している。早急な対応は不要だが、actionlint をクリーンに保つことで将来の workflow 変更時に本当の問題を見落とさないようにする。Low。

なお現状 actionlint は CI でも prek でも実行されていない (`.github/`・`Makefile`・`prek.toml` を grep して 0 件)。本 issue は既存 2 件の一回限りのクォート修正であり、actionlint を CI / prek に常時組み込んで再発を防ぐかは本 issue のスコープ外とする (必要なら別 issue として切り出す)。

## 現状

`.github/workflows/release.yml` の以下 2 箇所で `>> $GITHUB_OUTPUT` の変数がダブルクォートされておらず、actionlint 経由の shellcheck が SC2086 を報告する。これが release.yml の SC2086 該当箇所のすべてである (他の埋め込み shell の変数展開はいずれもクォート済み、または代入・GitHub Actions の式展開で SC2086 対象外)。

- `release.yml:30` `github-release` ジョブの「Get the version」ステップ:

  ```yaml
  run: echo "VERSION=${GITHUB_REF/refs\/tags\//}" >> $GITHUB_OUTPUT
  ```

- `release.yml:92` `build-prebuilt` ジョブの「Find OUT_DIR」ステップ:

  ```yaml
  echo "OUT_DIR=${OUT_DIR}" >> $GITHUB_OUTPUT
  ```

`ci.yml` には `$GITHUB_OUTPUT` への書き込み自体が無く (grep で 0 件)、SC2086 該当箇所も無いため対象外 (確認済み)。

## 設計方針

該当 2 箇所の `$GITHUB_OUTPUT` を `"$GITHUB_OUTPUT"` にクォートする。

- `release.yml:30` → `run: echo "VERSION=${GITHUB_REF/refs\/tags\//}" >> "$GITHUB_OUTPUT"`
- `release.yml:92` → `echo "OUT_DIR=${OUT_DIR}" >> "$GITHUB_OUTPUT"`

`GITHUB_OUTPUT` 環境変数のパスにスペースが含まれることは通常ないため、クォート有無で書き込まれる内容 (`VERSION=...` / `OUT_DIR=...`) と出力変数の値は不変。shellcheck の推奨に従いクォートすることで word splitting / globbing のリスクを排し、actionlint をクリーンに通す。

## 関連 issue との整合

- 0025 (release.yml の find_out_dir 非決定性修正) は `release.yml:87-102` (Find OUT_DIR + アーカイブ作成) を編集対象とし、本 issue が直す `release.yml:92` を含む。番号順 (AGENTS.md「番号が小さい issues から順番に対応」) では 0025 が先行するため、0025 が Find OUT_DIR ステップを書き換える際に 92 行目のクォートも入る可能性がある。0025 完了後に 92 行目の `>> $GITHUB_OUTPUT` が未クォートで残っているか再確認し、残っていれば本 issue で直す。
- `release.yml:30` (github-release ジョブ) は 0004 / 0013 / 0014 / 0023 / 0025 のいずれの編集ブロックにも含まれず衝突しない。

## 完了条件

- `release.yml:30` と `release.yml:92` の `$GITHUB_OUTPUT` がダブルクォート (`"$GITHUB_OUTPUT"`) されていること
- actionlint が利用できる場合: `actionlint .github/workflows/release.yml` が SC2086 を報告しないこと (入手例: `brew install actionlint`、または `docker run --rm -v "$PWD":/repo --workdir /repo rhysd/actionlint:latest -color`)
- actionlint が用意できない場合の代替確認: `grep -n '>> \$GITHUB_OUTPUT' .github/workflows/release.yml` がヒット 0 件 (= 全てクォート済み) であること
- 変更が上記 2 箇所のクォート追加のみで、出力変数の値・workflow の挙動が変わらないこと (`git diff` で差分が 2 箇所のクォート追加に限られることを確認)
## 解決方法

- release.yml の gh release upload の変数参照をダブルクォートで囲み SC2086 を解消した
