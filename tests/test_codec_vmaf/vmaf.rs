use shiguredo_vmaf::{BuiltinModel, Context, ContextConfig, Model, Picture};

use crate::pixel::decoded_to_i420_frame;
use crate::types::{DecodedI420, I420Frame, RoundtripMetrics};

pub fn vmaf_scores_i420(
    reference: &[I420Frame],
    distorted: &[I420Frame],
    width: u32,
    height: u32,
) -> Vec<f64> {
    assert_eq!(reference.len(), distorted.len());

    let mut ctx = Context::new(ContextConfig::new()).expect("Context の生成に失敗");
    let model = Model::load_builtin(BuiltinModel::V061).expect("Model の読み込みに失敗");
    ctx.use_model(&model).expect("use_model に失敗");

    for (index, (reference_frame, distorted_frame)) in reference.iter().zip(distorted).enumerate() {
        let ref_pic = Picture::from_i420(
            &reference_frame.y,
            &reference_frame.u,
            &reference_frame.v,
            width,
            height,
        )
        .expect("ref Picture の生成に失敗");
        let dist_pic = Picture::from_i420(
            &distorted_frame.y,
            &distorted_frame.u,
            &distorted_frame.v,
            width,
            height,
        )
        .expect("dist Picture の生成に失敗");

        ctx.read_pictures(Some(ref_pic), Some(dist_pic), index as u32)
            .expect("read_pictures に失敗");
    }
    ctx.read_pictures(None, None, 0).expect("flush に失敗");

    (0..reference.len())
        .map(|index| {
            ctx.score_at_index(&model, index as u32)
                .unwrap_or_else(|_| panic!("score_at_index({index}) に失敗"))
        })
        .collect()
}

pub fn measure_roundtrip(
    width: u32,
    height: u32,
    reference: &[I420Frame],
    encoded_size: usize,
    decoded: &[DecodedI420],
) -> RoundtripMetrics {
    let distorted: Vec<I420Frame> = decoded.iter().map(decoded_to_i420_frame).collect();
    let scores = vmaf_scores_i420(reference, &distorted, width, height);
    let avg_vmaf = scores.iter().sum::<f64>() / scores.len() as f64;
    let min_vmaf = scores.iter().copied().fold(f64::INFINITY, f64::min);

    RoundtripMetrics {
        encoded_size,
        avg_vmaf,
        min_vmaf,
    }
}
