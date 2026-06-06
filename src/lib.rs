//! [libvmaf] 映像品質メトリクス VMAF の Rust バインディング
//!
//! [libvmaf]: https://github.com/Netflix/vmaf
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{ffi::CStr, ffi::c_int, mem::MaybeUninit, ptr};

mod sys;

/// リンクされている libvmaf ライブラリのバージョン文字列を返す
///
/// # Panics
///
/// `vmaf_version()` が NULL を返した場合、または不正な UTF-8 を返した場合にパニックする。
/// libvmaf のバージョン文字列は常に ASCII であるため、後者は通常発生しない
pub fn version() -> &'static str {
    unsafe {
        let ptr = sys::vmaf_version();
        assert!(!ptr.is_null(), "vmaf_version() returned null pointer");
        CStr::from_ptr(ptr)
            .to_str()
            .expect("vmaf_version() returned invalid UTF-8")
    }
}

/// ビルド時に参照したリポジトリ URL
pub const BUILD_REPOSITORY: &str = sys::BUILD_METADATA_REPOSITORY;

/// ビルド時に参照したリポジトリのバージョン（タグ）
pub const BUILD_VERSION: &str = sys::BUILD_METADATA_VERSION;

/// libvmaf に組み込まれた VMAF モデル
///
/// libvmaf ヘッダにはモデル一覧 API がないため、Rust 側で定数化している。
/// 根拠: Netflix/vmaf `libvmaf/src/model.c` の `built_in_models` 配列
/// （将来 libvmaf の更新で変更される可能性がある）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinModel {
    /// デフォルトモデル (vmaf_v0.6.1)
    V061,
    /// ブートストラップモデル (vmaf_b_v0.6.3)
    BV063,
    /// NEG モードモデル (vmaf_v0.6.1neg)
    V061Neg,
    /// 4K 向けモデル (vmaf_4k_v0.6.1)
    V4k061,
    /// 4K 向け NEG モードモデル (vmaf_4k_v0.6.1neg)
    V4k061Neg,
}

impl BuiltinModel {
    fn version_str(self) -> &'static str {
        match self {
            Self::V061 => "vmaf_v0.6.1",
            Self::BV063 => "vmaf_b_v0.6.3",
            Self::V061Neg => "vmaf_v0.6.1neg",
            Self::V4k061 => "vmaf_4k_v0.6.1",
            Self::V4k061Neg => "vmaf_4k_v0.6.1neg",
        }
    }
}

/// エラー
///
/// 入力検証エラーと libvmaf FFI 由来エラーを型で区別する。
#[derive(Debug)]
pub enum Error {
    /// クレート側の入力検証エラー
    InvalidInput(&'static str),
    /// libvmaf FFI 由来エラー (負の errno code)
    Ffi {
        /// libvmaf が返した負の errno code
        code: c_int,
        /// エラーを返した C 関数名
        function: &'static str,
    },
}

impl Error {
    fn check(code: c_int, function: &'static str) -> Result<(), Self> {
        if code == 0 {
            Ok(())
        } else {
            assert!(code < 0, "libvmaf returned non-negative error code: {code}");
            Err(Self::Ffi { code, function })
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidInput(msg) => write!(f, "{msg}"),
            Error::Ffi { code, function } => write!(
                f,
                "{function}() failed: {}",
                std::io::Error::from_raw_os_error(-code)
            ),
        }
    }
}

impl std::error::Error for Error {}

/// VMAF コンテキストの設定
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// ログレベル (デフォルト: エラーのみ)
    pub log_level: LogLevel,
    /// 並列スレッド数 (デフォルト: 0 = libvmaf が自動決定)
    pub n_threads: u32,
    /// N フレームごとにスコアを計算する (デフォルト: 1 = 全フレーム)
    pub n_subsample: u32,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Error,
            n_threads: 0,
            n_subsample: 1,
        }
    }
}

/// ログレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// ログ出力なし
    None,
    /// エラーのみ
    Error,
    /// 警告以上
    Warning,
    /// 情報以上
    Info,
    /// デバッグ
    Debug,
}

impl LogLevel {
    fn to_sys(self) -> sys::VmafLogLevel {
        match self {
            Self::None => sys::VmafLogLevel_VMAF_LOG_LEVEL_NONE,
            Self::Error => sys::VmafLogLevel_VMAF_LOG_LEVEL_ERROR,
            Self::Warning => sys::VmafLogLevel_VMAF_LOG_LEVEL_WARNING,
            Self::Info => sys::VmafLogLevel_VMAF_LOG_LEVEL_INFO,
            Self::Debug => sys::VmafLogLevel_VMAF_LOG_LEVEL_DEBUG,
        }
    }
}

/// VMAF プーリングメソッド
///
/// クリップ全体のスコアを集計する方法を指定する。
/// libvmaf の `VmafPoolingMethod` に対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolingMethod {
    /// 全フレームの最小値
    Min,
    /// 全フレームの最大値
    Max,
    /// 全フレームの算術平均
    Mean,
    /// 全フレームの調和平均
    HarmonicMean,
}

