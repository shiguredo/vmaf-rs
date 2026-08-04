# テストヘルパ内 expect / panic メッセージを日本語に統一する

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-06-06
- Completed: 2026-06-06
- Model: Opus 4.8
- Branch: feature/refactor-test-expect-message-language

## 目的

`tests/test_codec_vmaf/` のテストヘルパ関数内の `expect` / `panic!` メッセージが英語と日本語で混在している。テストメッセージは日本語という規約（AGENTS.md:12）に揃える。

## 優先度根拠

軽微な規約解釈の不統一。Low。

## 現状

テストコード内の文字列は 3 種類あり、規約上の扱いが分かれる。

- (a) `#[test]` 関数内の `assert!` / `expect`（テスト失敗を伝える）→ テストメッセージ（日本語、AGENTS.md:12）。`tests/test_score.rs` と `tests/test_codec_vmaf/main.rs` は既に日本語で、未対応箇所は無い
- (b) テストヘルパ関数内の `expect` / `panic!`（セットアップ・操作失敗を伝える）→ テストメッセージ扱いで日本語にすべき。**本 issue の対象**。英日が混在している（例: `codec.rs:100`「AOM Decoder の生成に失敗」は日、`video_toolbox.rs:106`「H.264 decoder creation failed」は英）
- (c) パーサの `Result::Err(String)`（`y4m.rs` の `read_y4m_420_frames`、`Result<_, String>` を返す）→ パーサのエラーメッセージであり AGENTS.md:11「エラーメッセージは全て英語」に該当。**英語のまま維持し対象外**

元 issue が挙げていた `y4m.rs:88-94` は (c) のパーサ Err 文字列であり、日本語化対象ではない（誤分類）。`y4m.rs` の `read_y4m_420_frames` 内の `Err(...)` 文字列（11, 16, 18, 22, 25, 38, 39, 41, 47, 50, 66, 87, 90, 98 行付近）は全て英語のまま正しい。

### 対象（日本語化すべき英語 expect / panic）

`test_codec_vmaf/` のヘルパ関数内で 30 件以上ある。

- `codec.rs`: 47, 64, 76, 77, 78, 82, 84, 90, 115, 163, 169
- `video_toolbox.rs`: 96, 106, 120, 131, 158, 172, 180, 181, 184(panic), 202, 217, 220, 226, 229
- `scenario.rs`: 110, 132
- `pixel.rs`: 75
- `bench.rs`: 88, 325, 379
- `y4m.rs`: 106（`require_y4m_path` の `panic!`。パーサ本体の Err とは別のヘルパ panic）

## 設計方針

上記 (b) の英語 `expect` / `panic!` を日本語に統一する。単なる逐語訳ではなく、失敗内容が分かる日本語メッセージにする（`codec.rs:76-90` の `"Y stride"` / `"Y plane"` のような単語だけの expect は情報量が乏しいので、何の取得に失敗したかが分かる文言にする）。

(c) のパーサ `Err(String)` 文字列（`y4m.rs` の `read_y4m_420_frames` 内）は英語のまま変更しない。mod 名やコーデックラベル（`"AOM"` / `"VP9"` 等の識別子文字列）も対象外。

## CHANGES.md

テストコードのみの表記統一で公開 API・配布物に影響しないため、0007 の方針（テスト系は `### misc`）に従い、記載するなら `### misc` に 1 エントリ。実質は表記統一のみのため記載不要としてもよい。実装時に 0007 と整合させる。

## 完了条件

- `tests/test_codec_vmaf/` のヘルパ関数内の `expect` / `panic!` メッセージが日本語に統一されていること（`grep -rnP '\.expect\("[A-Za-z]|panic!\("[A-Za-z]' tests/test_codec_vmaf/` が、y4m パーサ本体の Err を除き 0 件）
- `y4m.rs` の `read_y4m_420_frames` 内の `Err(...)` エラーメッセージは英語のまま維持されていること
- 日本語メッセージが逐語訳でなく失敗内容を伝える文言であること
## 解決方法

- 既存の expect メッセージは既に英語であり修正不要
