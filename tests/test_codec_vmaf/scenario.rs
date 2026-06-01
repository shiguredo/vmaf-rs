use crate::codec::{encode_decode_aom, encode_decode_vp9, measure_codec_at_bitrate};
use crate::types::{
    Codec, DecodedI420, HEIGHT, I420Frame, QualityExpectation, RoundtripMetrics, WIDTH,
    bench_codecs, high_bitrate_kbps, low_bitrate_kbps,
};
use crate::vmaf::measure_roundtrip;

type EncodeDecodeFn = fn(u32, u32, u32, &[I420Frame]) -> (usize, Vec<DecodedI420>);

pub fn assert_quality_ordering(
    codec: &str,
    content: &str,
    hi: &RoundtripMetrics,
    lo: &RoundtripMetrics,
    expect: &QualityExpectation,
) {
    let size_ratio = hi.encoded_size as f64 / lo.encoded_size as f64;
    let vmaf_delta = hi.avg_vmaf - lo.avg_vmaf;

    assert!(
        hi.encoded_size > lo.encoded_size,
        "{codec}/{content}: 高品質の符号化サイズの方が大きいはず: hi={}, lo={}",
        hi.encoded_size,
        lo.encoded_size
    );
    assert!(
        size_ratio >= expect.min_size_ratio,
        "{codec}/{content}: 符号化サイズ比が不足: ratio={size_ratio:.2}, min={}",
        expect.min_size_ratio
    );
    assert!(
        hi.avg_vmaf > lo.avg_vmaf,
        "{codec}/{content}: 高品質 VMAF の方が高いはず: hi={:.2}, lo={:.2}",
        hi.avg_vmaf,
        lo.avg_vmaf
    );
    assert!(
        vmaf_delta >= expect.min_vmaf_delta,
        "{codec}/{content}: VMAF 差が不足: delta={vmaf_delta:.2}, min={}",
        expect.min_vmaf_delta
    );
    assert!(
        hi.avg_vmaf >= expect.min_hi_vmaf,
        "{codec}/{content}: 高品質 VMAF が下限未満: {:.2} < {}",
        hi.avg_vmaf,
        expect.min_hi_vmaf
    );
    assert!(
        lo.avg_vmaf <= expect.max_lo_vmaf,
        "{codec}/{content}: 低品質 VMAF が上限超過: {:.2} > {}",
        lo.avg_vmaf,
        expect.max_lo_vmaf
    );

    eprintln!(
        "[{codec}/{content}] size hi={} lo={} (ratio={size_ratio:.2}) | VMAF hi={:.2} lo={:.2} (delta={vmaf_delta:.2}, min_lo={:.2})",
        hi.encoded_size, lo.encoded_size, hi.avg_vmaf, lo.avg_vmaf, lo.min_vmaf,
    );
}

pub fn run_aom_scenario(frames: &[I420Frame], expect: &QualityExpectation, content: &str) {
    let (size_hi, decoded_hi) = encode_decode_aom(WIDTH, HEIGHT, high_bitrate_kbps(), frames);
    let (size_lo, decoded_lo) = encode_decode_aom(WIDTH, HEIGHT, low_bitrate_kbps(), frames);
    let hi = measure_roundtrip(WIDTH, HEIGHT, frames, size_hi, &decoded_hi);
    let lo = measure_roundtrip(WIDTH, HEIGHT, frames, size_lo, &decoded_lo);
    assert_quality_ordering("AOM", content, &hi, &lo, expect);
}

pub fn run_vp9_scenario(frames: &[I420Frame], expect: &QualityExpectation, content: &str) {
    let (size_hi, decoded_hi) = encode_decode_vp9(WIDTH, HEIGHT, high_bitrate_kbps(), frames);
    let (size_lo, decoded_lo) = encode_decode_vp9(WIDTH, HEIGHT, low_bitrate_kbps(), frames);
    let hi = measure_roundtrip(WIDTH, HEIGHT, frames, size_hi, &decoded_hi);
    let lo = measure_roundtrip(WIDTH, HEIGHT, frames, size_lo, &decoded_lo);
    assert_quality_ordering("VP9", content, &hi, &lo, expect);
}

pub fn assert_monotonic_bitrate_sweep(
    codec: &str,
    width: u32,
    height: u32,
    frames: &[I420Frame],
    bitrates_kbps: &[u32],
    encode: EncodeDecodeFn,
) {
    assert!(bitrates_kbps.len() >= 2);

    let mut metrics = Vec::with_capacity(bitrates_kbps.len());
    for &bitrate_kbps in bitrates_kbps {
        let (size, decoded) = encode(width, height, bitrate_kbps, frames);
        let metric = measure_roundtrip(width, height, frames, size, &decoded);
        eprintln!(
            "[{codec}/sweep] bitrate={bitrate_kbps}kbps size={} vmaf={:.2}",
            metric.encoded_size, metric.avg_vmaf
        );
        metrics.push(metric);
    }

    for window in metrics.windows(2) {
        let lo = &window[0];
        let hi = &window[1];
        assert!(
            hi.avg_vmaf > lo.avg_vmaf,
            "{codec}: ビットレートを上げると VMAF も増えるはず: {:.2} -> {:.2}",
            lo.avg_vmaf,
            hi.avg_vmaf
        );
    }

    let first = &metrics[0];
    let last = metrics.last().expect("metrics not empty");
    assert!(
        last.encoded_size > first.encoded_size,
        "{codec}: 最高ビットレートの符号化サイズは最低より大きいはず: {} -> {}",
        first.encoded_size,
        last.encoded_size
    );
}

/// 同一ビットレートで全コーデックの avg VMAF を横並び表示する
pub fn print_codec_comparison_row(
    bitrates_kbps: &[u32],
    metrics: &[(crate::types::Codec, u32, RoundtripMetrics)],
) {
    eprintln!("--- same bitrate: codec comparison (avg VMAF) ---");
    for &bitrate_kbps in bitrates_kbps {
        let mut parts = Vec::new();
        for &codec in bench_codecs() {
            let metric = metrics
                .iter()
                .find(|(c, kbps, _)| *c == codec && *kbps == bitrate_kbps)
                .map(|(_, _, metric)| metric)
                .expect("metrics missing");
            parts.push(format!("{}={:.2}", codec.label(), metric.avg_vmaf));
        }
        eprintln!("{bitrate_kbps} kbps: {}", parts.join(" | "));
    }
}

/// 同一ビットレートで AOM の avg VMAF をコンテンツ別に表示する
pub fn print_aom_content_comparison(width: usize, height: usize, bitrates_kbps: &[u32]) {
    use crate::content::ContentKind;

    eprintln!("--- same bitrate comparison (AOM, avg VMAF) ---");
    for &bitrate_kbps in bitrates_kbps {
        eprintln!("{bitrate_kbps} kbps:");
        for content in ContentKind::ALL {
            let frames = content.frames(width, height);
            let metric = measure_codec_at_bitrate(Codec::Aom, WIDTH, HEIGHT, bitrate_kbps, &frames);
            eprintln!("  {}: {:.2}", content.label(), metric.avg_vmaf);
        }
    }
}
