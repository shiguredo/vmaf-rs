# CHANGES.md の凡例ブロックと clippy.toml の無効設定を削除する

- Priority: Low
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/refactor-remove-redundant-metadata
- Polished: 2026-06-06

## 目的

1. `CHANGES.md:3-10` の CHANGE/ADD/UPDATE/FIX 凡例ブロックは AGENTS.md と重複しているため削除する
2. `clippy.toml:3-8` の `allow-unwrap-in-tests` 等の設定は、Cargo.toml に対応する lint deny が存在しないため無効。AGENTS.md の `.unwrap()` 禁止規約とも矛盾するため削除する

## 優先度根拠

- 重複ドキュメントは一貫性の維持を困難にする
- 無効な設定は誤解を招く
- 影響範囲が狭く、削除リスクはゼロ

## 現状

`CHANGES.md:3-10`:
```markdown
- UPDATE
  - 後方互換がある変更
- ADD
  ...
```

`clippy.toml:3-8`（全体 :1-8）:
```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
```

## 解決方法

1. `CHANGES.md:3-10` の凡例ブロックを削除する
2. `clippy.toml` の 3 設定を削除する。コメント行のみが残る場合はファイルごと削除する
