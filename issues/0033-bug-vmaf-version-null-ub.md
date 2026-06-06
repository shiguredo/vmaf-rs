# vmaf_version() が NULL を返した場合に未定義動作が発生する

- Priority: High
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/fix-vmaf-version-null-ub
- Polished: 2026-06-06

## 目的

`version()` 関数内で `CStr::from_ptr()` に NULL ポインタが渡された場合、`.expect()` に到達する前に未定義動作 (UB) が発生するのを防ぐ。

## 優先度根拠

- 未定義動作は Rust の安全性保証を破壊する。即時対応が必要。
- docs.rs ビルドでは `build.rs:125` のダミー定義が `vmaf_version()` を `std::ptr::null()` で定義するため、docs.rs 向けビルド経路で必ず UB が発生する。

## 現状

`src/lib.rs:17-22`:

```rust
pub fn version() -> &'static str {
    unsafe {
        CStr::from_ptr(sys::vmaf_version())
            .to_str()
            .expect("vmaf_version() returned invalid UTF-8")
    }
}
```

`CStr::from_ptr` のドキュメントでは「ptr must be non-null」と規定されている。`build.rs:125` の docs.rs 向けダミー定義では `vmaf_version()` が `std::ptr::null()` を返す。

### なぜ build.rs 側ではなく lib.rs 側で修正するか

`build.rs:125` のダミー定義を有効な文字列（例: `"vmaf-rs (docs.rs)\0"` のポインタ）に修正すれば docs.rs 経路の UB は回避できる。しかし以下の理由で lib.rs 側に防護壁を入れる方が堅牢である:

- prebuilt / source-build 経路でも、libvmaf 実装上のバグ等で `vmaf_version()` が NULL を返す可能性は否定できない
- AGENTS.md の「性能より堅牢性を優先すること」に従い、全 FFI 経路での防御が望ましい

## 設計方針

`CStr::from_ptr` を呼ぶ前に戻り値が NULL でないことを `assert!` で確認する。

- `assert!` を選択する理由: `version()` のシグネチャは `&'static str` であり、エラーを Result で返せない。NULL は libvmaf の正常動作では発生せず、発生した場合は実装バグであるためパニックが適切
- `assert!` メッセージは英語で「vmaf_version() returned null pointer」とする

NULL 以外の無効ポインタ（ダングリングポインタ等）は `CStr::from_ptr` の安全性契約の範囲であり、libvmaf 側の責務として本 issue の範囲外とする。

## 完了条件

- `version()` 内で `CStr::from_ptr` 呼び出し前に NULL チェックが入っていること
- `# Panics` ドキュメント（`src/lib.rs:13-16`）に NULL ポインタ時のパニック条件が追記されていること
- 既存の単体テスト `version_文字列が空でない()`（`tests/test_score.rs:45-48`）が修正後も通過すること
- CI の docs-rs ジョブ（`DOCS_RS=1 cargo build --lib`）でパニックメッセージ「vmaf_version() returned null pointer」が確認できること

## 解決方法

`src/lib.rs:17-22` を以下に変更する:

```rust
pub fn version() -> &'static str {
    unsafe {
        let ptr = sys::vmaf_version();
        assert!(!ptr.is_null(), "vmaf_version() returned null pointer");
        CStr::from_ptr(ptr)
            .to_str()
            .expect("vmaf_version() returned invalid UTF-8")
    }
}
```

`src/lib.rs:13-16` の `# Panics` ドキュメントを以下に更新する:

```rust
/// # Panics
///
/// `vmaf_version()` が NULL を返した場合、または不正な UTF-8 を返した場合にパニックする。
/// libvmaf のバージョン文字列は常に ASCII であるため、後者は通常発生しない
```

### テスト

NULL パスは AGENTS.md のモック禁止により自動テスト不可。代替として docs-rs CI ジョブで手動検証する。

既存の単体テスト `version_文字列が空でない()` が正常系の退行テストとして機能し続けることを確認する。
