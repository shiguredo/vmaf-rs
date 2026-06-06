# build.rs の macOS x86_64 ターゲットをサポートする

- Priority: Medium
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/add-macos-x86-64-target
- Polished: 2026-06-06

## 目的

`get_target_platform()` で macOS x86_64 が未サポートでパニックするのを修正し、より明確なエラーメッセージかサポートを提供する。

## 優先度根拠

- macOS x86_64 は依然として広く使われており、未サポートであることがビルド時のパニックでのみ判明するのはユーザー体験が悪い。
- 最低限、未サポートであることを明確に伝えるエラーメッセージが必要。

## 現状

`build.rs:346-350`:
```rust
match (target_os.as_str(), target_arch.as_str()) {
    ("linux", "x86_64") => format!("{}_x86_64", detect_linux_distro()),
    ("linux", "aarch64") => format!("{}_arm64", detect_linux_distro()),
    ("macos", "aarch64") => "macos_arm64".to_string(),
    _ => panic!("unsupported target: os={}, arch={}", target_os, target_arch),
}
```

macOS x86_64 は `_` アームに落ちてパニックする。

## 設計方針

1. 可能であれば macOS x86_64 ターゲットを追加する（prebuilt バイナリ提供が必要）
2. できない場合は `_` アームのパニックメッセージをより具体的にし、macOS x86_64 が未サポートであることを明示する

## 完了条件

- macOS x86_64 がサポート対象に追加されている、またはパニックメッセージが改善されていること
- 既存のターゲットが引き続き動作すること
