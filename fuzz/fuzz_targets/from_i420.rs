#![no_main]

use libfuzzer_sys::fuzz_target;
use shiguredo_vmaf::Picture;

/// from_i420 のパニック安全性を検証する Fuzz ターゲット。
///
/// 任意の &[u8] 入力を y/u/v スライスと width/height に分割して from_i420 に渡す。
/// width / height には上限（4096）を設け、libvmaf の実メモリ確保による OOM を防ぐ。
/// width / height が 0 になることは許容する（上限付き剰余の自然な結果であり、
/// ゼロ寸法での libvmaf の挙動も fuzz の探索範囲として有意義なため）。
/// nightly 専用のため stable CI では実行されない。手動実行:
///   cargo +nightly fuzz run from_i420
///
/// y/u/v の分割方法: 先頭 8 バイトから width/height/y_len/u_len を抽出し、
/// 残りを y_len 分 → u_len 分 → 残りすべてを v の順で消費する。
/// y/u の切り出しが大きいと v が小さくなるが、入力空間の多様性は fuzzer の
/// mutation がカバーするため問題ない。
fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    let w = u32::from_le_bytes([data[0], data[1], 0, 0]) % 4096;
    let h = u32::from_le_bytes([data[2], data[3], 0, 0]) % 4096;

    let y_len = u32::from_le_bytes([data[4], data[5], 0, 0]) as usize % 4096;
    let u_len = u32::from_le_bytes([data[6], data[7], 0, 0]) as usize % 4096;

    let rest = &data[8..];
    let y = &rest.get(..y_len.min(rest.len())).unwrap_or(&[]);
    let after_y = &rest.get(y.len()..).unwrap_or(&[]);
    let u = &after_y.get(..u_len.min(after_y.len())).unwrap_or(&[]);
    let after_u = &after_y.get(u.len()..).unwrap_or(&[]);
    let v = after_u;

    let _ = Picture::from_i420(y, u, v, w, h);
});