impl PoolingMethod {
    fn to_sys(self) -> sys::VmafPoolingMethod {
        match self {
            Self::Min => sys::VmafPoolingMethod_VMAF_POOL_METHOD_MIN,
            Self::Max => sys::VmafPoolingMethod_VMAF_POOL_METHOD_MAX,
            Self::Mean => sys::VmafPoolingMethod_VMAF_POOL_METHOD_MEAN,
            Self::HarmonicMean => sys::VmafPoolingMethod_VMAF_POOL_METHOD_HARMONIC_MEAN,
        }
    }
}

/// VMAF 計算コンテキスト
///
/// 生ポインタを保持するため `!Send + !Sync` になる。
/// libvmaf のクロススレッド安全性を保証する一次資料が無いため、
/// 保守的に `Send` / `Sync` を実装していない。
/// 利用者は単一スレッドから逐次利用すること。
pub struct Context {
    inner: *mut sys::VmafContext,
}

impl Context {
    /// 設定をもとに VMAF コンテキストを生成する
    pub fn new(config: ContextConfig) -> Result<Self, Error> {
        let cfg = sys::VmafConfiguration {
            log_level: config.log_level.to_sys(),
            n_threads: config.n_threads,
            n_subsample: config.n_subsample,
            cpumask: 0,
            gpumask: 0,
        };

        let mut inner = ptr::null_mut();
        Error::check(unsafe { sys::vmaf_init(&mut inner, cfg) }, "vmaf_init")?;

        Ok(Self { inner })
    }

    /// モデルに必要な feature extractor を登録する
    pub fn use_model(&mut self, model: &Model) -> Result<(), Error> {
        Error::check(
            unsafe { sys::vmaf_use_features_from_model(self.inner, model.inner) },
            "vmaf_use_features_from_model",
        )
    }

    /// 参照 / 劣化フレームのペアを読み込む
    ///
    /// `reference` と `distorted` の両方が `None` の場合、内部バッファをフラッシュする。
    /// フラッシュ後はこれ以上 `read_pictures` を呼び出せない。
    ///
    /// 成功した場合のみ libvmaf が `Picture` の所有権を取得する。エラーを返した場合は
    /// libvmaf 側で unref されないため、渡した `Picture` が drop 時に破棄され、リークを防ぐ。
    pub fn read_pictures(
        &mut self,
        mut reference: Option<Picture>,
        mut distorted: Option<Picture>,
        index: u32,
    ) -> Result<(), Error> {
        // 所有権移譲は FFI 成功後に行う (ここで owned を false にするとエラー時にリークする)
        let ref_ptr = reference
            .as_mut()
            .map(|pic| &mut pic.inner as *mut _)
            .unwrap_or(ptr::null_mut());
        let dist_ptr = distorted
            .as_mut()
            .map(|pic| &mut pic.inner as *mut _)
            .unwrap_or(ptr::null_mut());

        Error::check(
            unsafe { sys::vmaf_read_pictures(self.inner, ref_ptr, dist_ptr, index) },
            "vmaf_read_pictures",
        )?;

        // 成功時は libvmaf が呼び出し元の Picture を unref 済み (構造体は memset 済み) のため、
        // drop 時の無意味な再 unref を避けるべく所有権を放棄する。
        if let Some(pic) = reference.as_mut() {
            pic.owned = false;
        }
        if let Some(pic) = distorted.as_mut() {
            pic.owned = false;
        }
        Ok(())
    }

    /// 指定インデックスの VMAF スコアを取得する
    pub fn score_at_index(&self, model: &Model, index: u32) -> Result<f64, Error> {
        let mut score = 0.0;
        Error::check(
            unsafe { sys::vmaf_score_at_index(self.inner, model.inner, &mut score, index) },
            "vmaf_score_at_index",
        )?;
        Ok(score)
    }

    /// 指定範囲のフレームをプールした VMAF スコアを取得する
    ///
    /// `index_low` と `index_high` はプール対象フレーム範囲（両端 inclusive）。
    /// クリップ全体のスコアを取得するには、読み込んだ最終フレームの index を
    /// `index_high` に渡すこと。
    pub fn score_pooled(
        &self,
        model: &Model,
        method: PoolingMethod,
        index_low: u32,
        index_high: u32,
    ) -> Result<f64, Error> {
        let mut score = 0.0;
        Error::check(
            unsafe {
                sys::vmaf_score_pooled(
                    self.inner,
                    model.inner,
                    method.to_sys(),
                    &mut score,
                    index_low,
                    index_high,
                )
            },
            "vmaf_score_pooled",
        )?;
        Ok(score)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            let ret = unsafe { sys::vmaf_close(self.inner) };
            if ret != 0 {
                tracing::warn!("vmaf_close() failed with error code: {ret}");
            }
        }
    }
}

