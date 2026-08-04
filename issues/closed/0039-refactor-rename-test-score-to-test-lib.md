# tests/test_score.rs を tests/test_lib.rs にリネームし命名規約に準拠する

- Priority: High
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/refactor-rename-test-score-to-test-lib
- Polished: 2026-06-06
- Completed: 2026-06-07

## 目的

単体テストファイル名を AGENTS.md の命名規約に準拠させる。

## 優先度根拠

- AGENTS.md:155「単体テストのファイル名は `tests/test_<module>.rs` とし、`src/<module>.rs` に対応させる」に違反している。
- `test_score` はどのモジュールに対応するか不明確。

## 現状

`src/lib.rs` に対応する単体テストファイルが `tests/test_score.rs` になっている。正しくは `tests/test_lib.rs`。

## 設計方針

`tests/test_score.rs` → `tests/test_lib.rs` にリネームし、Makefile の `test-quick` ターゲット (`make test-quick` → `--test test_score` → `--test test_lib`) を修正する。

## 完了条件

- `tests/test_lib.rs` が存在すること
- `tests/test_score.rs` が存在しないこと
- `make test-quick` が正しく `--test test_lib` を参照すること
- CI の `cargo test --features source-build --test test_score` → `--test test_lib` が修正されていること

## 解決方法

1. `git mv tests/test_score.rs tests/test_lib.rs`
2. `Makefile:9` の `--test test_score` → `--test test_lib`
3. `.github/workflows/ci.yml:70` の `--test test_score` → `--test test_lib`
4. `prek.toml:41` の `--test test_score` → `--test test_lib`
