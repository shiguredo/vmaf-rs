# read_pictures がエラー時に Picture をリークする

- Priority: High
- Created: 2026-05-29
- Completed: 2026-06-01
- Polished: 2026-05-31
- Model: Opus 4.8

## 目的

`Context::read_pictures` がエラーを返す経路で、確保済みの `VmafPicture` バッファがリークする。FFI の所有権移譲タイミングの誤りを修正し、メモリ安全性を確保する。

対象は非 CUDA ビルド（このバインディングがリンクする構成）。CUDA ビルド（`#ifdef HAVE_CUDA`）は本バインディングの対象外とする。

## 優先度根拠

メモリリークは利用者のプロセスで蓄積し、長時間・多フレームの計測で顕在化する。バインディングの中核 API で発生するため High。

## 現状

`read_pictures` メソッドは、`reference` / `distorted` を `as_mut().map()` するクロージャ内（`src/lib.rs:184`, `src/lib.rs:191`）で `pic.owned = false` を FFI 呼び出しの **前** に無条件で設定している。

`Picture::drop`（`src/lib.rs:304-309`）は `owned == false` の場合 `vmaf_picture_unref` を呼ばない。したがって `vmaf_read_pictures` がエラーを返しても、libvmaf も Rust も unref せず、`vmaf_picture_alloc` で確保したバッファがリークする。

### libvmaf の所有権モデル

`vmaf_read_pictures`（`libvmaf.c`）は、`vmaf->thread_pool` の有無で 2 経路に分岐する。どちらの経路でも「成功時のみ呼び出し元の `ref` / `dist` を unref し、エラー時は unref しない」という所有権モデルは一貫している。

非スレッド経路（`n_threads == 0`、`thread_pool` が NULL）:

- エラー return（いずれも unref せず `return err`）
  - `validate_pic_params` 失敗
  - `check_picture_pool` 失敗
  - feature extractor の extract 失敗
- 成功時の unref: 末尾の `vmaf_picture_unref(ref)` / `vmaf_picture_unref(dist)` のみ

スレッド経路（`n_threads > 0`、`threaded_read_pictures_batch` へ分岐）:

- enqueue 失敗時: 内部複製 `pic_a` / `pic_b` のみ unref し、呼び出し元の `ref` / `dist` は unref しない
- 成功時: `vmaf_picture_unref(ref) | vmaf_picture_unref(dist)` で呼び出し元を unref する
- **部分的成功は発生しない**: `threaded_read_pictures_batch` は enqueue の成否で分岐し、成功時は両方を unref、失敗時は両方を unref しないため、片方だけ所有権が移る状態は存在しない

`ContextConfig::n_threads`（`src/lib.rs:89`）はデフォルト 0 だが、利用者が正の値を設定するとスレッド経路に入る。本修正は両は両経路を対象とする。

`validate_pic_params`、`check_picture_pool`、feature extractor 失敗のいずれも、呼び出し元の `ref` / `dist` を unref せずに `return err` する。これは libvmaf.c のソースコードで確認済みである。

### 再 unref の安全性

`vmaf_picture_unref`（`picture.c`）は冒頭 `if (!pic->ref) return -EINVAL;` で早期 return し、末尾で `memset(pic, 0, ...)` して構造体をゼロ化する。一度 unref した `VmafPicture` は `ref` フィールドが NULL になるため、再度 unref しても早期 return で弾かれ参照カウントのデクリメントに到達しない。二重解放は起きない。

本修正の設計では、エラー経路で Rust 側の `Drop` が `vmaf_picture_unref` を呼び、成功経路では libvmaf が既に unref 済みのため `Drop` は何もしない。エラー経路でのみ冪等性に依存する。

## 設計方針

`owned` の設定を FFI が成功（`code == 0`）した場合のみに限定する。

具体的には、`as_mut().map()` クロージャ内の `pic.owned = false;`（`src/lib.rs:184`, `src/lib.rs:191`）を **削除** し、ポインタ取得だけを残す。FFI 呼び出し後に成功時だけ `owned = false` を設定する。