/// VMAF モデル
///
/// 生ポインタを保持するため `!Send + !Sync` になる。
/// Context と同様の理由で `Send` / `Sync` を実装していない。
pub struct Model {
    inner: *mut sys::VmafModel,
}

impl Model {
    /// 組み込みモデルを読み込む
    pub fn load_builtin(model: BuiltinModel) -> Result<Self, Error> {
        let version = model.version_str();
        let version_cstr =
            std::ffi::CString::new(version).expect("builtin model version must not contain NUL");

        let mut cfg = sys::VmafModelConfig {
            name: ptr::null(),
            flags: sys::VmafModelFlags_VMAF_MODEL_FLAGS_DEFAULT as u64,
        };

        let mut inner = ptr::null_mut();
        Error::check(
            unsafe { sys::vmaf_model_load(&mut inner, &mut cfg, version_cstr.as_ptr()) },
            "vmaf_model_load",
        )?;

        Ok(Self { inner })
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { sys::vmaf_model_destroy(self.inner) };
        }
    }
}

/// 8-bit I420 ピクセルデータを保持する VMAF ピクチャ
///
/// 生ポインタを保持するため `!Send + !Sync` になる。
pub struct Picture {
    inner: sys::VmafPicture,
    /// libvmaf に所有権が移譲された場合は false
    owned: bool,
}

impl Picture {
    /// 8-bit I420 ピクセルデータから `Picture` を生成する
    ///
    /// `y` / `u` / `v` は密なプレーンである必要がある (Y プレーンの stride は width、
    /// U / V プレーンの stride は width / 2)。
    ///
    /// 幅と高さは偶数である必要がある。奇数寸法は I420 (YUV 4:2:0) の Chroma Subsampling で
    /// クロマプレーンの端数が切り捨てられ、入力データの取りこぼし（誤ったスコア）を招くため、
    /// 明示的に拒否する。
    pub fn from_i420(y: &[u8], u: &[u8], v: &[u8], width: u32, height: u32) -> Result<Self, Error> {
        // ゼロ寸法は無効な picture を後段に流し込む前に拒否する。
        // 奇数寸法を拒否するのは、偶数なら div_ceil(n, 2) と n/2 (floor) が一致し、
        // 検証・コピー (copy_plane) ・libvmaf の確保寸法がすべて同一になるため。
        if width == 0 || height == 0 {
            return Err(Error::InvalidInput("width and height must be non-zero"));
        }
        if !width.is_multiple_of(2) || !height.is_multiple_of(2) {
            return Err(Error::InvalidInput(
                "width and height must be even for I420 chroma subsampling",
            ));
        }

        let y_size = (width as usize) * (height as usize);
        let uv_width = width.div_ceil(2) as usize;
        let uv_height = height.div_ceil(2) as usize;
        let uv_size = uv_width * uv_height;

        if y.len() != y_size || u.len() != uv_size || v.len() != uv_size {
            return Err(Error::InvalidInput(
                "plane size does not match width and height",
            ));
        }

        let mut inner = MaybeUninit::<sys::VmafPicture>::zeroed();
        Error::check(
            unsafe {
                sys::vmaf_picture_alloc(
                    inner.as_mut_ptr(),
                    sys::VmafPixelFormat_VMAF_PIX_FMT_YUV420P,
                    8,
                    width,
                    height,
                )
            },
            "vmaf_picture_alloc",
        )?;

        let mut inner = unsafe { inner.assume_init() };

        copy_plane(&mut inner, 0, y);
        copy_plane(&mut inner, 1, u);
        copy_plane(&mut inner, 2, v);

        Ok(Self { inner, owned: true })
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        if self.owned {
            let ret = unsafe { sys::vmaf_picture_unref(&mut self.inner) };
            if ret != 0 {
                tracing::warn!("vmaf_picture_unref() failed with error code: {ret}");
            }
        }
    }
}

/// libvmaf が確保したピクセルバッファへ密なプレーンデータをコピーする
fn copy_plane(pic: &mut sys::VmafPicture, plane: usize, src: &[u8]) {
    let width = pic.w[plane] as usize;
    let height = pic.h[plane] as usize;
    if width == 0 || height == 0 {
        return;
    }

    let expected_len = width * height;
    debug_assert!(
        src.len() >= expected_len,
        "plane {plane} buffer too small: expected at least {expected_len}, got {}",
        src.len()
    );

    let stride = pic.stride[plane] as usize;
    let dst_ptr = pic.data[plane] as *mut u8;
    debug_assert!(!dst_ptr.is_null());

    for row in 0..height {
        let src_row = &src[row * width..(row + 1) * width];
        let dst_row = unsafe { dst_ptr.add(row * stride) };
        unsafe {
            ptr::copy_nonoverlapping(src_row.as_ptr(), dst_row, width);
            if stride > width {
                ptr::write_bytes(dst_row.add(width), 0, stride - width);
            }
        }
    }
}
