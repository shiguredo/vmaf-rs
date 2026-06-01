# プール済みスコア取得 API (score_pooled) を公開する

- Priority: Medium
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/add-score-pooled-api

## 目的

VMAF の主要ユースケースである「クリップ全体の 1 スコア（mean / harmonic_mean 等）」を取得する API が無く、フレーム単位の `score_at_index` しか公開していない。`vmaf_score_pooled` をラップして公開する。主要ユースケースであり、最小公開を理由に省略しない。

## 優先度根拠

利用者の大半が必要とする全体スコアが取れないのは、最小公開というより主要ユースケースの欠落。Medium。

## 現状

`src/lib.rs:208-216` には `score_at_index` のみがあり、フレーム単位のスコアしか取れない。libvmaf には次の API がある（`/Users/voluntas/src/vmaf/libvmaf/include/libvmaf/libvmaf.h:274-276`、enum は 49-56 行）が Rust 側でラップされていない。

```c
enum VmafPoolingMethod {
    VMAF_POOL_METHOD_UNKNOWN = 0,
    VMAF_POOL_METHOD_MIN,
    VMAF_POOL_METHOD_MAX,
    VMAF_POOL_METHOD_MEAN,
    VMAF_POOL_METHOD_HARMONIC_MEAN,
    VMAF_POOL_METHOD_NB
};

int vmaf_score_pooled(VmafContext *vmaf, VmafModel *model,
                      enum VmafPoolingMethod pool_method, double *score,
                      unsigned index_low, unsigned index_high);
```

## 設計方針

### enum のラップ

`LogLevel`（`src/lib.rs:113-137`）・`BuiltinModel`（`src/lib.rs:36-57`）の既存スタイルに倣い、`Vmaf` プレフィックスを外した公開 enum `PoolingMethod` を定義する。公開するのは実メソッドの **Min / Max / Mean / HarmonicMean の 4 つのみ**。`UNKNOWN`（= 0、無効値）と `NB`（要素数番兵）は公開 enum に含めない。`to_sys()` で `sys::VmafPoolingMethod_VMAF_POOL_METHOD_*`（bindgen 生成シンボル）へ match 変換する。

### メソッド

`Context` に `score_at_index`（`src/lib.rs:208-216`）に倣ったメソッドを追加する。

```rust
pub fn score_pooled(
    &self,
    model: &Model,
    method: PoolingMethod,
    index_low: u32,
    index_high: u32,
) -> Result<f64, Error>
```

`model` は libvmaf の必須引数なので受け取る。`index_low` / `index_high` はプール対象フレーム範囲で両端 inclusive。範囲指定は呼び出し側責任であり、クリップ全体を取るには利用者が読み込んだ最終フレーム index を渡す。これを doc コメントに明記する。

### docs.rs ダミー bindings の更新（必須）

`score_pooled` は `sys::vmaf_score_pooled` と `sys::VmafPoolingMethod_VMAF_POOL_METHOD_*` 定数を参照するため、`build.rs` の docs.rs 向けダミー定義（`build.rs:45-127`）にも次を追加しないと `DOCS_RS=1 cargo build` が壊れる（0003 が扱う乖離問題そのもの）。

```rust
pub type VmafPoolingMethod = u32;
pub const VmafPoolingMethod_VMAF_POOL_METHOD_MIN: u32 = 1;
pub const VmafPoolingMethod_VMAF_POOL_METHOD_MAX: u32 = 2;
pub const VmafPoolingMethod_VMAF_POOL_METHOD_MEAN: u32 = 3;
pub const VmafPoolingMethod_VMAF_POOL_METHOD_HARMONIC_MEAN: u32 = 4;
pub fn vmaf_score_pooled(_vmaf: *mut VmafContext, _model: *mut VmafModel, _pool_method: VmafPoolingMethod, _score: *mut f64, _index_low: u32, _index_high: u32) -> i32 { 0 }
```

0003 が先に入っていればそのダミー方針に従って追加する。

## テスト

`tests/test_score.rs` に単体テストを追加する。既存テストは index 0 の 1 フレームのみ読む（`tests/test_score.rs:63-65`）が、プール検証には `read_pictures` で複数フレームを index を変えて読み込み、`index_low..index_high` をプールする必要がある。Mean でプールした結果が各フレームの `score_at_index` の平均と概ね一致することを確認する（HarmonicMean 等は単純平均と一致しないため pool_method ごとに期待値が変わる点に注意）。実 libvmaf の計算結果検証であり PBT 向きではないため単体テストとする。

## 関連 issue との整合

- 0003（docs.rs ダミー追従）と連動。上記ダミー追加が必要
- 0016（Error 型再設計）で `score_pooled` の戻り値 `Error` がバリアント化される。FFI エラーをそのまま返すだけなので衝突は小さいが、どちらが先でも他方のリベースが要る

## 完了条件

- `PoolingMethod`（Min / Max / Mean / HarmonicMean）と `Context::score_pooled` が公開され、クリップ全体のプール済み VMAF スコアが取得できること
- `build.rs` の docs.rs ダミーに `vmaf_score_pooled` と `VmafPoolingMethod` 定数が追加され、`DOCS_RS=1 cargo build` が成功すること
- `tests/test_score.rs` に複数フレームをプールするテストがあること
- `CHANGES.md` の `## develop` に `[ADD]` エントリを `- @voluntas` 付きで追記すること
