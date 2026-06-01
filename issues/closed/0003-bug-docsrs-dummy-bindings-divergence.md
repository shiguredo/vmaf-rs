# docs.rs 向けダミー bindings が実コードと乖離しコンパイル不能

- Priority: High
- Created: 2026-05-29
- Completed: 2026-06-01
- Polished: 2026-05-31
- Model: Opus 4.8

## 目的

`build.rs` の docs.rs 向けダミー bindings が `src/lib.rs` の実際の使用シンボルと乖離しており、本体が型チェックを通らない。CI の docs-rs ジョブは `cargo doc` のみを実行するが、rustdoc は関数本体を型チェックしないため緑になり、ダミーが壊れていることを検知できていない。乖離を解消し、CI で本体コンパイルを担保して再発を防ぐ。

## 現状

`DOCS_RS=1 cargo build --no-default-features` を実行するとコンパイルエラーで失敗する（実行確認済み）。現在のダミー定義（`build.rs` の heredoc）には以下の不足・型不一致・過剰がある。

不足:
- `VmafLogLevel_VMAF_LOG_LEVEL_ERROR` / `_WARNING` / `_INFO` / `_DEBUG`（`src/lib.rs:130-134` が使用、ダミーは `_NONE` のみ）
- `VmafModelFlags_VMAF_MODEL_FLAGS_DEFAULT`（`src/lib.rs:235`）

フィールド無し: `VmafConfiguration` / `VmafModelConfig` / `VmafPicture` が unit struct だが、`src/lib.rs` はフィールドを構築・アクセスする。`VmafPicture` は `copy_plane`（`src/lib.rs`）が `pic.w[plane]` / `pic.h[plane]` / `pic.stride[plane]` / `pic.data[plane]` と配列としてインデックスアクセスするため、フィールドは配列型でなければならない。

型不一致:
- `vmaf_version() -> *const u8` だが `src/lib.rs:19` は `CStr::from_ptr(...)`（`*const c_char` 要求）
- `vmaf_model_load(..., _version: *const u8)` だが `src/lib.rs:240` は `version_cstr.as_ptr()`（`*const c_char`）

過剰: `VmafRef`、未使用 pix_fmt 定数 `_UNKNOWN` / `_YUV422P` / `_YUV444P` / `_YUV400P` は `src/lib.rs` から参照されない。

加えて `build.rs` の `rerun-if-env-changed` に `DOCS_RS` が無い。

## 設計方針

手書きダミーを CI で縛る方針を採用する。API 追加時に手でダミーを追従させる必要が残る構造的限界があるが、CI ゲート（`cargo build`）がその追従漏れを検知する安全網として機能する。

### ダミー定義を実シンボルに追従させる

`src/lib.rs` が参照する型・フィールド・定数・関数に一致させ、過剰定義を削除する。`VmafPicture` / `VmafConfiguration` / `VmafModelConfig` は `vmaf_init` や `vmaf_model_load` で C 関数に値渡し・ポインタ渡しするため、`#[repr(C)]` を付けて C レイアウトを保証する。`VmafContext` / `VmafModel` はポインタ経由でしか使われない opaque 型のため `#[repr(C)]` 不要。

**注意**: フィールド型・順序は lib.rs の使用パターンから推定している。実装時に libvmaf ヘッダ（git clone で取得）を確認し、フィールド型・順序・サイズが一致することを検証する。不一致の場合、`#[repr(C)]` を付けてもレイアウトは一致せず、未定義動作のリスクがある。

```rust
pub struct VmafContext;
pub struct VmafModel;

#[repr(C)]
pub struct VmafPicture {
    pub w: [u32; 3],
    pub h: [u32; 3],
    pub stride: [isize; 3],
    pub data: [*mut std::ffi::c_void; 3],
}

#[repr(C)]
pub struct VmafConfiguration {
    pub log_level: VmafLogLevel,
    pub n_threads: u32,
    pub n_subsample: u32,
    pub cpumask: u64,
    pub gpumask: u64,
}

#[repr(C)]
pub struct VmafModelConfig {
    pub name: *const std::ffi::c_char,
    pub flags: u64,
}

pub type VmafPixelFormat = u32;
pub const VmafPixelFormat_VMAF_PIX_FMT_YUV420P: u32 = 1;

pub type VmafLogLevel = u32;
pub const VmafLogLevel_VMAF_LOG_LEVEL_NONE: u32 = 0;
pub const VmafLogLevel_VMAF_LOG_LEVEL_ERROR: u32 = 1;
pub const VmafLogLevel_VMAF_LOG_LEVEL_WARNING: u32 = 2;
pub const VmafLogLevel_VMAF_LOG_LEVEL_INFO: u32 = 3;
pub const VmafLogLevel_VMAF_LOG_LEVEL_DEBUG: u32 = 4;

pub const VmafModelFlags_VMAF_MODEL_FLAGS_DEFAULT: u32 = 0;

pub fn vmaf_init(_vmaf: *mut *mut VmafContext, _cfg: VmafConfiguration) -> i32 { 0 }
pub fn vmaf_close(_vmaf: *mut VmafContext) -> i32 { 0 }
pub fn vmaf_model_load(_model: *mut *mut VmafModel, _cfg: *mut VmafModelConfig, _version: *const std::ffi::c_char) -> i32 { 0 }
pub fn vmaf_model_destroy(_model: *mut VmafModel) {}
pub fn vmaf_use_features_from_model(_vmaf: *mut VmafContext, _model: *mut VmafModel) -> i32 { 0 }
pub fn vmaf_picture_alloc(_pic: *mut VmafPicture, _pix_fmt: VmafPixelFormat, _bpc: u32, _w: u32, _h: u32) -> i32 { 0 }
pub fn vmaf_picture_unref(_pic: *mut VmafPicture) -> i32 { 0 }
pub fn vmaf_read_pictures(_vmaf: *mut VmafContext, _ref: *mut VmafPicture, _dist: *mut VmafPicture, _index: u32) -> i32 { 0 }
pub fn vmaf_score_at_index(_vmaf: *mut VmafContext, _model: *mut VmafModel, _score: *mut f64, _index: u32) -> i32 { 0 }
pub fn vmaf_version() -> *const std::ffi::c_char { std::ptr::null() }
```

