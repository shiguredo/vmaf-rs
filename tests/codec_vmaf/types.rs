//! 統合テスト / ローカルベンチ共通の型と定数

pub const WIDTH: u32 = 192;
pub const HEIGHT: u32 = 108;
pub const MOTION_FRAME_COUNT: usize = 5;

/// 1280x720 を基準としたビットレート面積比スケールの参照解像度
pub const BENCH_REF_WIDTH: u32 = 1280;
pub const BENCH_REF_HEIGHT: u32 = 720;

/// ベンチ対象解像度
#[derive(Clone, Copy)]
pub struct BenchResolution {
    pub label: &'static str,
    pub width: u32,
    pub height: u32,
}

pub const BENCH_RESOLUTIONS: [BenchResolution; 3] = [
    BenchResolution {
        label: "1080p",
        width: 1920,
        height: 1080,
    },
    BenchResolution {
        label: "720p",
        width: 1280,
        height: 720,
    },
    BenchResolution {
        label: "540p",
        width: 960,
        height: 540,
    },
];

/// ベンチ対象コーデック
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Codec {
    Aom,
    Vp8,
    Vp9,
    /// Video Toolbox H.264 (macOS のみ)
    #[cfg(target_os = "macos")]
    H264,
    /// Video Toolbox H.265 / HEVC (macOS のみ)
    #[cfg(target_os = "macos")]
    Hevc,
}

impl Codec {
    pub fn label(self) -> &'static str {
        match self {
            Self::Aom => "AOM",
            Self::Vp8 => "VP8",
            Self::Vp9 => "VP9",
            #[cfg(target_os = "macos")]
            Self::H264 => "H.264",
            #[cfg(target_os = "macos")]
            Self::Hevc => "H.265",
        }
    }
}

/// ベンチ対象コーデック一覧 (macOS では H.264 / H.265 を含む)
pub fn bench_codecs() -> &'static [Codec] {
    #[cfg(target_os = "macos")]
    {
        &[Codec::Aom, Codec::Vp8, Codec::Vp9, Codec::H264, Codec::Hevc]
    }
    #[cfg(not(target_os = "macos"))]
    {
        &[Codec::Aom, Codec::Vp8, Codec::Vp9]
    }
}

/// I420 1 フレーム分
#[derive(Clone)]
pub struct I420Frame {
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
}

/// デコード後 I420 1 フレーム分
pub type DecodedI420 = I420Frame;

/// ラウンドトリップ計測結果
#[derive(Clone)]
pub struct RoundtripMetrics {
    pub encoded_size: usize,
    pub avg_vmaf: f64,
    pub min_vmaf: f64,
}

/// コンテンツ種別ごとの VMAF / サイズ期待値
///
/// 閾値は libvmaf v3.2.0 での実測値を基にマージンを取っている。
/// 実測 (192x108, AOM/VP9 リアルタイム CBR):
///   - AOM/static:  hi=95.36, lo=85.09 (delta=10.27, ratio=1.66)
///   - AOM/motion:  hi=88.48, lo=72.25 (delta=16.23, ratio=1.15)
///   - VP9/static:  hi=97.89, lo=94.64 (delta=3.24,  ratio=1.32)
///   - VP9/motion:  hi=91.92, lo=80.79 (delta=11.13, ratio=1.29)
pub struct QualityExpectation {
    pub min_hi_vmaf: f64,
    pub max_lo_vmaf: f64,
    pub min_vmaf_delta: f64,
    pub min_size_ratio: f64,
}

pub const STATIC_EXPECT: QualityExpectation = QualityExpectation {
    min_hi_vmaf: 95.0,
    max_lo_vmaf: 98.0,
    min_vmaf_delta: 0.5,
    min_size_ratio: 1.05,
};

pub const MOTION_EXPECT: QualityExpectation = QualityExpectation {
    min_hi_vmaf: 87.0,
    max_lo_vmaf: 85.0,
    min_vmaf_delta: 1.0,
    min_size_ratio: 1.10,
};

/// 1280x720 基準の kbps を指定解像度へ面積比スケールする
pub fn scale_ref_bitrate_kbps(width: u32, height: u32, ref_kbps: u32) -> u32 {
    let num = u64::from(width) * u64::from(height) * u64::from(ref_kbps);
    let den = u64::from(BENCH_REF_WIDTH) * u64::from(BENCH_REF_HEIGHT);
    (num.div_ceil(den).max(1)) as u32
}

fn scale_ref_bitrate_kbps_test(ref_kbps: u32) -> u32 {
    scale_ref_bitrate_kbps(WIDTH, HEIGHT, ref_kbps)
}

/// 高 / 中 / 低ビットレート (1280x720 基準: 1500 / 800 / 300 kbps)
pub fn high_bitrate_kbps() -> u32 {
    scale_ref_bitrate_kbps_test(1500)
}

pub fn mid_bitrate_kbps() -> u32 {
    scale_ref_bitrate_kbps_test(800)
}

pub fn low_bitrate_kbps() -> u32 {
    scale_ref_bitrate_kbps_test(300)
}

/// 目標 VMAF に対するビットレート探索結果
pub struct MatchedBitrateResult {
    pub bitrate_kbps: u32,
    pub metrics: RoundtripMetrics,
    /// 探索範囲内で目標に到達できたか
    pub reachable: bool,
}
