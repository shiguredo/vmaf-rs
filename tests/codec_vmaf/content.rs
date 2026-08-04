use crate::types::{I420Frame, MOTION_FRAME_COUNT};

/// 合成コンテンツ種別
#[derive(Clone, Copy)]
pub enum ContentKind {
    Conferencing,
    ConferencingMotion,
    SportsProxy,
}

impl ContentKind {
    pub const ALL: [Self; 3] = [
        Self::Conferencing,
        Self::ConferencingMotion,
        Self::SportsProxy,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Conferencing => "conferencing (static)",
            Self::ConferencingMotion => "conferencing (motion)",
            Self::SportsProxy => "sports-proxy",
        }
    }

    pub fn frames(self, width: usize, height: usize) -> Vec<I420Frame> {
        match self {
            Self::Conferencing => generate_static_sequence(1, width, height),
            Self::ConferencingMotion => generate_motion_sequence(MOTION_FRAME_COUNT, width, height),
            Self::SportsProxy => generate_sports_proxy_sequence(MOTION_FRAME_COUNT, width, height),
        }
    }
}

/// カラーバー上を横移動するマーカーを重ねた I420 フレームを生成する
///
/// 面談系コンテンツ (会議 UI の動き) の簡易代理。
/// CI 向けの低コストな合成シーケンス。
fn generate_moving_marker_i420(width: usize, height: usize, frame_index: usize) -> I420Frame {
    let mut frame = generate_colorbar_i420(width, height);
    let marker_w = (width / 25).max(6);
    let x = (frame_index * 11) % width;

    for row in (height * 2 / 5)..(height * 3 / 5) {
        for dx in 0..marker_w {
            let col = (x + dx) % width;
            frame.y[row * width + col] = 235;
        }
    }

    frame
}

/// SMPTE カラーバー風の I420 フレームを生成する
///
/// 面談系の静止寄りコンテンツの簡易代理。
fn generate_colorbar_i420(width: usize, height: usize) -> I420Frame {
    let bars: [(u8, u8, u8); 7] = [
        (235, 235, 235),
        (235, 235, 16),
        (16, 235, 235),
        (16, 235, 16),
        (235, 16, 235),
        (235, 16, 16),
        (16, 16, 235),
    ];

    let y_size = width * height;
    let uv_width = width.div_ceil(2);
    let uv_height = height.div_ceil(2);
    let uv_size = uv_width * uv_height;

    let mut y_plane = vec![0u8; y_size];
    let mut u_plane = vec![128u8; uv_size];
    let mut v_plane = vec![128u8; uv_size];

    for row in 0..height {
        for col in 0..width {
            let bar_index = col * 7 / width;
            let (r, g, b) = bars[bar_index];
            let rf = r as f64;
            let gf = g as f64;
            let bf = b as f64;
            let yv = (0.257 * rf + 0.504 * gf + 0.098 * bf + 16.0).clamp(16.0, 235.0) as u8;
            y_plane[row * width + col] = yv;

            if row % 2 == 0 && col % 2 == 0 {
                let u = (-0.148 * rf - 0.291 * gf + 0.439 * bf + 128.0).clamp(16.0, 240.0) as u8;
                let v = (0.439 * rf - 0.368 * gf - 0.071 * bf + 128.0).clamp(16.0, 240.0) as u8;
                let uv_row = row / 2;
                let uv_col = col / 2;
                u_plane[uv_row * uv_width + uv_col] = u;
                v_plane[uv_row * uv_width + uv_col] = v;
            }
        }
    }

    I420Frame {
        y: y_plane,
        u: u_plane,
        v: v_plane,
    }
}

pub fn generate_static_sequence(frame_count: usize, width: usize, height: usize) -> Vec<I420Frame> {
    let frame = generate_colorbar_i420(width, height);
    (0..frame_count)
        .map(|_| I420Frame {
            y: frame.y.clone(),
            u: frame.u.clone(),
            v: frame.v.clone(),
        })
        .collect()
}

pub fn generate_motion_sequence(frame_count: usize, width: usize, height: usize) -> Vec<I420Frame> {
    (0..frame_count)
        .map(|i| generate_moving_marker_i420(width, height, i))
        .collect()
}

/// 高周波・大動きの代理コンテンツ (モータースポーツ / ゲーム系の簡易代替)
fn generate_sports_proxy_i420(width: usize, height: usize, frame_index: usize) -> I420Frame {
    let block = 8usize;
    let shift_x = (frame_index * 5) % block;
    let shift_y = (frame_index * 3) % block;
    let uv_width = width.div_ceil(2);
    let uv_height = height.div_ceil(2);

    let mut y = vec![0u8; width * height];
    for row in 0..height {
        for col in 0..width {
            let bx = (col + shift_x) / block;
            let by = (row + shift_y) / block;
            y[row * width + col] = if (bx + by + frame_index).is_multiple_of(2) {
                220
            } else {
                16
            };
        }
    }

    let mut u = vec![128u8; uv_width * uv_height];
    let mut v = vec![128u8; uv_width * uv_height];
    for row in 0..uv_height {
        for col in 0..uv_width {
            let phase = (frame_index + row + col) % 4;
            u[row * uv_width + col] = [32, 96, 160, 224][phase];
            v[row * uv_width + col] = [224, 160, 96, 32][phase];
        }
    }

    I420Frame { y, u, v }
}

fn generate_sports_proxy_sequence(
    frame_count: usize,
    width: usize,
    height: usize,
) -> Vec<I420Frame> {
    (0..frame_count)
        .map(|i| generate_sports_proxy_i420(width, height, i))
        .collect()
}
