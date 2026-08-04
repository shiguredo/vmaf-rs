# Error::Display 実装の単体テストを追加する

- Priority: Medium
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/add-error-display-tests
- Polished: 2026-06-06
- Completed: 2026-06-07

## 目的

`Error::InvalidInput` と `Error::Ffi` の `Display` 実装、および `std::error::Error` 実装が正しく動作することを確認するテストを追加する。

## 優先度根拠

- ユーザーに表示されるエラーメッセージの品質は公開 API 品質の一部。
- `from_raw_os_error` への負数渡しや空文字列メッセージの挙動が未検証。
- テスト追加のコストが低い。

## 設計方針

各エラーバリアントの `to_string()` 出力文字列を assert する単体テストを追加する。

## 完了条件

- `InvalidInput("test message")` の Display 出力が期待通りであることのテスト
- `Ffi { code: -1, function: "test_func" }` の Display 出力が期待通りであることのテスト
- `std::error::Error` 実装が動作することの確認

## 解決方法

`tests/test_lib.rs` に以下を追加する:

```rust
#[test]
fn error_invalid_input_の表示が正しい() {
    let err = shiguredo_vmaf::Error::InvalidInput("test message");
    assert!(err.to_string().contains("test message"));
}
```
