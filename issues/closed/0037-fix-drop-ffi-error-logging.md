# Drop 実装で FFI クリーンアップの戻り値を無視せずログ出力する

- Priority: High
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/fix-drop-ffi-error-logging
- Polished: 2026-06-06
- Completed: 2026-06-07

## 目的

`Context::drop` と `Picture::drop` で `vmaf_close` / `vmaf_picture_unref` の戻り値を完全に握り潰しているのを改善し、FFI エラー発生時にデバッグ情報を残せるようにする。

## 優先度根拠

- libvmaf がエラーを返した場合（既に破棄済みポインタの二重解放、内部状態破損等）、その事実が完全に失われ問題の原因特定が不可能になる。
- Drop 内では `Result` を返せないが、最低限のエラーログ出力は可能かつ必要。

## 現状

`src/lib.rs:292-298`:

```rust
impl Drop for Context {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            let _ = unsafe { sys::vmaf_close(self.inner) };
        }
    }
}
```

`src/lib.rs:404-409`:

```rust
impl Drop for Picture {
    fn drop(&mut self) {
        if self.owned {
            let _ = unsafe { sys::vmaf_picture_unref(&mut self.inner) };
        }
    }
}
```

両方とも `let _ =` で戻り値を握り潰している。

## 設計方針

戻り値が 0 以外（エラー）の場合に `eprintln!` でエラーメッセージを出力する。AGENTS.md に従いログメッセージは英語とする。将来的に tracing が導入されたら `tracing::warn!` に置き換える。

## 完了条件

- `Context::drop` で `vmaf_close` の戻り値が 0 以外の場合にエラーメッセージが標準エラー出力に表示されること
- `Picture::drop` で同様の処理が追加されていること
- 既存のテストが全て通過すること

## 解決方法

`src/lib.rs:292-298` を以下に変更する:

```rust
impl Drop for Context {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            let ret = unsafe { sys::vmaf_close(self.inner) };
            if ret != 0 {
                eprintln!("vmaf_close() failed with error code: {ret}");
            }
        }
    }
}
```

`src/lib.rs:404-409` も同様に変更する。
