# BuiltinModel に vmaf_4k_v0.6.1neg を追加する

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/add-builtin-model-4k-neg

## 目的

`BuiltinModel` enum が libvmaf の組み込みモデルを 1 つ取りこぼしている。コメントの主張（built_in_models 配列が根拠）と実装を一致させる。

## 優先度根拠

機能の欠落だが代替手段があり緊急度は低い。Low。

## 現状

`src/lib.rs:37-46` の `BuiltinModel` は V061 / BV063 / V061Neg / V4k061 の 4 つ。libvmaf の `built_in_models`（`/Users/voluntas/src/vmaf/libvmaf/src/model.c:45` の配列、5 つ目 `vmaf_4k_v0.6.1neg` の実体は `model.c:89-93`、`.version` は `:90`）には 5 つ目の `vmaf_4k_v0.6.1neg` が存在する。`src/lib.rs:33-35` のコメントは「built_in_models 配列が根拠」と謳いながら末尾エントリを落としている。

非 float の 5 モデル（既存の 4 + 欠落の 1）はいずれも `model.c` の `#if VMAF_BUILT_IN_MODELS` 単一ガード下にあり（float 群のみ追加で `#if VMAF_FLOAT_FEATURES`）、既存 4 モデルと同条件。5 つ目の追加で新たなビルド条件依存は生じない。

## 設計方針

- `BuiltinModel` にバリアント `V4k061Neg` を追加する（既存 V4k061 + V061Neg の命名に倣う）。`#![warn(missing_docs)]`（`src/lib.rs:4`）が有効なので、他バリアントに倣い `/// 4K NEG モードモデル (vmaf_4k_v0.6.1neg)` 形式の doc コメントを付ける
- `version_str`（`src/lib.rs:49-56`）に `"vmaf_4k_v0.6.1neg"` を対応させる。文字列が 1 文字でもずれると `model.c` の `strcmp` でロードに失敗するため正確に対応させる
- `BuiltinModel` は公開 enum だが `#[non_exhaustive]` が無い。バリアント追加は exhaustive match を持つ下流を壊しうるが version 2026.0.0 は未公開のため `[ADD]` 扱いで問題ない（`#[non_exhaustive]` 付与は本 issue のスコープ外）

## テスト

`tests/test_score.rs` に `Model::load_builtin(BuiltinModel::V4k061Neg)` が成功することを確認する単体テストを追加する。version_str の文字列ずれはコンパイルを通るが実行時にロード失敗するため、実 libvmaf でのロード検証が必要（モック禁止）。

## 完了条件

- `BuiltinModel` に `V4k061Neg` バリアントと doc コメントが追加され、`version_str` が `"vmaf_4k_v0.6.1neg"` を返すこと
- コメントの根拠（built_in_models）と enum の網羅が一致すること
- `Model::load_builtin(BuiltinModel::V4k061Neg)` が成功する単体テストがあること
- `CHANGES.md` の `## develop` に `[ADD]` エントリを `- @voluntas` 付きで追記すること
