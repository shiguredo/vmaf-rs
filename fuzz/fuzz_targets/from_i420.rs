#![no_main]

use libfuzzer_sys::fuzz_target;
use shiguredo_vmaf::Picture;

/// from_i420 のパニック安全性を検証する Fuzz ターゲット。
///
/// 先頭 2 バイトから width / height を生成し (偶数化 + 上限 512 で vmaf_picture_alloc
/// の実メモリ確保による OOM を防ぐ)、残りを y / u / v プレーンへ順に割り当てる。
/// データが不足すればサイズ不一致エラー、十分なら Ok 経路 (vmaf_picture_alloc /
/// copy_plane / ptr::copy_nonoverlapping) に到達する。
/// nightly 専用のため stable CI では実行されない。手動実行:
///   cargo +nightly fuzz run from_i420
fn fuzz(data: &[u8]) {
    if data.len() < 2 {
        return;
    }

    let w = u32::from(data[0]) * 2 % 1024;
    let h = u32::from(data[1]) * 2 % 1024;

    let y_size = (w as usize) * (h as usize);
    let uv_size = ((w / 2) as usize) * ((h / 2) as usize);

    let y = &data[2..];
    let y = &y[..y_size.min(y.len())];
    let after_y = &data[2 + y.len()..];
    let u = &after_y[..uv_size.min(after_y.len())];
    let v = &after_y[u.len()..];

    let _ = Picture::from_i420(y, u, v, w, h);
}

fuzz_target!(|data: &[u8]| fuzz(data));
