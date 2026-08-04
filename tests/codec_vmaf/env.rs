/// 環境変数からベンチ設定を読み込む
use std::path::PathBuf;

use crate::types::{
    BENCH_RESOLUTIONS, BenchResolution, high_bitrate_kbps, low_bitrate_kbps, mid_bitrate_kbps,
    scale_ref_bitrate_kbps,
};

/// カンマ区切り数値一覧を環境変数から読み込む
fn comma_separated<T>(name: &str) -> Option<Vec<T>>
where
    T: std::str::FromStr,
{
    let raw = std::env::var(name).ok()?;
    let values: Vec<T> = raw
        .split(',')
        .filter_map(|part| part.trim().parse().ok())
        .collect();
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn positive<T>(name: &str) -> Option<T>
where
    T: std::str::FromStr + PartialOrd + Default,
{
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.parse().ok())
        .filter(|value| value > &T::default())
}

/// 192x108 向けのビットレート一覧 (`VMAF_BENCH_BITRATES` で上書き可能)
pub fn bench_bitrates_for_test_resolution() -> Vec<u32> {
    comma_separated("VMAF_BENCH_BITRATES")
        .unwrap_or_else(|| vec![low_bitrate_kbps(), mid_bitrate_kbps(), high_bitrate_kbps()])
}

/// 指定解像度向けのビットレート一覧 (`VMAF_BENCH_BITRATES` で上書き可能)
pub fn bench_bitrates_for_resolution(width: u32, height: u32) -> Vec<u32> {
    comma_separated("VMAF_BENCH_BITRATES").unwrap_or_else(|| {
        vec![
            scale_ref_bitrate_kbps(width, height, 300),
            scale_ref_bitrate_kbps(width, height, 800),
            scale_ref_bitrate_kbps(width, height, 1500),
        ]
    })
}

pub fn bench_resolutions_from_env() -> Vec<BenchResolution> {
    if let Ok(raw) = std::env::var("VMAF_BENCH_RESOLUTIONS") {
        let mut resolutions = Vec::new();
        for part in raw.split(',') {
            let token = part.trim();
            if let Some(res) = BENCH_RESOLUTIONS.iter().find(|res| res.label == token) {
                resolutions.push(BenchResolution {
                    label: res.label,
                    width: res.width,
                    height: res.height,
                });
            }
        }
        if !resolutions.is_empty() {
            return resolutions;
        }
    }
    BENCH_RESOLUTIONS.to_vec()
}

pub fn y4m_path_from_env() -> PathBuf {
    if let Ok(path) = std::env::var("VMAF_Y4M_PATH") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("local_videos/natural/rush_hour_1080p25.y4m")
}

pub fn bench_frames_from_env() -> usize {
    positive::<u32>("VMAF_BENCH_FRAMES")
        .map(|count| count as usize)
        .unwrap_or(30)
}

pub fn match_targets_from_env() -> Vec<f64> {
    comma_separated("VMAF_MATCH_TARGETS").unwrap_or_else(|| vec![90.0])
}

pub fn match_min_kbps_from_env() -> u32 {
    positive("VMAF_MATCH_MIN_KBPS").unwrap_or(200)
}

pub fn match_max_kbps_from_env() -> u32 {
    positive("VMAF_MATCH_MAX_KBPS").unwrap_or(6000)
}

pub fn match_tolerance_from_env() -> f64 {
    positive("VMAF_MATCH_TOLERANCE").unwrap_or(0.5)
}

/// 1080p 基準の探索上限 kbps を面積比で解像度に合わせる
pub fn match_max_kbps_for_resolution(width: u32, height: u32) -> u32 {
    if std::env::var("VMAF_MATCH_MAX_KBPS").is_ok() {
        return match_max_kbps_from_env();
    }
    scale_ref_bitrate_kbps(width, height, match_max_kbps_from_env()).max(1500)
}
