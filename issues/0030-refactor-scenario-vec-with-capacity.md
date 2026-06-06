# scenario.rs の Vec::with_capacity を Vec::new にする

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-06-06
- Model: Opus 4.8
- Branch: feature/refactor-scenario-vec-with-capacity

## 目的

`tests/test_codec_vmaf/scenario.rs` で性能根拠なく `Vec::with_capacity` を使っており、`Vec::new` を基本とする AGENTS.md の方針に反する。`Vec::new` に置き換える。

## 優先度根拠

軽微な規約違反。Low。

## 現状

`tests/test_codec_vmaf/scenario.rs:87` の `let mut metrics = Vec::with_capacity(bitrates_kbps.len());`。`Vec::with_capacity` は src / tests 中この 1 箇所のみ。

なお AGENTS.md:122 の「入力バイナリデータをデコードする際には事前割り当てメソッドを使わない」という規則（破損入力でサイズ値が極端に大きくなる OOM リスクが根拠）は、ここには直接当てはまらない。`bitrates_kbps.len()` は既知の設定スライスの長さであり、入力からデコードしたサイズ値ではない。

根拠は AGENTS.md:125「性能がきわめて重要な箇所でもなければ `Vec::with_capacity` と `Vec::new` の性能差は誤差程度。基本的に後者を使えばよい」の一般方針。数件のビットレートを回すベンチループに性能根拠は無い。

## 設計方針

`tests/test_codec_vmaf/scenario.rs:87` の `Vec::with_capacity(bitrates_kbps.len())` を `Vec::new()` に置き換える。

## CHANGES.md

テストコードのみの変更で公開 API・配布物に影響しないため、0007 の方針に従い CHANGES.md への単独記載は不要とする。

## 完了条件

- `tests/test_codec_vmaf/scenario.rs:87` が `Vec::new()` になっていること
- `cargo build --tests`（または該当テストのコンパイル）が通ること
