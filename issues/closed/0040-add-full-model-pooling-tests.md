# BuiltinModel 全バリアントと PoolingMethod 全バリアントのテストを追加する

- Priority: High
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/add-full-model-pooling-tests
- Polished: 2026-06-06
- Completed: 2026-06-07

## 目的

公開 API `BuiltinModel`（5 バリアント）と `PoolingMethod`（4 バリアント）の全バリアントが正しく動作することを確認するテストを追加する。`LogLevel`（5 バリアント）についても `Context::new` が成功することを確認する。

## 優先度根拠

- `Model::load_builtin` は `V061` のみテストされており、`BV063`, `V061Neg`, `V4k061`, `V4k061Neg` の計 4 バリアントが未テスト。libvmaf 組み込みモデルの列挙漏れや FFI 失敗を検出できない。
- `score_pooled` は `Mean` のみテストされており、`Min`, `Max`, `HarmonicMean` の計 3 バリアントが未テスト。`to_sys()` 変換ミスはコンパイル時検出不可。
- `LogLevel` は `Error` のみテストされており、`None`, `Warning`, `Info`, `Debug` が未テスト。

## 設計方針

1. `BuiltinModel`: 全 5 バリアントで `load_builtin` が成功することを確認するパラメータ化テスト。`VersionStr` の値が空でないことも併せて検証する
2. `PoolingMethod`: 複数フレームを読み込んだ後、全 4 メソッドで `score_pooled` を呼び、結果の関係性（Min <= Mean <= Max など）を検証する
3. `LogLevel`: 各レベルで `Context::new` が成功することを確認する

## 完了条件

- `BuiltinModel` の全 5 バリアントに対するテストが存在すること
- `PoolingMethod` の全 4 バリアントに対するテストが存在し、Min <= Mean <= Max が成立すること
- `LogLevel` の全 5 バリアントで `Context::new` が成功するテストが存在すること

## 解決方法

`tests/test_lib.rs`（0039 でのリネーム後）に追加するテスト例:

```rust
#[test]
fn 全組み込みモデルをロードできる() {
    for model in [
        BuiltinModel::V061,
        BuiltinModel::BV063,
        BuiltinModel::V061Neg,
        BuiltinModel::V4k061,
        BuiltinModel::V4k061Neg,
    ] {
        let _ = Model::load_builtin(model).expect("モデルのロードに失敗");
    }
}

#[test]
fn 全ログレベルでコンテキストを生成できる() {
    for level in [LogLevel::None, LogLevel::Error, LogLevel::Warning, LogLevel::Info, LogLevel::Debug] {
        let config = ContextConfig { log_level: level, ..ContextConfig::default() };
        let _ = Context::new(config).expect("Context の生成に失敗");
    }
}

#[test]
fn 複数フレームを全プーリングメソッドで集計できる() {
    // 複数フレームを読み込み、全 PoolingMethod で score_pooled を呼び、
    // Min <= Mean <= Max が成立することを検証する
}
```
