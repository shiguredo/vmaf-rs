# from_i420 で width / height = 0 を明示的に弾く

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/change-from-i420-reject-zero-dimension

## 目的

`Picture::from_i420` が width / height = 0 を弾かず、`from_i420(&[], &[], &[], 0, 0)` のような呼び出しが現状 `Ok` を返してしまう。無効なゼロ寸法 picture を後段に流し込む前に、入力段で明示的に拒否する。

## 優先度根拠

ゼロ寸法はクラッシュこそしないが無効な状態を後段に流し込み、`read_pictures` での無駄な失敗や診断困難な挙動を招く。入力段で fail-fast すべき設計上の穴。Medium。（0001 修正前はエラー時リーク経路も誘発し得たが、0001 修正後はリークしないため、リークは主たる根拠ではない。）

## 現状

`src/lib.rs:288-298` で width = 0 の場合 `y_size = 0`、`uv_size = 0` となり、空スライスが長さ検証（`src/lib.rs:293`）を通過する。`vmaf_picture_alloc` は w = 0 を許容し、`copy_plane` は `width == 0 || height == 0` で early return する（`src/lib.rs:336`）。結果として `from_i420` は **ゼロ寸法でも `Ok` を返す**。

この 0 サイズ picture を `read_pictures` に渡しても、libvmaf の `validate_pic_params`（`/Users/voluntas/src/vmaf/libvmaf/src/libvmaf.c:609-638`）は初回フレームで `pic_params.w = ref->w[0]`（= 0）を代入してから比較するため、ゼロ寸法を即座には弾かない。失敗は後段の feature extractor 等で生じ、返るエラーが `-EINVAL` である保証はない。いずれにせよ無効な状態を後段に流すため、入力段での拒否が正しい。

## 後方互換性

ゼロ寸法を渡していた呼び出しは従来 `from_i420` が `Ok` を返していたが、本変更後はエラーになる。`from_i420` の受理入力を狭める後方互換のない変更のため、ブランチ prefix は `feature/change-`、CHANGES.md の種別は `[CHANGE]` とする（0002 と同基準）。

## 設計方針

0002（奇数寸法拒否、High）が先に `from_i420` 冒頭へ奇数拒否ガードを入れる。本 issue はそのガードへゼロ寸法条件を追加する。0 は偶数のため 0002 の奇数チェック（`width % 2 != 0`）では捕捉されず、ゼロ拒否が別途必要。両ガードは `width == 0 || height == 0 || width % 2 != 0 || height % 2 != 0` の 1 ブロックに統合してもよい。

0016（Error 再設計）が本 issue より先に入り `Error::InvalidInput(&'static str)` が導入される（0016 が「ゼロ寸法（0017）の理由を `InvalidInput` メッセージで表す」と明記）。したがってゼロ寸法エラーは `Error::InvalidInput("...")` を直接返す。`-22` 魔法数は経由しない。

対応順は番号順で 0002（奇数拒否）→ 0016（InvalidInput 導入）→ 0017（本 issue）。

## テスト戦略

ゼロ寸法は境界値であり、AGENTS.md のテスト役割分担で単体テストに該当する（PBT 基盤は未整備で、その整備は 0006 のスコープ）。`tests/test_score.rs` に単体テストを追加する。

- `0×N`（width のみ 0）、`N×0`（height のみ 0）、`0×0`（両方 0）がいずれも `from_i420` でエラーになること
- 非ゼロの最小寸法 `2×2`（0002 の偶数必須と両立する最小値）が従来どおり成功すること（回帰確認）

## 完了条件

- width / height = 0 が `from_i420` で `Error::InvalidInput` として明示的に拒否されること
- 上記の境界値・回帰の単体テストが追加されていること
- `CHANGES.md` の `## develop` に `[CHANGE]` エントリを `- @voluntas` 付きで追記すること
