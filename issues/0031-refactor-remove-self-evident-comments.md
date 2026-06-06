# build.rs の自明なコメントを削除する

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-06-06
- Model: Opus 4.8
- Branch: feature/refactor-remove-self-evident-comments

## 目的

`build.rs` に、直後のコードと同義で情報量ゼロの自明なコメントがある。可読性のため削除する。

## 優先度根拠

軽微な可読性の改善。Low。

## 現状

`build.rs` には直後のコードを言い換えただけの自明なコメントがある。

なお issue 当初の「AGENTS.md は不要なコメントを禁じている」は誤り。AGENTS.md にコメント削除を求める規約は無く、コメント関連の規約は :9（コメントは日本語）、:121（資料由来の機能は根拠資料名・節番号を明記）、:137（依存ライブラリには用途を明記）で、むしろ特定のコメントを **要求** している。本 issue は規約違反の是正ではなく可読性の改善である。

### 削除候補（直後のコードと同義の自明なコメント）

- `build.rs:170` `// curl でアーカイブをダウンロード`（直後 `Command::new("curl")`）
- `build.rs:182` `// curl で SHA256 チェックサムをダウンロード`（直後 curl）
- `build.rs:193` `// SHA256 を検証`（直後 `verify_sha256(...)`）
- `build.rs:196` `// tar で展開`（直後 `Command::new("tar")`）
- `build.rs:208` `// ライブラリファイルを OUT_DIR/lib/ にコピー`（直後 `fs::copy`）
- `build.rs:217` `// bindings.rs を OUT_DIR/ にコピー`（直後 `fs::copy`）
- `build.rs:308` `// 依存ライブラリのリポジトリを取得する`（直後 `git_clone_external_lib`）
- `build.rs:335` `// バインディングを生成する`（直後 `bindgen::Builder`）

`build.rs:21`（`// 各種変数やビルドディレクトリのセットアップ`）、`build.rs:26`（`// 各種メタデータを書き込む`）、`build.rs:311`（`// 依存ライブラリをビルドする`）も自明寄りだが、削除可否は実装時に判断する。

### 保持するコメント

意図・根拠・理由を説明するコメントは残す。

- `build.rs:41-42`（docs.rs で clone できないためスキップする理由）
- `build.rs:140`（C++ 標準ライブラリをリンクする理由）
- `build.rs:148`（source-build feature と prebuilt の分岐意図）
- `build.rs:391-394`（shallow clone とアノテーティッドタグの注意）
- `build.rs:277-280`, `build.rs:288`（certutil / shasum 出力形式の説明＝パース根拠）
- 関数の説明コメント、および AGENTS.md:121 / :137 が要求するコメント

## 関連 issue との整合

一部のコメント行は他 issue が削除する。重複・競合を避けるため対応順を擦り合わせる。

- 0027（certutil デッドコード削除）が `build.rs:256`・`build.rs:277-280` を含むブロックを削除する
- 0029（未使用 metadata 定数削除）が `build.rs:26` 付近の metadata 書き出しを削除する
- 0003（docs.rs ダミー追従）が `build.rs:40-57` 付近を編集する

## CHANGES.md

コメントのみの変更で公開 API・配布物の挙動に影響しないため、0007 の方針に従い CHANGES.md への単独記載は不要とする。

## 完了条件

- 上記の自明なコメント（`build.rs:170, 182, 193, 196, 208, 217, 308, 335`）が削除されていること
- 意図・根拠・理由を説明するコメント、および AGENTS.md:121 / :137 が要求するコメントは残っていること
- `cargo build` が通ること
