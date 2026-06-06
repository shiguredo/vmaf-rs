# ContextConfig に cpumask / gpumask フィールドを追加する

- Priority: Low
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/add-contextconfig-cpumask-gpumask
- Polished: 2026-06-06

## 目的

`VmafConfiguration` の `cpumask` と `gpumask` が常に 0 に固定されているのに対し、呼び出し側が制御できるようにフィールドを追加する。または 0 の意味をコメントで明記する。

## 優先度根拠

- libvmaf が CPU/GPU affinity をサポートする場合に設定不可なのは API の不足。
- 最低限、0 が「自動」を意味するのか「全コア無効」なのかをコメントで明記すべき。

## 現状

`src/lib.rs:191-197`:
```rust
let cfg = sys::VmafConfiguration {
    log_level: config.log_level.to_sys(),
    n_threads: config.n_threads,
    n_subsample: config.n_subsample,
    cpumask: 0,
    gpumask: 0,
};
```

## 設計方針

1. `ContextConfig` に `cpumask: u64` と `gpumask: u64` を追加し、デフォルト値を 0 とする
2. または、libvmaf のソースコードを確認し、0 の意味をコメントで明記する

## 完了条件

- `ContextConfig` に `cpumask` / `gpumask` が追加されている、または 0 の意味がコメントで明記されていること