クロージャから `owned = false` を削除し忘れると「FFI 前に false、成功時に再度 false」となり修正にならないため、削除は必須である。

成功時は `owned == false` で `Picture::drop` が unref を呼ばず、libvmaf 側が unref 済みのため二重解放しない。エラー時は `owned == true` のまま `Picture::drop` が unref し、リークを防ぐ。両経路で正しく機能する。

あわせて doc コメント（`src/lib.rs:174`）を修正し、成功時のみ所有権が libvmaf に移り、エラー時は `Picture` が drop 時に破棄される旨を明記する。

## テスト戦略

AGENTS.md によりモック / スタブは使えないため、実 libvmaf でエラー経路を発火させる。テストは `tests/test_score.rs` に追記する。

### エラー経路テスト（単体テスト）

`Picture::from_i420` で正しいバッファサイズの `Picture` を生成し、寸法の異なるペアで `read_pictures` を呼ぶ。`from_i420` のバリデーションを通過させるため、バッファサイズは各 Picture の寸法に合わせる。libvmaf の `validate_pic_params`（`libvmaf.c`）がピクチャの幅・高さの一致を検証し、不一致の場合 `-EINVAL` を返すことを前提とする。

### スレッド経路テスト

`ContextConfig { n_threads: 2, .. }` でスレッド経路を発火させ、上記と同じエラー経路テストを実行する。

### デフォルト経路テスト

既存テスト「同一フレームの VMAF スコアは高得点」（`tests/test_score.rs:49`）と「劣化フレームの VMAF スコアは低得点」（`tests/test_score.rs:78`）が `ContextConfig::new()`（`n_threads: 0`）の成功経路をカバーしている。修正後もこれらのテストがパスすることを確認する。

### flush エラー経路

`read_pictures(None, None, index)` の呼び出しではリークは発生しない。この経路は本 issue のスコープ外である。

## 完了条件

- 非スレッド・スレッド両経路のエラー時に `Picture` が unref され、リークしないこと
- 成功経路で二重解放が起きないこと
- エラー経路を発火させ、`read_pictures` が `Err` を返すことを検証する単体テストが追加されていること
- doc コメント（`src/lib.rs:174`）が修正されていること
- `CHANGES.md` の `## develop` に `[FIX]` エントリを追記すること

## 解決方法

`src/lib.rs` の `Context::read_pictures` を修正し、`Picture` の所有権移譲を FFI 成功後に限定した。

- `as_mut().map()` クロージャ内で無条件に実行していた `pic.owned = false;` を削除し、ポインタ取得だけを残した。
- `vmaf_read_pictures` の戻り値を `Error::check(...)?` で判定し、成功（`code == 0`）した場合にのみ `reference` / `distorted` の `owned` を `false` に設定するようにした。これによりエラー時は `owned == true` のまま `Picture::drop` が `vmaf_picture_unref` を呼び、確保済みバッファのリークを防ぐ。成功時は libvmaf が呼び出し元の `Picture` を unref 済み（構造体は memset 済み）のため、`Picture::drop` は何もせず二重 unref も起こさない。
- doc コメント（`read_pictures`）を「成功時のみ libvmaf に所有権が移り、エラー時は `Picture` が drop 時に破棄される」旨に修正した。

テストは `tests/test_score.rs` に 2 本追加した。

- `read_pictures_は寸法不一致でエラーを返す`: 参照 64x64 と劣化 32x32 の寸法不一致で `validate_pic_params` を失敗させ、`read_pictures` が `Err` を返すエラー経路を発火させる。
- `read_pictures_はスレッド設定でも寸法不一致でエラーを返す`: `n_threads: 2` のスレッドプール付き `Context` でも `read_pictures` がエラー時にパニックせず `Err` を返すことを確認する（寸法不一致は `validate_pic_params` で早期に弾かれるため `threaded_read_pictures_batch` には到達しない点をコメントに明記）。

既存の成功経路テスト（同一フレーム高得点 / 劣化フレーム低得点）が修正後もパスすることを確認し、成功経路での二重解放が起きないことを担保した。
