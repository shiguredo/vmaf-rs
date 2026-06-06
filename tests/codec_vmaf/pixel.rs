use shiguredo_libyuv::{FilterMode, I420Image, I420ImageMut, ImageSize};

use crate::types::{DecodedI420, I420Frame};

/// ストライド付き I420 プレーンを密なバッファに詰める
pub fn pack_plane(data: &[u8], width: usize, height: usize, stride: usize) -> Vec<u8> {
    let mut out = vec![0u8; width * height];
    for row in 0..height {
        out[row * width..(row + 1) * width]
            .copy_from_slice(&data[row * stride..row * stride + width]);
    }
    out
}

pub fn decoded_to_i420_frame(decoded: &DecodedI420) -> I420Frame {
    I420Frame {
        y: decoded.y.clone(),
        u: decoded.u.clone(),
        v: decoded.v.clone(),
    }
}

/// I420 フレーム列を libyuv で指定解像度へスケールする
pub fn scale_i420_sequence(
    frames: &[I420Frame],
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> Vec<I420Frame> {
    if src_width == dst_width && src_height == dst_height {
        return frames.to_vec();
    }

    frames
        .iter()
        .map(|frame| scale_i420_frame(frame, src_width, src_height, dst_width, dst_height))
        .collect()
}

fn scale_i420_frame(
    frame: &I420Frame,
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> I420Frame {
    let src_size = ImageSize::new(src_width as usize, src_height as usize);
    let dst_size = ImageSize::new(dst_width as usize, dst_height as usize);
    let src_uv_stride = src_width.div_ceil(2) as usize;
    let src = I420Image {
        y: &frame.y,
        y_stride: src_width as usize,
        u: &frame.u,
        u_stride: src_uv_stride,
        v: &frame.v,
        v_stride: src_uv_stride,
    };

    let dst_uv_stride = dst_width.div_ceil(2) as usize;
    let dst_uv_height = dst_height.div_ceil(2) as usize;
    let mut y = vec![0u8; (dst_width as usize) * (dst_height as usize)];
    let mut u = vec![0u8; dst_uv_stride * dst_uv_height];
    let mut v = vec![0u8; dst_uv_stride * dst_uv_height];
    let mut dst = I420ImageMut {
        y: &mut y,
        y_stride: dst_width as usize,
        u: &mut u,
        u_stride: dst_uv_stride,
        v: &mut v,
        v_stride: dst_uv_stride,
    };

    shiguredo_libyuv::i420_scale(&src, src_size, &mut dst, dst_size, FilterMode::Box)
        .expect("I420 スケールに失敗");

    I420Frame { y, u, v }
}
