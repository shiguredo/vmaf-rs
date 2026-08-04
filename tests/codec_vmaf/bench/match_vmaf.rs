use std::collections::HashMap;
use std::path::Path;

use crate::bench::search::find_bitrate_for_target_vmaf;
use crate::bench::{
    BitrateMatchRequest, BitrateSearchRange, ENCODER_PROFILE_LABEL, MatchedVmafRun,
};
use crate::env::{bench_resolutions_from_env, match_max_kbps_for_resolution};
use crate::pixel::scale_i420_sequence;
use crate::types::{Codec, bench_codecs};
use crate::y4m::read_y4m_420_frames;

/// 指定フレーム列について目標 VMAF ごとに CBR kbps を探索する
pub fn run_matched_vmaf_for_frames(run: MatchedVmafRun<'_>) {
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
