use std::collections::HashMap;
use std::path::Path;

use crate::codec::measure_codec_at_bitrate;
use crate::content::ContentKind;
use crate::env::{
    bench_bitrates_for_resolution, bench_bitrates_for_test_resolution, bench_frames_from_env,
    bench_resolutions_from_env, match_max_kbps_for_resolution, match_min_kbps_from_env,
    match_targets_from_env, match_tolerance_from_env, y4m_path_from_env,
};
use crate::pixel::scale_i420_sequence;
use crate::scenario::{print_aom_content_comparison, print_codec_comparison_row};
use crate::types::{
    Codec, HEIGHT, I420Frame, MatchedBitrateResult, RoundtripMetrics, WIDTH, bench_codecs,
};
use crate::y4m::{read_y4m_420_frames, require_y4m_path};

const ENCODER_PROFILE_LABEL: &str = "Realtime encoder profile";

/// 目標 VMAF 探索の kbps 範囲と許容誤差
struct BitrateSearchRange {
    min_kbps: u32,
    max_kbps: u32,
    tolerance: f64,
}

/// 1 コーデック向けの目標 VMAF 探索リクエスト
struct BitrateMatchRequest<'a> {
    codec: Codec,
    width: u32,
    height: u32,
    frames: &'a [I420Frame],
    target_vmaf: f64,
    range: BitrateSearchRange,
}

/// 解像度単位の目標 VMAF 探索実行パラメータ
struct MatchedVmafRun<'a> {
    label: &'a str,
    width: u32,
    height: u32,
    frames: &'a [I420Frame],
    targets: &'a [f64],
    range: BitrateSearchRange,
}

/// ローカル試行用: assert なしでコンテンツ × コーデック × ビットレートの表を出力する
pub fn run_local_bench_report(bitrates_kbps: &[u32]) {
    let width = WIDTH as usize;
    let height = HEIGHT as usize;

    eprintln!();
    eprintln!("=== VMAF local bench ({}x{}) ===", WIDTH, HEIGHT);
    eprintln!("bitrates (kbps): {bitrates_kbps:?}");
    eprintln!("{ENCODER_PROFILE_LABEL}");
    eprintln!();
    eprintln!(
        "{:<28} {:>4} {:>8} {:>8} {:>8} {:>8}",
        "content", "codec", "kbps", "size", "vmaf_avg", "vmaf_min"
    );
    eprintln!("{}", "-".repeat(72));

    for content in ContentKind::ALL {
        let frames = content.frames(width, height);
        for &codec in bench_codecs() {
            for &bitrate_kbps in bitrates_kbps {
                let metric = measure_codec_at_bitrate(codec, WIDTH, HEIGHT, bitrate_kbps, &frames);
                eprintln!(
                    "{:<28} {:>4} {:>8} {:>8} {:>8.2} {:>8.2}",
                    content.label(),
                    codec.label(),
                    bitrate_kbps,
                    metric.encoded_size,
                    metric.avg_vmaf,
                    metric.min_vmaf,
                );
            }
        }
    }

    eprintln!();
    print_aom_content_comparison(width, height, bitrates_kbps);
    eprintln!();
}

/// ローカル Y4M クリップ向け: 同一ビットレートで AOM / VP8 / VP9 を比較する
pub fn run_y4m_bench_report(path: &Path, max_frames: usize, bitrates_kbps: &[u32]) {
    let (width, height, frames) =
        read_y4m_420_frames(path, max_frames).expect("Y4M の読み込みに失敗");

    eprintln!();
    eprintln!(
        "=== VMAF Y4M bench ({width}x{height}, {} frames) ===",
        frames.len()
    );
    eprintln!("source: {}", path.display());
    eprintln!("bitrates (kbps): {bitrates_kbps:?}");
    eprintln!("{ENCODER_PROFILE_LABEL}");
    eprintln!();
    eprintln!(
        "{:>8} {:>4} {:>8} {:>8} {:>8} {:>8}",
        "codec", "kbps", "size", "size/f", "vmaf_avg", "vmaf_min"
    );
    eprintln!("{}", "-".repeat(56));

    let mut metrics = Vec::new();
    for &codec in bench_codecs() {
        for &bitrate_kbps in bitrates_kbps {
            let metric = measure_codec_at_bitrate(codec, width, height, bitrate_kbps, &frames);
            let size_per_frame = metric.encoded_size as f64 / frames.len() as f64;
            eprintln!(
                "{:>8} {bitrate_kbps:>8} {size:>8} {size_per_frame:>8.0} {avg:>8.2} {min:>8.2}",
                codec.label(),
                size = metric.encoded_size,
                avg = metric.avg_vmaf,
                min = metric.min_vmaf,
            );
            metrics.push((codec, bitrate_kbps, metric));
        }
    }

    eprintln!();
    print_codec_comparison_row(bitrates_kbps, &metrics);
    eprintln!();
}

