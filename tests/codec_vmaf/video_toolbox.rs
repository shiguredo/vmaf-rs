//! Video Toolbox (H.264 / H.265) ラウンドトリップ

use shiguredo_video_toolbox::{
    CodecConfig, DecodedFrame, Decoder, DecoderCodec, DecoderConfig, EncodeOptions, Encoder,
    EncoderConfig, FrameData, H264EncoderConfig, H264EntropyMode, H264Profile, HevcEncoderConfig,
    HevcProfile, I420Frame as VtI420Frame, PixelFormat, VideoCodecType, supported_codecs,
};

use crate::pixel::pack_plane;
use crate::types::{DecodedI420, I420Frame};

const NALU_LEN_BYTES: u32 = 4;
const BENCH_FPS_NUMERATOR: u32 = 30;
const BENCH_FPS_DENOMINATOR: u32 = 1;

fn ensure_encoding_supported(hevc: bool) {
    let codec_type = if hevc {
        VideoCodecType::Hevc
    } else {
        VideoCodecType::H264
    };
    let label = if hevc { "H.265" } else { "H.264" };
    let supported = supported_codecs()
        .iter()
        .any(|info| info.codec == codec_type && info.encoding.supported);
    assert!(
        supported,
        "{label} hardware encoding is not supported on this Mac"
    );
}

/// Video Toolbox リアルタイム符号化設定
fn realtime_encoder_config(
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    hevc: bool,
) -> EncoderConfig {
    let codec = if hevc {
        CodecConfig::Hevc(HevcEncoderConfig {
            profile: HevcProfile::Main,
            allow_open_gop: false,
        })
    } else {
        CodecConfig::H264(H264EncoderConfig {
            profile: H264Profile::Main,
            entropy_mode: H264EntropyMode::Cabac,
        })
    };

    EncoderConfig {
        width,
        height,
        codec,
        pixel_format: PixelFormat::I420,
        average_bitrate: Some(u64::from(bitrate_kbps) * 1000),
        fps_numerator: BENCH_FPS_NUMERATOR,
        fps_denominator: BENCH_FPS_DENOMINATOR,
        prioritize_encoding_speed_over_quality: true,
        real_time: true,
        maximize_power_efficiency: false,
        allow_frame_reordering: false,
        allow_temporal_compression: true,
        max_key_frame_interval: None,
        max_key_frame_interval_duration: None,
        max_frame_delay_count: None,
    }
}

fn vt_i420_to_decoded(frame: &VtI420Frame<'_>) -> DecodedI420 {
    let width = frame.width();
    let height = frame.height();
    DecodedI420 {
        y: pack_plane(frame.y_plane(), width, height, frame.y_stride()),
        u: pack_plane(
            frame.u_plane(),
            width.div_ceil(2),
            height.div_ceil(2),
            frame.u_stride(),
        ),
        v: pack_plane(
            frame.v_plane(),
            width.div_ceil(2),
            height.div_ceil(2),
            frame.v_stride(),
        ),
    }
}

fn decode_h264_packets(packets: &[shiguredo_video_toolbox::EncodedFrame]) -> Vec<DecodedI420> {
    let first_key = packets
        .iter()
        .find(|packet| {
            packet.keyframe && !packet.sps_list.is_empty() && !packet.pps_list.is_empty()
        })
        .expect("H.264 のパラメータセット付きキーフレームが見つからない");

    let mut decoder = Decoder::new(DecoderConfig {
        codec: DecoderCodec::H264 {
            sps: &first_key.sps_list[0],
            pps: &first_key.pps_list[0],
            nalu_len_bytes: NALU_LEN_BYTES,
        },
        pixel_format: PixelFormat::I420,
    })
    .expect("H.264 デコーダの生成に失敗");

    decode_avcc_packets(&mut decoder, packets, DecoderKind::H264)
}

fn decode_hevc_packets(packets: &[shiguredo_video_toolbox::EncodedFrame]) -> Vec<DecodedI420> {
    let first_key = packets
        .iter()
        .find(|packet| {
            packet.keyframe
                && !packet.vps_list.is_empty()
                && !packet.sps_list.is_empty()
                && !packet.pps_list.is_empty()
        })
        .expect("H.265 のパラメータセット付きキーフレームが見つからない");

    let mut decoder = Decoder::new(DecoderConfig {
        codec: DecoderCodec::Hevc {
            vps: &first_key.vps_list[0],
            sps: &first_key.sps_list[0],
            pps: &first_key.pps_list[0],
            nalu_len_bytes: NALU_LEN_BYTES,
        },
        pixel_format: PixelFormat::I420,
    })
    .expect("H.265 デコーダの生成に失敗");

    decode_avcc_packets(&mut decoder, packets, DecoderKind::Hevc)
}

