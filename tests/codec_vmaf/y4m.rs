use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use crate::types::I420Frame;

/// Y4M (420 のみ) から I420 フレーム列を読み込む
pub fn read_y4m_420_frames(
    path: &Path,
    max_frames: usize,
) -> Result<(u32, u32, Vec<I420Frame>), String> {
    let file = std::fs::File::open(path).map_err(|err| format!("failed to open Y4M: {err}"))?;
    let mut reader = BufReader::new(file);
    let mut header = Vec::new();
    reader
        .read_until(b'\n', &mut header)
        .map_err(|err| format!("failed to read Y4M header: {err}"))?;
    let header = String::from_utf8(header)
        .map_err(|err| format!("Y4M header is not UTF-8: {err}"))?
        .trim()
        .to_string();
    if !header.starts_with("YUV4MPEG2") {
        return Err(format!("unsupported Y4M magic: {header}"));
    }
    if header.contains("C422") || header.contains("C444") {
        return Err(format!("unsupported Y4M chroma format: {header}"));
    }

    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    for token in header.split_whitespace() {
        if let Some(value) = token.strip_prefix('W') {
            width = value.parse().ok();
        }
        if let Some(value) = token.strip_prefix('H') {
            height = value.parse().ok();
        }
    }
    let width = width.ok_or_else(|| format!("Y4M width missing: {header}"))?;
    let height = height.ok_or_else(|| format!("Y4M height missing: {header}"))?;
    if width == 0 || height == 0 {
        return Err(format!("invalid Y4M dimensions: {width}x{height}"));
    }

    let mut marker = Vec::new();
    reader
        .read_until(b'\n', &mut marker)
        .map_err(|err| format!("failed to read first FRAME marker: {err}"))?;
    if marker != b"FRAME\n" {
        return Err(format!(
            "expected FRAME marker, got {:?}",
            String::from_utf8_lossy(&marker)
        ));
    }

    let y_size = (width as usize) * (height as usize);
    let uv_width = width.div_ceil(2) as usize;
    let uv_height = height.div_ceil(2) as usize;
    let uv_size = uv_width * uv_height;
    let frame_bytes = y_size + uv_size * 2;

    let mut frames = Vec::new();
    while frames.len() < max_frames {
        let mut raw = vec![0u8; frame_bytes];
        if let Err(err) = reader.read_exact(&mut raw) {
            if frames.is_empty() {
                return Err(format!("failed to read first Y4M frame: {err}"));
            }
            break;
        }

        let (y_raw, rest) = raw.split_at(y_size);
        let (u_raw, v_raw) = rest.split_at(uv_size);
        frames.push(I420Frame {
            y: y_raw.to_vec(),
            u: u_raw.to_vec(),
            v: v_raw.to_vec(),
        });

        if frames.len() >= max_frames {
            break;
        }

        marker.clear();
        match reader.read_until(b'\n', &mut marker) {
            Ok(0) => break,
            Ok(_) => {}
            Err(err) => return Err(format!("failed to read FRAME marker: {err}")),
        }
        if marker != b"FRAME\n" {
            return Err(format!(
                "expected FRAME marker, got {:?}",
                String::from_utf8_lossy(&marker)
            ));
        }
    }

    if frames.is_empty() {
        return Err("no Y4M frames were read".to_string());
    }

    Ok((width, height, frames))
}

/// Y4M ファイルの存在を確認し、無ければスキップ用のエラーメッセージを返す
///
/// ローカル試行用ベンチのため、ファイルが無い環境でも全テストが通るようにする。
pub fn y4m_path_or_skip(path: PathBuf) -> Result<PathBuf, String> {
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "Y4M ファイルが見つからないためスキップ: {} (VMAF_Y4M_PATH を設定するか local_videos/ に rush_hour を配置してください)",
            path.display()
        ))
    }
}
