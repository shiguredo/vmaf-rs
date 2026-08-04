# from_i420 が奇数寸法で検証 (ceil) とコピー (floor) が食い違いデータを取りこぼす

- Priority: High
- Created: 2026-05-29
- Completed: 2026-06-01
- Polished: 2026-05-31
- Model: Opus 4.8

## 目的

奇数幅・奇数高の I420 入力で、サイズ検証とプレーンコピーが別々の寸法定義に依存しており、入力データが取りこぼされて誤ったスコアを無音で返す問題を修正する。

## 優先度根拠

誤ったスコアを無音で返す正しさのバグであり、バインディングの中核 API で発生する。High。

## 現状

`from_i420` の検証は ceil で U/V サイズを計算する。

```rust
let uv_width = width.div_ceil(2) as usize;
let uv_height = height.div_ceil(2) as usize;
```

一方 libvmaf の `vmaf_picture_alloc`（`picture.c`）はクロマ寸法を floor（右シフト `w >> 1`, `h >> 1`）で計算する。`copy_plane`（`src/lib.rs`）は `pic.w[plane]` / `pic.h[plane]`（= floor）を使ってコピーする。

### 再現手順

width=3, height=4 のとき、検証は `uv_width=2, uv_height=2`（`u.len() == 4` を要求）だが、libvmaf は `w[1]=1, h[1]=2` となり、`copy_plane` は 1×2=2 バイトしか転送しない。渡した U/V データの右端列が無視される。height が奇数の場合は下端行が同様に無視される。

```rust
let width = 3u32;
let height = 4u32;
let y = vec![0u8; (width * height) as usize];
let u = vec![128u8; 4]; // ceil サイズ
let v = vec![128u8; 4];
let pic = Picture::from_i420(&y, &u, &v, width, height).unwrap();
// libvmaf は floor (1x2) で確保し、copy_plane は 2 バイトしかコピーしない
// U/V データの右端列（2 バイト）が無視される
// read_pictures → score_at_index まで実行すると、データ損失によりスコアが誤る
```

### 根拠資料

libvmaf の `vmaf_picture_alloc`（`picture.c`）がクロマ寸法を floor で計算する根拠は、I420 (YUV 4:2:0) の Chroma Subsampling 定義である。4:2:0 では水平・垂直ともに 1/2 にサブサンプルするため、奇数寸法の端数は切り捨てられる。libvmaf の `w >> 1`（右シフト = floor）はこの定義と一致する。

## 設計方針

### 代替案の検討

1. **検証を floor に変更する**: `div_ceil` を `/ 2` に変更すれば、奇数寸法を制約なしで受け入れかつ libvmaf の確保寸法と一致する。しかし、利用者が ceil サイズのバッファを渡した場合、末尾バイトが無視される（データ損失）。これは現在のバグと同じ構造であるため、却下する。

2. **libvmaf を ceil で確保するよう修正する（upstream patch）**: libvmaf 側を修正すれば根本的に解決するが、upstream への提案が必要であり、本バインディングのスコープ外である。

3. **ceil で検証して ceil でコピーする**: `copy_plane` を ceil サイズでコピーするよう修正すれば、データ損失を防げる。しかし、libvmaf が floor で確保するバッファに ceil サイズのデータを書き込むため、バッファオーバーフローのリスクがある。

### 採用する設計

`from_i420` の入力検証で、`width` または `height` が奇数なら入力検証エラーを返す。偶数寸法のみを受理すれば `div_ceil(n, 2)` と `n / 2`（floor）は一致するため、検証・コピー・libvmaf の確保寸法がすべて同一になり、食い違いは解消する。

具体的には、`from_i420` のバリデーションに以下のガードを追加する:

```rust
if width % 2 != 0 || height % 2 != 0 {
    return Err(Error {
        code: -22, // EINVAL
        function: "Picture::from_i420",
    });
}
```

エラー表現は現状を維持する。呼び出し側は「バッファサイズ不一致」と「奇数寸法制約」を区別できないが、0016（Error 型再設計）で対応する。`copy_plane` の `debug_assert!` は本 issue のスコープ外とする（リリースビルドでは無効のため検証手段にならない）。

### ドキュメント更新

`from_i420` の doc comment（`src/lib.rs:266`）に偶数寸法制約を明記する。「`y` / `u` / `v` は密なプレーン (stride = width / width/2) である必要がある」という既存記述に加え、「幅と高さは偶数である必要がある。奇数寸法は I420 の Chroma Subsampling でデータ損失が発生するため、明示的に拒否する」旨を追記する。

## 後方互換性

奇数寸法を渡していた呼び出しは従来（誤ったスコアを返しつつ）成功していたが、本変更後はエラーになる。後方互換のない変更のため、ブランチ prefix は `feature/change-`、CHANGES.md の種別は `[CHANGE]` とする。

## テスト戦略

奇数寸法は境界値であり、AGENTS.md のテスト役割分担で単体テストに該当する。`tests/test_score.rs` に単体テストを追加する。

- 奇数幅（例: 3×4）が `from_i420` でエラーになること
- 奇数高（例: 4×3）が `from_i420` でエラーになること
- 両方奇数（例: 3×3）が `from_i420` でエラーになること
- 偶数寸法（例: 2×2）が従来どおり成功すること（回帰確認）

既存テストは 192×108・64×64 の偶数寸法のみを使用しており、本変更の影響を受けない。

## 完了条件

- `width` または `height` が奇数の入力が `from_i420` で明示的に拒否されること
- 偶数寸法の入力は従来どおり受理されること
- 上記の境界値単体テストが追加されていること
- 既存テストが影響を受けないこと
- `from_i420` の doc comment に偶数寸法制約が明記されていること
- `CHANGES.md` の `## develop` に `[CHANGE]` エントリを追記すること

## 解決方法

`src/lib.rs` の `Picture::from_i420` の先頭に偶数寸法ガードを追加した。

- `!width.is_multiple_of(2) || !height.is_multiple_of(2)` のとき `EINVAL` (code: -22) を返す。偶数寸法のみを受理すれば `div_ceil(n, 2)` と `n / 2` (floor) が一致し、検証・`copy_plane` のコピー・libvmaf の `vmaf_picture_alloc` の確保寸法がすべて同一になり、奇数寸法での取りこぼし（誤ったスコア）が解消する。
- ガードは既存のサイズ検証より前に置き、`vmaf_picture_alloc` 到達前に短絡する。
- doc コメントに「幅と高さは偶数である必要がある」旨と、奇数寸法を拒否する理由（Chroma Subsampling で端数が切り捨てられデータ損失が生じる）を明記した。あわせて既存の stride 記述（Y は width、U / V は width / 2）を明確化した。

テストは `tests/test_score.rs` に 4 本追加した。

- `from_i420_は奇数幅を拒否する`（3x4）、`from_i420_は奇数高を拒否する`（4x3）、`from_i420_は幅高とも奇数を拒否する`（3x3）: いずれも U/V を ceil クロマサイズ（4）で用意しており、ガードが無ければサイズ検証を通過してしまう構成にすることで、寸法ガードが効いていること自体を検証する。
- `from_i420_は偶数寸法を受理する`（2x2）: 偶数寸法が従来どおり受理される回帰確認。

既存テスト（偶数 192x108 / 64x64）は本変更の影響を受けないことを確認した。

## 後方互換性

奇数寸法を渡していた呼び出しは従来（誤ったスコアを返しつつ）成功していたが、本変更後は `EINVAL` エラーになる。後方互換のない変更のため、ブランチ prefix は `feature/change-`、`CHANGES.md` の種別は `[CHANGE]` とした。
