use std::collections::HashMap;

use crate::bench::{BitrateMatchRequest, BitrateSearchRange, MatchedBitrateResult};
use crate::codec::measure_codec_at_bitrate;
use crate::types::{Codec, I420Frame, RoundtripMetrics};

pub fn measure_codec_at_bitrate_cached(
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
pub fn find_bitrate_for_target_vmaf(
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
