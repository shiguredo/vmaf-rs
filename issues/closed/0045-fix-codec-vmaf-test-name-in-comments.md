# tests/codec_vmaf/main.rs のコメント記載テスト名を修正する

- Priority: Medium
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/fix-codec-vmaf-test-name-in-comments
- Polished: 2026-06-06
- Completed: 2026-06-07

## 目的

`tests/codec_vmaf/main.rs` のドキュメントコメント内のテスト名が `--test test_codec_vmaf` と誤って記載されているのを `--test codec_vmaf` に修正する。

## 優先度根拠

- ユーザーがコメントをコピペ実行すると失敗する。
- 影響範囲はコメントのみで修正リスクはゼロ。

## 現状

`main.rs:79-82`, `:91-93`, `:102-104` に `--test test_codec_vmaf` と記載されているが、実際のテストバイナリ名は `codec_vmaf`。

## 設計方針

全該当箇所の `--test test_codec_vmaf` → `--test codec_vmaf` に修正する。

## 解決方法

`tests/codec_vmaf/main.rs` 内の以下 3 箇所を修正する:

```diff
- --test test_codec_vmaf
+ --test codec_vmaf
```
