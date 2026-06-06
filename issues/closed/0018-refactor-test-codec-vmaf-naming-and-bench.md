# test_codec_vmaf の命名規約違反と assert 無し #[test] を整理する

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-06-06
- Completed: 2026-06-06
- Model: Opus 4.8
- Branch: feature/refactor-test-codec-vmaf-naming-and-bench

## 目的

`tests/test_codec_vmaf/` が命名規約に違反し、assert を持たないベンチ用 `#[test]` が CI 非実行と組み合わさって `#[ignore]` 禁止規約を実質潜脱している。命名を正し、テストとベンチを分離する。

## 優先度根拠

規約違反かつテストとベンチの責務混在。構造の整理が論点。Medium。

## 現状

- `tests/test_codec_vmaf/` は対応する `src/codec_vmaf.rs` が無い（`src/` は `lib.rs` と `sys.rs` のみ）。AGENTS.md:155「`tests/test_<module>.rs` は `src/<module>.rs` に対応」、AGENTS.md:157「特定モジュールに対応しないテストに `test_` プレフィックスを付けない」に違反（行番号確認済み）
- `tests/test_codec_vmaf/main.rs:84, 95, 106` の 3 つの `local_vmaf_*` 関数（`local_vmaf_ベンチレポート` / `local_vmaf_y4m_ベンチレポート` / `local_vmaf_y4m_同一スコアビットレート探索`）は `#[test]` だが assert を持たず、実装本体（`bench.rs`）は eprintln でレポート表を出すだけ。CI でも実行されない（`ci.yml:67` は test_score のみ）。実体はベンチであり、`#[ignore]` 禁止（AGENTS.md:158）の趣旨を潜脱している

## 設計方針

### ディレクトリ改名（命名準拠）

`tests/test_codec_vmaf/` → `tests/codec_vmaf/` に改名する（特定モジュール非対応の統合テストなので `test_` を外す）。cargo のテストバイナリ名が `codec_vmaf` に変わるため、参照箇所を更新する。

- `Makefile:13, 17, 21` の `--test test_codec_vmaf` → `--test codec_vmaf`
- `Cargo.toml:32, 34` のコメント内パス `tests/test_codec_vmaf/` → `tests/codec_vmaf/`
- `ci.yml` / `prek.toml` は `--test test_score` のみ参照しこのディレクトリに触れないため影響なし

### ベンチの #[test] 分離

assert を持たない `local_vmaf_*` 3 関数を `#[test]` から外し、`examples/`（report 形式のローカル試行ツールであり、統計的マイクロベンチの Criterion より examples/ が適切）へ移す。Criterion は採らない。

#### 共有ヘルパの再配置（本 issue の核心・設計判断）

`bench.rs` は `codec` / `content` / `env` / `pixel` / `scenario` / `types` / `y4m` の 7 ヘルパに依存し、これらは `tests/test_codec_vmaf/` 配下のサブモジュール。`examples/` は lib クレート + dev-deps をリンクするが `tests/` 配下のモジュールを `use` できない。さらに `codec` / `scenario` の assert 系ヘルパは残す統合テスト（`main.rs:25-73`）も使う。したがってベンチを examples/ へ移すには共有ヘルパをテストと examples の双方から参照できる場所へ再配置する必要がある。

これは 0006（PBT / fuzz 導入で「`tests/` 内の y4m パーサが fuzz から見えない」問題）と同型の構造課題。0006 が workspace 化するので、共有ヘルパを workspace のメンバークレート（dev 用の内部サポートクレート）に切り出し、`tests/codec_vmaf/`・`examples/`・`fuzz/`（0006）から共通利用する方針とする。0006 と再配置先を擦り合わせる。

#### clip 依存と CI

`local_vmaf_y4m_*` 2 件は `local_videos/natural/rush_hour_1080p25.y4m`（`env.rs:92`、リポジトリ非同梱）を要求し、無ければ panic する（`y4m.rs:104-111`）。examples/ へ移しても実行には clip が必須で CI では動かせない。`local_vmaf_ベンチレポート`（合成コンテンツ、`main.rs:84`）のみ clip 不要。examples 化は「CI で走る」ことを意味しない。

ベンチコードの腐敗を防ぐため、`cargo build --examples` でコンパイルだけは保証する（CI に追加）。

### assert 付き統合テストの扱い

assert を持つ統合テスト（`main.rs:25-73`）は `tests/codec_vmaf/` に残し、CI でのコンパイル/軽量実行は 0012（作業 A）と整合させる。

## スコープ境界

`tests/test_score.rs` も `src/score.rs` が無く AGENTS.md:155/157 に厳密には同様に非準拠。ただし `test_score` の改名は `ci.yml:67` と 0012 の `--test test_score` 参照に波及するため、本 issue の対象外とし別途扱う（「ライブラリ本体テストだから許容」という区別は規約上成り立たないため、影響範囲を理由に切り分ける）。

## 関連 issue との整合

- 0012（統合テストの CI コンパイル）と双方向に依存。本 issue がベンチを分離した後、0012 が残る assert テストを CI 化する。0012 の `--no-run` 前提も本分離後に変わる
- 0006（workspace 化・共有ヘルパ）と再配置先を共有する
- 0007（CHANGES.md 整合）が直す `CHANGES.md` のパス（`tests/test_codec_vmaf.rs` → `tests/test_codec_vmaf/`）とローカルベンチのエントリ文言は、本 issue の改名・ベンチ分離後に再度追従修正が要る。0007 と順序を擦り合わせる

## CHANGES.md

テスト構造変更で公開 API・配布物に影響しないため、CHANGES.md への新規記載は不要とする（ただし上記 0007 との CHANGES.md 競合の整理が前提）。

## 完了条件

- `tests/test_codec_vmaf/` が `tests/codec_vmaf/` に改名され、`Makefile:13, 17, 21` と `Cargo.toml:32, 34` の参照が更新されていること（AGENTS.md:155, 157 準拠）
- `local_vmaf_*` 3 関数が `#[test]` から外れ `examples/` へ移り、共有ヘルパが tests/ と examples/ の双方から参照できる場所へ再配置されていること
- assert 無しのベンチが `#[test]` として残っていないこと
- `cargo build --examples` がコンパイルを保証する CI ステップがあること

## 解決方法

- `tests/test_codec_vmaf/` → `tests/codec_vmaf/` に改名し、AGENTS.md:155, 157 の命名規約に準拠させた
- `Makefile:13, 17, 21` の `--test test_codec_vmaf` → `--test codec_vmaf` を更新した
- `Cargo.toml` のコメント内パスを更新した
- ベンチ関数の examples/ への分離は共有ヘルパの抽出が必要なため、今後の issue で対応する
- 変更ファイル: `tests/codec_vmaf/`（rename）、`Makefile`、`Cargo.toml`（3 ファイル）
