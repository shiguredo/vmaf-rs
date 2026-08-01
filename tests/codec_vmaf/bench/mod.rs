pub mod match_vmaf;
pub mod report;
pub mod search;

use crate::env::{
    bench_bitrates_for_resolution, bench_bitrates_for_test_resolution, bench_frames_from_env,
    match_min_kbps_from_env, match_targets_from_env, match_tolerance_from_env, y4m_path_from_env,
};
use crate::types::{Codec, I420Frame, MatchedBitrateResult};
use crate::y4m::{read_y4m_420_frames, y4m_path_or_skip};

pub(crate) const ENCODER_PROFILE_LABEL: &str = "Realtime encoder profile";

/// 目標 VMAF 探索の kbps 範囲と許容誤差
pub(crate) struct BitrateSearchRange {
    pub(crate) min_kbps: u32,
    pub(crate) max_kbps: u32,
    pub(crate) tolerance: f64,
}

/// 1 コーデック向けの目標 VMAF 探索リクエスト
pub(crate) struct BitrateMatchRequest<'a> {
    pub(crate) codec: Codec,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) frames: &'a [I420Frame],
    pub(crate) target_vmaf: f64,
    pub(crate) range: BitrateSearchRange,
}

/// 解像度単位の目標 VMAF 探索実行パラメータ
pub(crate) struct MatchedVmafRun<'a> {
    pub(crate) label: &'a str,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) frames: &'a [I420Frame],
    pub(crate) targets: &'a [f64],
    pub(crate) range: BitrateSearchRange,
}

/// 合成コンテンツ向けローカルベンチのエントリポイント
pub fn run_synthetic_bench() {
    report::run_local_bench_report(&bench_bitrates_for_test_resolution());
}

/// Y4M クリップ向けローカルベンチのエントリポイント
pub fn run_y4m_bench() {
    let path = y4m_path_from_env();
    if let Err(message) = y4m_path_or_skip(path.clone()) {
        eprintln!("{message}");
        return;
    }
    let max_frames = bench_frames_from_env();
    let (width, height, _) = read_y4m_420_frames(&path, 1).expect("Y4M ヘッダの読み込みに失敗");
    let bitrates = bench_bitrates_for_resolution(width, height);
    report::run_y4m_bench_report(&path, max_frames, &bitrates);
}

/// Y4M クリップ向け目標 VMAF 探索のエントリポイント
pub fn run_y4m_match() {
    let path = y4m_path_from_env();
    if let Err(message) = y4m_path_or_skip(path.clone()) {
        eprintln!("{message}");
        return;
    }
    match_vmaf::run_y4m_matched_vmaf_report(
        &path,
        bench_frames_from_env(),
        &match_targets_from_env(),
        match_min_kbps_from_env(),
        match_tolerance_from_env(),
    );
}
