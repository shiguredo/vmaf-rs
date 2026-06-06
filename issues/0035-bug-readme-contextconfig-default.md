# README.md のコード例の ContextConfig::new() を ContextConfig::default() に修正する

- Priority: High
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/fix-readme-contextconfig-default
- Polished: 2026-06-06

## 目的

README.md のコード例がコンパイル不可能な API を参照しているのを修正し、新規ユーザーがコピペで使い始められるようにする。

## 優先度根拠

- README は新規ユーザーの最初の接点であり、ここでコンパイルエラーになると悪印象を与える。
- `ContextConfig::new()` は issue 0020 で削除済みであり、README の追従漏れ。

## 現状

`README.md:84`:

```rust
let mut ctx = Context::new(ContextConfig::new())?;
```

`ContextConfig` は `src/lib.rs:114-122` で `Default` のみ実装しており、`new()` 関連関数は存在しない。issue 0020 (`0020-refactor-context-config-new-default`) で削除された。

## 設計方針

`ContextConfig::new()` → `ContextConfig::default()` に修正するのみ。

## 完了条件

- README.md のコード例が `ContextConfig::default()` を使用していること
- コード例の前後で `ContextConfig::new()` が残っていないこと

## 解決方法

`README.md:84` を以下に修正する:

```rust
let mut ctx = Context::new(ContextConfig::default())?;
```