enum DecoderKind {
    H264,
    Hevc,
}

fn decode_avcc_packets(
    decoder: &mut Decoder,
    packets: &[shiguredo_video_toolbox::EncodedFrame],
    kind: DecoderKind,
) -> Vec<DecodedI420> {
    let mut decoded = Vec::new();

    for packet in packets {
        if packet.keyframe {
            match kind {
                DecoderKind::H264 if !packet.sps_list.is_empty() && !packet.pps_list.is_empty() => {
                    decoder
                        .update_format(DecoderCodec::H264 {
                            sps: &packet.sps_list[0],
                            pps: &packet.pps_list[0],
                            nalu_len_bytes: NALU_LEN_BYTES,
                        })
                        .expect("H.264 デコーダのフォーマット更新に失敗");
                }
                DecoderKind::Hevc
                    if !packet.vps_list.is_empty()
                        && !packet.sps_list.is_empty()
                        && !packet.pps_list.is_empty() =>
                {
                    decoder
                        .update_format(DecoderCodec::Hevc {
                            vps: &packet.vps_list[0],
                            sps: &packet.sps_list[0],
                            pps: &packet.pps_list[0],
                            nalu_len_bytes: NALU_LEN_BYTES,
                        })
                        .expect("H.265 デコーダのフォーマット更新に失敗");
                }
                _ => {}
            }
        }

        let frame = decoder
            .decode(&packet.data)
            .expect("Video Toolbox デコードに失敗")
            .expect("Video Toolbox デコードがフレームを返さなかった");
        match frame {
            DecodedFrame::I420(frame) => decoded.push(vt_i420_to_decoded(&frame)),
            DecodedFrame::Nv12(_) => panic!("I420 デコードフレームを期待したが NV12 が返された"),
        }
    }

    decoded
}

fn encode_video_toolbox(
    hevc: bool,
    label: &'static str,
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    frames: &[I420Frame],
) -> (usize, Vec<DecodedI420>) {
    ensure_encoding_supported(hevc);

    let config = realtime_encoder_config(width, height, bitrate_kbps, hevc);
    let mut encoder = Encoder::new(config).expect("Video Toolbox エンコーダの生成に失敗");
    let mut packets = Vec::new();

    for (index, frame) in frames.iter().enumerate() {
        encoder
            .encode(
                &FrameData::I420 {
                    y: &frame.y,
                    u: &frame.u,
                    v: &frame.v,
                },
                &EncodeOptions {
                    force_key_frame: index == 0,
                },
            )
            .expect("Video Toolbox エンコードに失敗");
        while let Some(encoded) = encoder
            .next_frame()
            .expect("Video Toolbox next_frame に失敗")
        {
            packets.push(encoded);
        }
    }

    encoder.finish().expect("Video Toolbox finish に失敗");
    while let Some(encoded) = encoder
        .next_frame()
        .expect("Video Toolbox flush next_frame に失敗")
    {
        packets.push(encoded);
    }

    let encoded_size: usize = packets.iter().map(|packet| packet.data.len()).sum();
    assert!(!packets.is_empty(), "{label} encoded packets are empty");

    let decoded = if hevc {
        decode_hevc_packets(&packets)
    } else {
        decode_h264_packets(&packets)
    };
    assert_eq!(
        decoded.len(),
        frames.len(),
        "{label} decoded frame count mismatch: got {}, expected {}",
        decoded.len(),
        frames.len()
    );

    (encoded_size, decoded)
}

pub fn encode_decode_h264(
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    frames: &[I420Frame],
) -> (usize, Vec<DecodedI420>) {
    encode_video_toolbox(false, "H.264", width, height, bitrate_kbps, frames)
}

pub fn encode_decode_hevc(
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    frames: &[I420Frame],
) -> (usize, Vec<DecodedI420>) {
    encode_video_toolbox(true, "H.265", width, height, bitrate_kbps, frames)
}
