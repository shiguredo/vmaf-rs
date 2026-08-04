# ContextConfig の new() と Default の冗長を整理する

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-06-06
- Completed: 2026-06-06
- Model: Opus 4.8
- Branch: feature/refactor-context-config-new-default

## 目的

`ContextConfig::new()` と `Default::default()` が完全に同義で冗長。`new()` を削除して `Default` に一本化する。

## 優先度根拠

軽微な可読性の問題。Low。

## 現状

`src/lib.rs:95-110` で `new()`（97-103）が `Default::default()`（106-110 は `new()` を呼ぶ）と同じ値を返す。`ContextConfig` は全フィールド pub（`src/lib.rs:86`）で引数も取らないため、Rust 慣習では `Default` だけで十分で、利用者は `ContextConfig { log_level: ..., ..Default::default() }` で部分上書きできる。

`ContextConfig::new()` は次の 5 箇所から呼ばれている。

- `tests/test_score.rs:54`
- `tests/test_score.rs:84`
- `tests/test_score.rs:112`
- `tests/test_score.rs:138`
- `tests/test_codec_vmaf/vmaf.rs:14`

（`README.md:84` も `ContextConfig::new()` を使っており例が壊れるが、README はドキュメント管理のため本 issue の必須作業からは除外する。例が壊れる点には留意する。）

## 設計方針

`new()` を削除して `Default` に一本化する。clippy の `new_without_default` lint は「`new()` を持つのに `Default` が無い型」を警告するもので、`new()` を削除すれば対象から外れ新たな警告は出ない。

`new()` 削除は public メソッドの削除だが version 2026.0.0 は未公開で外部利用者がおらず、冗長コードの整理であるため refactor とする。削除に伴い上記 5 箇所の呼び出しを `ContextConfig::default()` に更新する。

## CHANGES.md

未公開バージョン内の冗長整理（公開 API の機能には影響しない）のため、`## develop` の `### misc` に記載する（記載する場合の種別は実装時に確定。public メソッド削除なので `[CHANGE]` 寄り）。`- @voluntas` 担当者行を付ける。

## 完了条件

- `ContextConfig::new()` が削除され `Default` に一本化されていること
- 呼び出し 5 箇所（`tests/test_score.rs:54, 84, 112, 138`、`tests/test_codec_vmaf/vmaf.rs:14`）が `ContextConfig::default()` に更新され `cargo test` がビルド・通過すること
- `cargo clippy` が警告なしであること
## 解決方法

- ContextConfig::new() を削除し Default::default() に一本化した
- テスト内の ContextConfig::new() 呼び出しを ContextConfig::default() に更新した
- 変更ファイル: src/lib.rs, tests/test_score.rs, tests/codec_vmaf/vmaf.rs
