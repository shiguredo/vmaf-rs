# prebuilt ダウンロードのサプライチェーン耐性を強化する

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-05-29
- Closed: 2026-06-01
- Model: Opus 4.8
- Branch: feature/change-prebuilt-supply-chain-hardening

## クローズ理由（対応見送り）

本 issue が目指す「完全性検証を配布元（GitHub Releases）から分離する」(`0010:36-44`) は、自前で実装する限り費用対効果が見合わないと判断し、対応を見送る。

- 添付 `.sha256` ファイルも、GitHub が 2025-06 から API 公開している `asset.digest`（アップロード時に GitHub が自動計算する SHA256）も、いずれも GitHub と同一の信頼ドメインにある。アセットを差し替えられる攻撃者は digest も同時に差し替えられるため、どちらも「破損検知」止まりで「改竄検知」にならない
- ハッシュを `build.rs` に pin する案は chicken-and-egg な直列リリースフロー（`0010:42`）の新設と毎リリースの保守コストを伴ううえ、`release.yml` 全体が侵害されればソース内ハッシュごと書き換えられて無力（`0010:44`）。コストに対して防げる脅威が限定的
- トラストアンカーを GitHub の外に出す手段（自前鍵での署名）は鍵管理・依存追加を伴い、本ライブラリの規模に対して過剰。AGENTS.md「依存は最小限」とも衝突する
- GitHub の外に頼らず効くのは Immutable Releases（公開後のアセット差し替え自体を禁止する GitHub の設定）程度だが、現時点では導入しない判断

現状の SHA256 検証（破損検知）で当面十分とする。将来 prebuilt の改竄検知が本当に必要になった場合は、ハッシュ pin ではなく Immutable Releases 有効化 + artifact attestation（sigstore）を起点に再検討する。なお tar 展開の path traversal / symlink 検査（`0010:46-48`）はサプライチェーン議論とは独立して意味があるため、必要なら別 issue として切り出す。

## pending にした理由

本 issue は確定していない設計判断と外部依存追加を複数伴うため、AGENTS.md「外部依存の追加や設計判断が必要で保留中の issue は `issues/pending/` に置くこと」に従い保留する。確定すべき設計は以下。

1. 完全性検証の方式: 期待ハッシュをソースに pin する方式か、リリース署名（cosign / sigstore など、鍵を別チャネルで管理）か
2. pin 方式を採る場合の release.yml 改修（後述の chicken-and-egg を解く直列フロー）
3. tar 展開の安全化に Rust の `tar` crate（build-dependency 追加）を使うか、システム tar + 事前列挙検証にとどめるか
4. 防御する脅威モデルの確定（GitHub Release アセット単体の差し替えか、リリースパイプライン全体の侵害か）

これらは prebuilt を通る全利用者に影響する設計判断であり、確定後に着手する。ブランチ prefix（`change` か `fix` か）も方式確定後に見直す。

## 目的

`build.rs` の prebuilt ダウンロード経路は、チェックサムを成果物と同一サーバから取得し、tar 展開と `bindings.rs` の取り込みを無検証で行うため、リリースが侵害された場合に改竄を検出できず任意コード実行の余地がある。検証を強化する。

## 優先度根拠

デフォルト feature で全利用者が通る経路のサプライチェーンリスク。即時の悪用条件は限定的だが、公開ライブラリとして看過できない。Medium。

## 現状（行番号は実コードと一致を確認済み）

- `build.rs:107-113`: `.tar.gz` と `.sha256` を同一の GitHub Releases アセットから取得している。`verify_sha256`（`build.rs:178-194`）はダウンロードした `.sha256` ファイルと照合するだけで、ソースに pin した値とは比較していない。したがってリリースが侵害されれば両方差し替え可能で、SHA256 検証は「壊れたダウンロードの検出」にしかならず改竄防御にならない
- `build.rs:147-155`: システム `tar xzf` に丸投げで、path traversal（`../` を含むパス・絶対パス）や symlink / hardlink を検査していない
- `build.rs:167-172` + `src/sys.rs:9`: 展開した `bindings.rs` を無検証で `include!` する。`bindings.rs` は通常の Rust コードとしてコンパイルされるため、差し替えで任意コード混入の余地

## 設計方針（要確定）

### 完全性検証

期待ハッシュをソース側（`build.rs` か `Cargo.toml` メタデータ）に pin し、ダウンロードした成果物と照合する。これにより検証チャネルが成果物の配布元（GitHub Releases）と分離する。

ただし次の制約・限界を踏まえて方式を確定すること。

- chicken-and-egg: crates.io に publish される `build.rs` にハッシュを埋めるには、成果物（5 ターゲット分、`release.yml:48-58` の matrix）が確定した後にハッシュをソースへ書き戻して publish する必要がある。現状の `release.yml` は `build-prebuilt`（40-114）と `publish`（116-128）がタグ push で動き、ハッシュ書き戻しの step がない。pin 方式を採るなら「prebuilt ビルド → 5 ハッシュ確定 → ソース commit → cargo publish」の直列フローを `release.yml` に新設する必要がある
- 保守コスト: 毎リリースで 5 ターゲット分のハッシュを更新する運用になる。上記の自動化が前提
- 脅威モデルの限界: pin は「Release アセット単体の差し替え」には有効だが、`release.yml` 全体が侵害されれば成果物とソース内ハッシュを同時に書き換えられるため無力。パイプライン全体侵害まで防ぐにはリリース署名（鍵を別管理）の検討が要る

### tar 展開の安全化

検証は展開「前」に行う必要がある（システム tar に展開させた後では traversal は既に発生済み）。`tar tzf` で内容を列挙して `../`・絶対パス・symlink / hardlink エントリを弾くか、Rust の `tar` crate を build-dependency に追加してエントリを検査しながら展開する。後者は依存追加の設計判断（AGENTS.md「依存は最小限」）を伴う。

### bindings.rs の取り込み

`bindings.rs` はアーカイブ（`vmaf-{target}.tar.gz`）の中身そのもの（`release.yml:94-95`）であり、アーカイブ全体のハッシュを pin すれば自動的に検証対象に含まれる。独立した対策は不要で、完全性検証（ハッシュ pin）の従属項目。

## 関連 issue との整合

- 0027（certutil デッドコード削除）は `compute_sha256`（`build.rs:197-245`）を触る。本 issue がハッシュ検証ロジックを pin 方式へ作り替える場合、`compute_sha256` 自体は残る想定なら 0027 のデッドコード削除は独立して成立する。番号順は 0010 → 0027
- 0013（action ref の pin）も同じサプライチェーン強化の思想だが対象は CI のアクション参照で、本 issue（build.rs のダウンロード経路）とは別物
- 0003 が触る docs.rs 分岐（`build.rs:39-79`、`DOCS_RS` 時に 78 行で early return）はダウンロード経路に到達しないため、本 issue と経路が分離し衝突しない

## 完了条件（方式確定後に精緻化する）

- prebuilt の完全性検証が成果物と同一サーバ依存でなくなること（pin したハッシュ、または署名検証）
- tar 展開「前」にパス検証が入り、traversal / symlink エントリを弾くこと
- `bindings.rs` の取り込みが完全性検証の対象に含まれること
- pin 方式を採る場合、`release.yml` にハッシュ書き戻しの直列フローが入っていること
- `CHANGES.md` の `## develop` に確定した種別のエントリを `- @voluntas` 付きで追記すること
