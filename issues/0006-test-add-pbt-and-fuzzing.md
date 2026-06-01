# PBT (proptest) と Fuzzing (cargo-fuzz) を導入する

- Priority: High
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/add-pbt-and-fuzzing

## 目的

AGENTS.md が中核に据える PBT と Fuzzing が一切存在しない。規約に沿ったテスト基盤を整備し、`from_i420` の入力検証のクラッシュ耐性とプロパティを担保する。

## 優先度根拠

AGENTS.md:120, 138-139, 156, 177 が PBT と Fuzzing を必須としているのに両方ゼロ。0001 / 0002 のような境界バグを取りこぼす直接原因であり、品質基盤として High。

## 現状

- リポジトリは単一クレート構成で `Cargo.toml` に `[workspace]` が無い（`Cargo.toml:1-2`、`name = "shiguredo_vmaf"`）
- `pbt/` も `fuzz/` も存在せず、proptest / cargo-fuzz とも未導入
- `rust-toolchain.toml` は `channel = "stable"` 固定
- PBT 適合の検証ロジックは `from_i420` のサイズ検証（`src/lib.rs:267-278`）。`copy_plane`（`src/lib.rs:313-341`）は private 関数で pbt クレートから直接呼べず、`from_i420` 経由で間接的にカバーする
- y4m パーサ `read_y4m_420_frames`（`tests/test_codec_vmaf/y4m.rs:7`）は `&Path` を取りファイル I/O を行うテストヘルパで、`tests/` 配下の dev コード。cargo-fuzz の `&[u8]` ターゲットからは参照できず、本 issue の fuzz 対象としない

## 設計方針

### クレート構成（workspace 化）

AGENTS.md:156 の `pbt/tests/prop_<module>.rs` と AGENTS.md のカバレッジ手順 `cargo llvm-cov -p {crate} --test prop_<module>` は、pbt が独立クレート（workspace member）であることを含意する。`tests/` ではなくリポジトリ直下の `pbt/tests/` を現クレートは拾わないため、以下を行う。

- ルート `Cargo.toml` に `[workspace]` を追加し、members に `pbt` を登録する
- `pbt/Cargo.toml` を新規作成し、`shiguredo_vmaf` を path 依存、`proptest` を依存に追加する（proptest はルートクレートの dev-dependencies ではなく pbt クレートの依存）
- pbt クレートから `shiguredo_vmaf` の公開 API（`Picture::from_i420`）を参照する
- `fuzz/` は `cargo fuzz init` で生成する。cargo-fuzz は nightly 必須で、`#![no_main]` 等 nightly 専用属性を持つため、stable での `cargo build` / `cargo test` を壊さないよう workspace の `exclude` に入れる（または default-members から外す）。fuzz の実行は `cargo +nightly fuzz run <target>`

### PBT

`pbt/tests/prop_lib.rs`（src/lib.rs に対応）に `from_i420` のプロパティを書く。

- プロパティの規則は 0002（奇数寸法を拒否）・0017（ゼロ寸法を拒否）適用後の最終仕様に合わせる: **width / height が偶数かつ非ゼロで、y/u/v のプレーン長が `width*height` / `(width/2)*(height/2)` に整合するなら `Ok`、それ以外は `Err`**。0006 を 0017 より先に実装する場合、ゼロ拒否部分は 0017 完了時にプロパティへ追加する（依存関係を明記しておく）
- `from_i420` の `Ok` 経路は `vmaf_picture_alloc`（`src/lib.rs:283`、FFI・実 libvmaf）が実メモリを確保するため、strategy で width / height を小さい偶数（例: 2〜256）に制限する。`Err` 経路は検証で alloc 前に return する（`src/lib.rs:273-278`）ため軽量。モックは使わない（AGENTS.md:93,151）ので pbt クレートは libvmaf をリンクできる必要がある
- 「任意入力でパニックしないだけ」のプロパティは書かない（fuzzing の役割。AGENTS.md:177）

### Fuzzing

`fuzz/fuzz_targets/from_i420.rs` を作成し、`from_i420` のパニック安全性を検証する。任意 `&[u8]` を y/u/v と width/height に割り当てるが、width / height には上限（例: 4096）を設けて libvmaf のネイティブ確保による OOM / 低速化を防ぐ。

### CI

追加した PBT を CI で実行する（実行されなければ死蔵する）。0012（CI のテストカバレッジ）と分担が重なるため、本 issue では pbt クレートのテストを CI マトリクスに追加することまでを担当し、prebuilt / 統合テストの CI カバレッジは 0012 に委ねる。fuzz は nightly 必須のため stable の CI ゲートには含めず、手動または別ジョブで実行する方針とする。

## 完了条件

- ルート `Cargo.toml` が workspace 化され、`pbt` が member として登録されていること
- `pbt/` が `pbt/tests/prop_lib.rs` の命名・配置で存在し、`from_i420` の検証プロパティ（0002/0017 の規則準拠）を検証していること
- `fuzz/` が `cargo fuzz` の規約配置で存在し、`from_i420` の fuzz ターゲットが存在すること。stable の `cargo build` / `cargo test` が壊れないこと
- PBT が CI で実行されること
- `CHANGES.md` の `## develop`（`### misc` 該当）に `[ADD]` エントリを `- @voluntas` 付きで追記すること
