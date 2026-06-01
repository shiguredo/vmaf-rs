# shiguredo_video_toolbox のバージョン指定をマイナーまでにする

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/refactor-video-toolbox-version-spec

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
