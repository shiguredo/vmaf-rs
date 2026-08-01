// リアルタイム符号化設定を前提とした統合テスト / ローカルベンチ。
// libaom-av1 / libvpx-vp8 / libvpx-vp9 / Video Toolbox H.264・H.265 (macOS) を対象とする。

mod bench;
mod codec;
mod content;
mod env;
mod pixel;
mod scenario;
mod types;
#[cfg(target_os = "macos")]
mod video_toolbox;
mod vmaf;
mod y4m;

use codec::{encode_decode_aom, encode_decode_vp9};
use content::{generate_motion_sequence, generate_static_sequence};
use scenario::{assert_monotonic_bitrate_sweep, run_aom_scenario, run_vp9_scenario};
use types::{
    HEIGHT, MOTION_EXPECT, MOTION_FRAME_COUNT, STATIC_EXPECT, WIDTH, high_bitrate_kbps,
    low_bitrate_kbps, mid_bitrate_kbps,
};

#[test]
fn aom_静止画_高品質は符号化サイズが大きく_vmaf_も高い() {
    let frames = generate_static_sequence(1, WIDTH as usize, HEIGHT as usize);
    run_aom_scenario(&frames, &STATIC_EXPECT, "static");
}

#[test]
fn aom_動画_高品質は符号化サイズが大きく_vmaf_も高い() {
    let frames = generate_motion_sequence(MOTION_FRAME_COUNT, WIDTH as usize, HEIGHT as usize);
    run_aom_scenario(&frames, &MOTION_EXPECT, "motion");
}

#[test]
fn vp9_静止画_高品質は符号化サイズが大きく_vmaf_も高い() {
    let frames = generate_static_sequence(1, WIDTH as usize, HEIGHT as usize);
    run_vp9_scenario(&frames, &STATIC_EXPECT, "static");
}

#[test]
fn vp9_動画_高品質は符号化サイズが大きく_vmaf_も高い() {
    let frames = generate_motion_sequence(MOTION_FRAME_COUNT, WIDTH as usize, HEIGHT as usize);
    run_vp9_scenario(&frames, &MOTION_EXPECT, "motion");
}

#[test]
fn リアルタイム符号化_動画_ビットレート上昇で_vmaf_が単調増加し最高ビットレートの符号化サイズが最大になる()
 {
    let frames = generate_motion_sequence(MOTION_FRAME_COUNT, WIDTH as usize, HEIGHT as usize);
    let bitrates = [low_bitrate_kbps(), mid_bitrate_kbps(), high_bitrate_kbps()];

    assert_monotonic_bitrate_sweep("AOM", WIDTH, HEIGHT, &frames, &bitrates, encode_decode_aom);
    assert_monotonic_bitrate_sweep("VP9", WIDTH, HEIGHT, &frames, &bitrates, encode_decode_vp9);
}

#[test]
fn 低品質_動画の符号化サイズは静止画より大きい() {
    let static_frames = generate_static_sequence(1, WIDTH as usize, HEIGHT as usize);
    let motion_frames =
        generate_motion_sequence(MOTION_FRAME_COUNT, WIDTH as usize, HEIGHT as usize);
    let low = low_bitrate_kbps();

    let (static_size, _) = encode_decode_aom(WIDTH, HEIGHT, low, &static_frames);
    let (motion_size, _) = encode_decode_aom(WIDTH, HEIGHT, low, &motion_frames);

    assert!(
        motion_size > static_size,
        "低品質でも動画の方が符号化サイズが大きいはず: motion={motion_size}, static={static_size}"
    );

    eprintln!("[AOM/low/size] static={static_size}, motion={motion_size}");
}

/// ローカル試行用ベンチ。CI では実行しない想定。
///
/// ```bash
/// make codec-bench
/// # または
/// VMAF_BENCH_BITRATES=7,18,34,100 cargo test --features source-build \
///   --test codec_vmaf local_vmaf_ベンチレポート -- --nocapture
/// ```
#[test]
fn local_vmaf_ベンチレポート() {
    bench::run_synthetic_bench();
}

/// ローカル Y4M 試行。`local_videos/` に実クリップを置いて実行する。
///
/// ```bash
/// cargo test --features source-build --test codec_vmaf local_vmaf_y4m_ベンチレポート -- --nocapture
/// VMAF_BENCH_FRAMES=10 VMAF_BENCH_BITRATES=675,1800,3375 cargo test ...
/// ```
#[test]
fn local_vmaf_y4m_ベンチレポート() {
    bench::run_y4m_bench();
}

/// 目標 VMAF ごとに AOM / VP8 / VP9 / H.264 / H.265 (macOS) の CBR kbps を二分探索する (1080p / 720p / 540p)。
///
/// ```bash
/// make codec-match
/// VMAF_MATCH_TARGETS=90 VMAF_BENCH_RESOLUTIONS=1080p,720p,540p cargo test ...
/// ```
#[test]
fn local_vmaf_y4m_同一スコアビットレート探索() {
    bench::run_y4m_match();
}