`VmafModelFlags` 型そのものは定義不要。`src/lib.rs` は `sys::VmafModelFlags_VMAF_MODEL_FLAGS_DEFAULT as u64` と定数を直接参照しキャストするため。`VmafRef` は `src/lib.rs` から参照されない過剰定義のため削除する。

### build.rs の再実行トリガー

`build.rs` の `rerun-if-env-changed` に `DOCS_RS` を追加する。

### CI で本体コンパイルを担保

`.github/workflows/ci.yml` の docs-rs ジョブに `cargo build --lib --no-default-features` ステップを追加する。`--lib` を付けることで dev-dependencies やテストコードまで型チェック対象になることを防ぐ。

## 完了条件

1. libvmaf ヘッダを確認し、提案ダミーのフィールド型・順序・サイズが一致することを検証する
2. `DOCS_RS=1 cargo build --lib --no-default-features` が成功すること
3. CI の docs-rs ジョブが `cargo build --lib` でダミーの本体コンパイルを検証していること
4. `CHANGES.md` の `## develop` に `[FIX]` エントリを追記すること

## 解決方法

`build.rs` の docs.rs 向けダミー bindings を `src/lib.rs` の使用シンボルに追従させ、CI で本体コンパイルを担保した。

### ダミー定義の修正（実 bindings 完全一致方針）

不足していた `VmafLogLevel_*`（ERROR / WARNING / INFO / DEBUG）と `VmafModelFlags_VMAF_MODEL_FLAGS_DEFAULT` を追加し、`VmafConfiguration` / `VmafModelConfig` / `VmafPicture` をフィールド付きで定義した。`vmaf_version` / `vmaf_model_load` の `*const u8` を `*const c_char` に修正し、関数を `unsafe fn` 化して呼び出し側の unsafe 要件を実 bindings と一致させた。過剰だった未使用の pix_fmt 定数（`_UNKNOWN` / `_YUV422P` / `_YUV444P` / `_YUV400P`）は削除した。

### 設計判断: issue 提案ダミーからの逸脱

issue の提案ダミーでは `VmafPicture` を 4 フィールド（`w` / `h` / `stride` / `data`）とし `VmafRef` を削除する案だったが、実装では **bindgen が生成する実 bindings に完全一致させ、8 フィールド（`pix_fmt` / `bpc` / `w` / `h` / `stride` / `data` / `ref_` / `priv_`）＋ `VmafRef` opaque を保持**する判断を採った。

理由は以下のとおり:

- issue 本文（「注意」）が「実装時に libvmaf ヘッダを確認し、フィールド型・順序・サイズが一致することを検証する。不一致の場合 `#[repr(C)]` を付けてもレイアウトは一致せず未定義動作のリスクがある」と明記し、提案コードを「lib.rs の使用パターンから推定」した暫定値と位置づけていた。
- 完了条件 1 も「libvmaf ヘッダを確認し、提案ダミーのフィールド型・順序・サイズが一致することを検証する」を要求している。
- 実 bindings を確認した結果、`VmafPicture` は `pix_fmt` / `bpc` が先頭にあり `w` のオフセットは 8 であった。提案 4 フィールド案では `w` のオフセットが 0 となりレイアウトが一致しないため、完了条件 1 を満たさない。8 フィールド完全一致にすると `ref_: *mut VmafRef` のため `VmafRef` の型定義が必要になる。

なお docs.rs ビルドは libvmaf にリンクせず関数本体も実行しないため、レイアウト不一致による未定義動作は docs.rs 経路では発生しないが、完了条件 1 の一致検証要求と保守時の乖離防止のため、実 bindings 完全一致を採用した。`VmafContext` / `VmafModel` / `VmafRef` は実 bindings と同じ `#[repr(C)] struct { _unused: [u8; 0] }` の opaque 表現に統一した。

### build.rs の再実行トリガー

`rerun-if-env-changed` に `DOCS_RS` を追加した。

### CI で本体コンパイルを担保

`.github/workflows/ci.yml` の docs-rs ジョブに `cargo build --lib --no-default-features` ステップを `cargo doc` の前に追加した。rustdoc は関数本体を型チェックしないため、このステップでダミーと `src/lib.rs` の型整合の乖離を検知する。

ローカルで `DOCS_RS=1 cargo build --lib --no-default-features` と `cargo doc --no-deps --no-default-features` がいずれも警告なしで成功すること、`actionlint .github/workflows/ci.yml` が通ることを確認した。