fn measure_codec_at_bitrate_cached(
    cache: &mut HashMap<(Codec, u32), RoundtripMetrics>,
    codec: Codec,
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    frames: &[I420Frame],
) -> RoundtripMetrics {
    let key = (codec, bitrate_kbps);
    if let Some(metrics) = cache.get(&key) {
        return metrics.clone();
    }
    let metrics = measure_codec_at_bitrate(codec, width, height, bitrate_kbps, frames);
    cache.insert(key, metrics.clone());
    metrics
}

/// CBR ターゲット kbps を二分探索し、目標 VMAF に最も近い設定を返す
fn find_bitrate_for_target_vmaf(
    cache: &mut HashMap<(Codec, u32), RoundtripMetrics>,
    request: BitrateMatchRequest<'_>,
) -> MatchedBitrateResult {
    let BitrateMatchRequest {
        codec,
        width,
        height,
        frames,
        target_vmaf,
        range,
    } = request;
    let BitrateSearchRange {
        min_kbps,
        max_kbps,
        tolerance,
    } = range;
    assert!(min_kbps <= max_kbps);

    let at_min = measure_codec_at_bitrate_cached(cache, codec, width, height, min_kbps, frames);
    let at_max = measure_codec_at_bitrate_cached(cache, codec, width, height, max_kbps, frames);

    if at_max.avg_vmaf < target_vmaf - tolerance {
        return MatchedBitrateResult {
            bitrate_kbps: max_kbps,
            metrics: at_max,
            reachable: false,
        };
    }
    if at_min.avg_vmaf >= target_vmaf - tolerance {
        return MatchedBitrateResult {
            bitrate_kbps: min_kbps,
            metrics: at_min,
            reachable: true,
        };
    }

    let mut lo = min_kbps;
    let mut hi = max_kbps;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let metrics = measure_codec_at_bitrate_cached(cache, codec, width, height, mid, frames);
        if metrics.avg_vmaf < target_vmaf {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    let mut best_kbps = lo;
    let mut best_metrics = measure_codec_at_bitrate_cached(cache, codec, width, height, lo, frames);
    let mut best_error = (best_metrics.avg_vmaf - target_vmaf).abs();

    if lo > min_kbps {
        let prev_metrics =
            measure_codec_at_bitrate_cached(cache, codec, width, height, lo - 1, frames);
        let prev_error = (prev_metrics.avg_vmaf - target_vmaf).abs();
        if prev_error < best_error {
            best_kbps = lo - 1;
            best_metrics = prev_metrics;
            best_error = prev_error;
        }
    }

    MatchedBitrateResult {
        bitrate_kbps: best_kbps,
        metrics: best_metrics,
        reachable: best_error <= tolerance,
    }
}

/// 指定フレーム列について目標 VMAF ごとに CBR kbps を探索する
fn run_matched_vmaf_for_frames(run: MatchedVmafRun<'_>) {
    let MatchedVmafRun {
        label,
        width,
        height,
        frames,
        targets,
        range,
    } = run;
    let BitrateSearchRange {
        min_kbps,
        max_kbps,
        tolerance,
    } = range;
    let mut cache = HashMap::new();

    let codecs = bench_codecs();

    eprintln!();
    eprintln!(
        "=== {label} ({width}x{height}, {} frames) ===",
        frames.len()
    );
    eprintln!("search range: {min_kbps}-{max_kbps} kbps, tolerance={tolerance:.2}");

    let header_kbps = codecs
        .iter()
        .map(|codec| format!("{:>8}", format!("{}_kbps", codec.label())))
        .collect::<Vec<_>>()
        .join("");
    let header_vmaf = codecs
        .iter()
        .map(|codec| format!("{:>8}", format!("{}_vmaf", codec.label())))
        .collect::<Vec<_>>()
        .join("");
    eprintln!("{:>7}{header_kbps}{header_vmaf}", "target");
    eprintln!("{}", "-".repeat(7 + header_kbps.len() + header_vmaf.len()));

    for &target_vmaf in targets {
        let results: Vec<_> = codecs
            .iter()
            .map(|&codec| {
                find_bitrate_for_target_vmaf(
                    &mut cache,
                    BitrateMatchRequest {
                        codec,
                        width,
                        height,
                        frames,
                        target_vmaf,
                        range: BitrateSearchRange {
                            min_kbps,
                            max_kbps,
                            tolerance,
                        },
                    },
                )
            })
            .collect();

        let kbps_cols = results
            .iter()
            .map(|result| format!("{:>8}", result.bitrate_kbps))
            .collect::<Vec<_>>()
            .join("");
        let vmaf_cols = results
            .iter()
            .map(|result| format!("{:>8.2}", result.metrics.avg_vmaf))
            .collect::<Vec<_>>()
            .join("");
        eprintln!("{target:>7.1}{kbps_cols}{vmaf_cols}", target = target_vmaf);

        if let Some(aom_index) = codecs.iter().position(|&codec| codec == Codec::Aom) {
            let aom_kbps = results[aom_index].bitrate_kbps as f64;
            let ratios: Vec<String> = codecs
                .iter()
                .zip(results.iter())
                .filter(|(codec, _)| **codec != Codec::Aom)
                .map(|(codec, result)| {
                    format!(
                        "{}={:.2}",
                        codec.label(),
                        result.bitrate_kbps as f64 / aom_kbps
                    )
                })
                .collect();
            eprintln!("  vs AOM kbps ratio: {}", ratios.join(" "));
        }

        for (&codec, result) in codecs.iter().zip(results.iter()) {
            if !result.reachable {
                eprintln!(
                    "  note: {} target {target_vmaf:.1} unreachable in {min_kbps}-{max_kbps} kbps",
                    codec.label()
                );
            }
        }
    }
}

/// 目標 VMAF ごとに各コーデックの CBR kbps を二分探索で求める
pub fn run_y4m_matched_vmaf_report(
    path: &Path,
    max_frames: usize,
    targets: &[f64],
    min_kbps: u32,
    tolerance: f64,
) {
    let (src_width, src_height, src_frames) =
        read_y4m_420_frames(path, max_frames).expect("Y4M の読み込みに失敗");
    let resolutions = bench_resolutions_from_env();

    eprintln!();
    eprintln!("=== VMAF matched bitrate search ===");
    eprintln!("source: {}", path.display());
    eprintln!(
        "source size: {src_width}x{src_height}, frames: {}",
        src_frames.len()
    );
    eprintln!("targets: {targets:?}");
    eprintln!(
        "resolutions: {:?}",
        resolutions.iter().map(|res| res.label).collect::<Vec<_>>()
    );
    eprintln!("{ENCODER_PROFILE_LABEL}");

    for resolution in resolutions {
        let frames = scale_i420_sequence(
            &src_frames,
            src_width,
            src_height,
            resolution.width,
            resolution.height,
        );
        let max_kbps = match_max_kbps_for_resolution(resolution.width, resolution.height);
        run_matched_vmaf_for_frames(MatchedVmafRun {
            label: resolution.label,
            width: resolution.width,
            height: resolution.height,
            frames: &frames,
            targets,
            range: BitrateSearchRange {
                min_kbps,
                max_kbps,
                tolerance,
            },
        });
    }

    eprintln!();
    eprintln!("同一目標 VMAF に必要な CBR ターゲット kbps を解像度 × コーデック別に探索した結果");
    eprintln!();
}

/// 合成コンテンツ向けローカルベンチのエントリポイント
pub fn run_synthetic_bench() {
    run_local_bench_report(&bench_bitrates_for_test_resolution());
}

/// Y4M クリップ向けローカルベンチのエントリポイント
pub fn run_y4m_bench() {
    let path = require_y4m_path(y4m_path_from_env());
    let max_frames = bench_frames_from_env();
    let (width, height, _) = read_y4m_420_frames(&path, 1).expect("Y4M ヘッダの読み込みに失敗");
    let bitrates = bench_bitrates_for_resolution(width, height);
    run_y4m_bench_report(&path, max_frames, &bitrates);
}

/// Y4M クリップ向け目標 VMAF 探索のエントリポイント
pub fn run_y4m_match() {
    let path = require_y4m_path(y4m_path_from_env());
    run_y4m_matched_vmaf_report(
        &path,
        bench_frames_from_env(),
        &match_targets_from_env(),
        match_min_kbps_from_env(),
        match_tolerance_from_env(),
    );
}
