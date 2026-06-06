# tests/codec_vmaf/ のテストメッセージを日本語に統一する

- Priority: Medium
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/refactor-codec-vmaf-japanese-test-messages
- Polished: 2026-06-06

## 目的

`tests/codec_vmaf/` 配下の全テストファイルで、AGENTS.md:12「テストメッセージは全て日本語にすること」に違反している英語の `.expect()` / `panic!()` メッセージを日本語に修正する。

## 優先度根拠

- AGENTS.md の明示的な規約違反。
- 影響範囲はコメント/メッセージ文字列のみで、機能変更はないためリスクが低い。

## 現状

以下のファイルに英語のテストメッセージが存在する:

| ファイル | 件数 | 例 |
|---|---|---|
| `tests/codec_vmaf/codec.rs` | 3 | `.expect("next_frame")`, `.expect("encoded data")` |
| `tests/codec_vmaf/video_toolbox.rs` | 14 | `.expect("H.264 keyframe ...")`, `panic!("expected I420 ...")` |
| `tests/codec_vmaf/bench.rs` | 3 | `.expect("Y4M read failed")` |
| `tests/codec_vmaf/pixel.rs` | 1 | `.expect("I420 scale failed")` |
| `tests/codec_vmaf/y4m.rs` | 10 | `format!("failed to ...")`, `format!("unsupported ...")` |

## 設計方針

全英語テストメッセージを日本語に置き換える。AGENTS.md の規約に従い、テストメッセージのみを対象とする（エラーメッセージは英語のまま維持）。

## 完了条件

- `tests/codec_vmaf/` 配下の全 `.expect()` / `panic!()` が日本語メッセージであること
- 既存のテストが全て通過すること（機能変更なしのため）

## 解決方法

各メッセージを以下のように修正する例:

| 修正前 | 修正後 |
|---|---|
| `.expect("next_frame")` | `.expect("next_frame に失敗")` |
| `.expect("encoded data")` | `.expect("符号化データの取得に失敗")` |
| `.expect("H.264 keyframe with parameter sets not found")` | `.expect("H.264 のパラメータセット付きキーフレームが見つからない")` |
