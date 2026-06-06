# Error::check で libvmaf 戻り値の符号を検証する

- Priority: High
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/fix-error-fmt-negative-os-error
- Polished: 2026-06-06

## 目的

`Error::check` で `code != 0` のすべての非ゼロ戻り値を負の errno と暗黙に仮定しているのを明示的に検証し、libvmaf が正のエラーコードを返した場合の未定義動作を防ぐ。

## 優先度根拠

- libvmaf の将来バージョンで正のエラーコードが導入された場合、`Display` 実装の `from_raw_os_error(-code)` に負数が渡り未定義動作の可能性がある。
- AGENTS.md の「性能より堅牢性を優先すること」に合致する予防的措置。
- 変更は `assert!` 1 行の追加のみでリスクはゼロ。

## 現状

`src/lib.rs:78-86`:

```rust
impl Error {
    fn check(code: c_int, function: &'static str) -> Result<(), Self> {
        if code == 0 {
            Ok(())
        } else {
            Err(Self::Ffi { code, function })
        }
    }
}
```

`src/lib.rs:88-98` の `Display` で `std::io::Error::from_raw_os_error(-code)` を呼んでいるが、`-code` の符号反転が安全であるためには `code < 0` が前提となる。この前提がコード上で表明されていない。

`Error` のドキュメントコメント（`src/lib.rs:69`）に「libvmaf FFI 由来エラー (負の errno code)」と記載されているが、これはコメント上の仮定に過ぎず実行時検証がない。

## 設計方針

`Error::check` 内で `code < 0` を `assert!` で検証する。

- `assert!` を選択する理由: `code` が非ゼロかつ非負数である状況は libvmaf の正常動作では発生せず、発生した場合は実装バグであるためパニックが適切
- assert メッセージは英語で「libvmaf returned non-negative error code: {code}」とする
- `Error` バリアントの分割（`FfiNegative` / `FfiPositive`）は YAGNI 違反となるため採用しない

## 完了条件

- `Error::check` 内で非ゼロ戻り値に対して `code < 0` の assert 検証が追加されていること
- 既存のテストが全て通過すること
- `Error::Display` の `from_raw_os_error(-code)` が負数前提で安全に呼ばれることが保証されていること

## 解決方法

`src/lib.rs:79-85` を以下に変更する:

```rust
fn check(code: c_int, function: &'static str) -> Result<(), Self> {
    if code == 0 {
        Ok(())
    } else {
        assert!(code < 0, "libvmaf returned non-negative error code: {code}");
        Err(Self::Ffi { code, function })
    }
}
```
