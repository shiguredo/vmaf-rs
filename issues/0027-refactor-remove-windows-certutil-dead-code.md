# build.rs の Windows (certutil) 死にコードを削除する

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/refactor-remove-windows-certutil-dead-code

## 目的

`build.rs` の `compute_sha256` に Windows (certutil) 分岐があるが、Windows はサポートするビルドホストでなく実質的なデッドコード。削除して一貫させる。

## 優先度根拠

実質到達しないデッドコード。Low。

## 現状

`build.rs:255-262`（計算側 certutil 分岐）と `build.rs:276-286`（出力解析の `if cfg!(target_os = "windows")` ブロック）に Windows (certutil) 分岐がある。

`compute_sha256` の分岐は `cfg!(target_os = "macos")` / `cfg!(target_os = "windows")` / else（`build.rs:248, 255`）で、`build.rs` はビルドホスト上でコンパイル・実行されるため、これらの `cfg!` は **ビルドホストの OS** を指す（ターゲットではない）。つまり certutil 分岐は「Windows ビルドホスト」用。

しかし Windows はサポートするビルドホストでない。README 動作要件（`README.md:30-37`）は Ubuntu / macOS のみ、CI / release の matrix も Ubuntu / macOS ランナーのみ。したがって certutil 分岐は実質到達しない。

（厳密には Windows ホストから Linux ターゲットへクロスコンパイルし `VMAF_TARGET` を指定すれば `get_target_platform` の panic を回避して到達し得るが、これはサポート外の構成。`get_target_platform`（`build.rs:351-365`、panic は 363 行）は CARGO_CFG_TARGET_OS=ターゲットで Windows ターゲットを弾くが、これは certutil のホスト判定とは別軸であり、削除根拠は「Windows がサポートビルドホストでない」点にある。）

## 設計方針

`compute_sha256` を macOS（shasum）/ else（sha256sum）の 2 分岐に簡素化する。

- 計算側: `build.rs:255-262` の `else if cfg!(target_os = "windows")` ブロックを削除し、`if cfg!(target_os = "macos") { shasum } else { sha256sum }` の 2 分岐にする
- 解析側: `build.rs:276-286` の `if cfg!(target_os = "windows")` ブロックを削除し、shasum / sha256sum 共通の出力解析のみ残す

将来 Windows をビルドホスト / ターゲットとしてサポートする場合は、README 動作要件・CI matrix・`compute_sha256` のホスト分岐・`get_target_platform` のターゲット分岐をまとめて対応する。

## CHANGES.md

デッドコード削除で公開 API・配布物の挙動に影響しないため、0007 の方針に従い CHANGES.md への単独記載は不要とする（記載するなら `### misc`）。

## 関連 issue との整合

0010（pending、prebuilt サプライチェーン強化）が同じ `compute_sha256` / `verify_sha256` を触る。0010 は「`compute_sha256` 自体は残る想定なら 0027 のデッドコード削除は独立して成立する」と明記済み。0010 は pending（着手保留）のため、本 issue を先行させてよい。完全性検証の方式（pin / 署名）とは独立。

## 完了条件

- `compute_sha256` の計算側・解析側がともに macOS / else の 2 分岐に簡素化され、Windows (certutil) 分岐が削除されていること
- macOS / Linux で prebuilt 経路の `verify_sha256` → `compute_sha256` が動作すること（`cargo build`）
- `cargo clippy` が警告なく通ること
