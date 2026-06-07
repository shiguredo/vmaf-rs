use std::path::Path;

use crate::bench::ENCODER_PROFILE_LABEL;
use crate::codec::measure_codec_at_bitrate;
use crate::content::ContentKind;
use crate::scenario::{print_aom_content_comparison, print_codec_comparison_row};
use crate::types::{HEIGHT, WIDTH, bench_codecs};
use crate::y4m::read_y4m_420_frames;

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
