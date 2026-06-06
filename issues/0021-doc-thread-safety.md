# Context / Model / Picture のスレッド安全性をドキュメント化する

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-06-06
- Model: Opus 4.8
- Branch: feature/fix-thread-safety-doc

## 目的

`Context` / `Model` / `Picture` は生ポインタ保持で自動的に `!Send + !Sync` になるが、その根拠が型 doc コメントに示されていない。`#![warn(missing_docs)]`（`src/lib.rs:4`）配下の公開 API doc コメントに根拠を明記する。

対象は README 等の別管理ドキュメントではなく、コードの一部である公開 API の doc コメント。したがって AGENTS.md「ドキュメントについては考慮しないこと」（レビュー時に別管理ドキュメントを対象外とする規定）には抵触しない。

## 優先度根拠

安全側に倒れているため動作上の問題はない。doc コメントの根拠不足。Low。

## 現状

`src/lib.rs:140-142`（`Context` が `*mut sys::VmafContext`）、`src/lib.rs:228-230`（`Model` が `*mut sys::VmafModel`）、`src/lib.rs:263-267`（`Picture` が `VmafPicture` + `owned`）で生ポインタを保持し、自動的に `!Send + !Sync` になる。`Context` は `n_threads`（`src/lib.rs:90`）で内部スレッドプールを起動し得るが、スレッド安全性の前提が doc コメントに示されていない。AGENTS.md「根拠を明記」に照らし不足。

## 設計方針

3 型（`Context` / `Model` / `Picture`）の既存 doc コメントに、スレッド安全性の前提と根拠を追記する。文言は技術的に正確であること。

- `!Send + !Sync` は生ポインタをフィールドに持つことによる Rust の自動挙動である（`unsafe impl Send` を書けば実装自体は「できる」ので、「できない」とは書かない）
- libvmaf のクロススレッド安全性を保証する一次資料が無いため、**保守的に `Send` / `Sync` を実装しない**。`VmafContext` は opaque なヒープハンドルで、特定 OS スレッドへの束縛（TLS 依存等）は libvmaf ヘッダに記載が無く、単一スレッド専用と断言する根拠も無い
- 利用者は単一スレッドから逐次利用すること

「単一スレッド専用（move も不可）」のような強い断定はせず、上記の保守的根拠ベースで記述する。

## CHANGES.md

公開 API のシグネチャ・挙動を変えない doc コメントの追記のみのため、CHANGES.md への記載は不要とする。

## 完了条件

- `Context` / `Model` / `Picture` の 3 型の doc コメントに、`!Send + !Sync` が生ポインタ由来の自動挙動であること・libvmaf のクロススレッド保証が無いため保守的に未実装であること・単一スレッドから利用すること、が記載されていること
- `cargo doc` が警告なく通ること
