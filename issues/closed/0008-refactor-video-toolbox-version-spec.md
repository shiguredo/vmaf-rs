# shiguredo_video_toolbox のバージョン指定をマイナーまでにする

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/refactor-video-toolbox-version-spec
- Completed: 2026-06-02

## 目的

dev-dependency のバージョン指定が AGENTS.md のライブラリ規約（マイナーまで）に違反している。規約に合わせる。後方互換性に影響しない指定形式の整理のためカテゴリは refactor とする。

## 現状

`Cargo.toml:44` の `shiguredo_video_toolbox = "2026.1.0"` はパッチバージョンまで指定している。AGENTS.md:134「バージョン番号はマイナーバージョンまで指定すること」に違反。他の依存（`Cargo.toml:33, 35, 36, 39, 40`）はマイナーまでで準拠している。

Cargo のキャレット解釈では `"2026.1.0"` と `"2026.1"` は同一の許容レンジ（`>=2026.1.0, <2027.0.0`）になるため、依存解決・ビルド結果に差はなく後方互換に影響しない。

## 設計方針

`Cargo.toml:44` を `shiguredo_video_toolbox = "2026.1"` に変更する。

CHANGES.md への記載は不要とする。公開 API に影響しない開発専用依存の指定形式の整理であり、リリースノートに載せる粒度ではない。

## 完了条件

- `Cargo.toml:44` が `shiguredo_video_toolbox = "2026.1"` になっていること
- macOS で `cargo build` / `cargo test --test test_codec_vmaf` が通ること（video_toolbox は macOS のみの dev-dependency）

## 解決方法

コード変更は行わず、完了条件が既に満たされているためクローズする（triage-issues での判定）。

- 完了条件である `Cargo.toml:44` は既に `shiguredo_video_toolbox = "2026.1"`（マイナーまで）になっている。本文「現状」が記す `"2026.1.0"`（パッチまで）は存在しない。
- `git blame` で確認したところ、`Cargo.toml:44` は唯一のコミット `a72df5b`（インポート）以来ずっと `"2026.1"` であり、`"2026.1.0"` が記録された履歴は存在しない。issue ファイルと `Cargo.toml` は同一コミットでインポートされているため、本 issue の「現状」記述が作成当初から事実と食い違っていた（誰かが後から直した形跡はない）。
- 他の依存（`shiguredo_aom` / `shiguredo_libvpx` / `shiguredo_libyuv` / `bindgen` / `shiguredo_toml`）もすべてマイナーまでの指定で、`AGENTS.md:134` 違反は残っていない。

以上より対応すべき変更は無く、実装済み（前提の現状記述が誤り）としてクローズする。
