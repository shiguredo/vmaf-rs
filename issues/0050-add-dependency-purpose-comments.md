# Cargo.toml / fuzz/Cargo.toml / pbt/Cargo.toml の依存に用途コメントを追加する

- Priority: Medium
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/add-dependency-purpose-comments
- Polished: 2026-06-06

## 目的

AGENTS.md:137「依存ライブラリには用途をコメントで明記すること」に従い、用途コメントが欠落している依存にコメントを追加する。

## 優先度根拠

- AGENTS.md の明示的な規約違反。該当箇所は 4 箇所のみで修正コストは低い。

## 現状

| ファイル | 行 | 依存 | 不足 |
|---|---|---|---|
| `Cargo.toml` | 44 | `bindgen = "0.72"` | 用途コメントなし |
| `Cargo.toml` | 45 | `shiguredo_toml = "2026.2"` | 用途コメントなし |
| `fuzz/Cargo.toml` | 11 | `libfuzzer-sys = "0.4"` | 用途コメントなし |
| `pbt/Cargo.toml` | 8 | `shiguredo_vmaf = { path = ".." }` | 用途コメントなし |

なお `pbt/Cargo.toml:12` の `proptest = "1.6"` と `Cargo.toml:38-41` の `shiguredo_aom`, `shiguredo_libvpx`, `shiguredo_libyuv` には既に用途コメントが付与されている。

## 解決方法

各依存の直前に日本語の用途コメントを追加する:

- `bindgen = "0.72"` → `# libvmaf C ヘッダから Rust バインディングを生成する`
- `shiguredo_toml = "2026.2"` → `# build.rs で Cargo.toml のメタデータをパースする`
- `libfuzzer-sys = "0.4"` → `# cargo-fuzz によるファジング用ランタイム`
- `shiguredo_vmaf = { path = ".." }` → `# PBT のテスト対象クレート`
