use shiguredo_aom::{
    Decoder, DecoderConfig, EncodeOptions, Encoder, EncoderConfig, ImageData, ImageFormat,
    RateControlMode as AomRateControlMode, Usage,
};
use shiguredo_libvpx::{
    CodecConfig, Decoder as VpxDecoder, DecoderCodec, DecoderConfig as VpxDecoderConfig,
    EncodeOptions as VpxEncodeOptions, Encoder as VpxEncoder, EncoderConfig as VpxEncoderConfig,
    EncodingDeadline, ImageData as VpxImageData, ImageFormat as VpxImageFormat,
    RateControlMode as VpxRateControlMode, Vp8Config, Vp9Config,
};

use crate::pixel::pack_plane;
use crate::types::{Codec, DecodedI420, I420Frame, RoundtripMetrics};
use crate::vmaf::measure_roundtrip;

/// libaom-av1 リアルタイム符号化設定
fn realtime_av1_encoder_config(width: u32, height: u32, bitrate_kbps: u32) -> EncoderConfig {
    let mut config = EncoderConfig::new(width, height, ImageFormat::I420);
    config.g_usage = Usage::Realtime;
    config.rc_end_usage = AomRateControlMode::Cbr;
    config.rc_target_bitrate = bitrate_kbps;
    config.rc_min_quantizer = 10;
    config.rc_max_quantizer = 52;
    config.cpu_used = Some(8);
    config.aq_mode = Some(3);
    config.row_mt = Some(true);
    config.enable_global_motion = Some(false);
    config.enable_restoration = Some(false);
    config.enable_intrabc = Some(false);
    config
}

/// libvpx-vp9 リアルタイム符号化設定
fn realtime_vp9_encoder_config(width: u32, height: u32, bitrate_kbps: u32) -> VpxEncoderConfig {
    let mut config = VpxEncoderConfig::new(
        width as usize,
        height as usize,
        VpxImageFormat::I420,
        CodecConfig::Vp9(Vp9Config {
            row_mt: true,
            tile_columns: Some(3),
            tile_rows: Some(1),
            frame_parallel_decoding: true,
            ..Vp9Config::default()
        }),
    );
    config.target_bitrate = usize::try_from(bitrate_kbps).expect("bitrate fits in usize") * 1000;
    config.min_quantizer = 2;
    config.max_quantizer = 52;
    config.cpu_used = Some(7);
    config.deadline = EncodingDeadline::Realtime;
    config.rate_control = VpxRateControlMode::Cbr;
    config
}

/// libvpx (VP8) リアルタイム符号化設定
fn realtime_vp8_encoder_config(width: u32, height: u32, bitrate_kbps: u32) -> VpxEncoderConfig {
    let mut config = VpxEncoderConfig::new(
        width as usize,
        height as usize,
        VpxImageFormat::I420,
        CodecConfig::Vp8(Vp8Config::default()),
    );
    config.target_bitrate = usize::try_from(bitrate_kbps).expect("bitrate fits in usize") * 1000;
    config.min_quantizer = 2;
    config.max_quantizer = 52;
    // libvpx の VP8E_SET_CPUUSED は符号付き。リアルタイム符号化の既定は -6。
    config.cpu_used = Some((-6i32 as u32) as usize);
    config.deadline = EncodingDeadline::Realtime;
    config.rate_control = VpxRateControlMode::Cbr;
    config
}

fn push_aom_decoded_frames(decoded: &mut Vec<DecodedI420>, decoder: &mut Decoder) {
    while let Some(frame) = decoder.next_frame() {
        let y_stride = frame.y_stride().expect("Y stride");
        let u_stride = frame.u_stride().expect("U stride");
        let v_stride = frame.v_stride().expect("V stride");
        let width = frame.width();
        let height = frame.height();
        decoded.push(DecodedI420 {
            y: pack_plane(frame.y_plane().expect("Y plane"), width, height, y_stride),
            u: pack_plane(
                frame.u_plane().expect("U plane"),
                width.div_ceil(2),
                height.div_ceil(2),
                u_stride,
            ),
            v: pack_plane(
                frame.v_plane().expect("V plane"),
                width.div_ceil(2),
                height.div_ceil(2),
                v_stride,
            ),
        });
    }
}

fn decode_aom_packets(packets: &[Vec<u8>]) -> Vec<DecodedI420> {
    let mut decoder = Decoder::new(DecoderConfig::default()).expect("AOM Decoder の生成に失敗");
    let mut decoded = Vec::new();

    for packet in packets {
        decoder.decode(packet).expect("AOM decode に失敗");
        push_aom_decoded_frames(&mut decoded, &mut decoder);
    }

    decoder.finish().expect("AOM decoder finish に失敗");
    push_aom_decoded_frames(&mut decoded, &mut decoder);

    decoded
}

fn push_vpx_decoded_frames(decoded: &mut Vec<DecodedI420>, decoder: &mut VpxDecoder) {
    while let Some(frame) = decoder.next_frame().expect("next_frame") {
        let width = frame.width();
        let height = frame.height();
        decoded.push(DecodedI420 {
            y: pack_plane(frame.y_plane(), width, height, frame.y_stride()),
            u: pack_plane(frame.u_plane(), width / 2, height / 2, frame.u_stride()),
            v: pack_plane(frame.v_plane(), width / 2, height / 2, frame.v_stride()),
        });
    }
}

fn decode_vpx_packets(codec: DecoderCodec, packets: &[Vec<u8>]) -> Vec<DecodedI420> {
    let mut decoder =
        VpxDecoder::new(VpxDecoderConfig::new(codec)).expect("libvpx Decoder の生成に失敗");
    let mut decoded = Vec::new();

    for packet in packets {
        decoder.decode(packet).expect("libvpx decode に失敗");
        push_vpx_decoded_frames(&mut decoded, &mut decoder);
    }

    decoder.finish().expect("libvpx decoder finish に失敗");
    push_vpx_decoded_frames(&mut decoded, &mut decoder);

    decoded
}

pub fn encode_decode_aom(
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    frames: &[I420Frame],
) -> (usize, Vec<DecodedI420>) {
    let config = realtime_av1_encoder_config(width, height, bitrate_kbps);
    let mut encoder = Encoder::new(config).expect("AOM Encoder の生成に失敗");
    let mut packets = Vec::new();

    for (index, frame) in frames.iter().enumerate() {
        let options = EncodeOptions {
            force_keyframe: index == 0,
        };
        let image = ImageData::I420 {
            y: &frame.y,
            u: &frame.u,
            v: &frame.v,
        };
        encoder.encode(&image, &options).expect("AOM encode に失敗");
        while let Some(encoded) = encoder.next_frame() {
            packets.push(encoded.data().expect("encoded data").to_vec());
        }
    }

    encoder.finish().expect("AOM finish に失敗");
    while let Some(encoded) = encoder.next_frame() {
        packets.push(encoded.data().expect("encoded data").to_vec());
    }

    let encoded_size: usize = packets.iter().map(|p| p.len()).sum();
    assert!(!packets.is_empty(), "AOM 符号化パケットが空");

    let decoded = decode_aom_packets(&packets);
    assert_eq!(
        decoded.len(),
        frames.len(),
        "AOM デコードフレーム数が一致しない: got {}, expected {}",
        decoded.len(),
        frames.len()
    );

    (encoded_size, decoded)
}

fn encode_decode_vpx(
    codec: DecoderCodec,
    label: &'static str,
    make_config: fn(u32, u32, u32) -> VpxEncoderConfig,
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    frames: &[I420Frame],
) -> (usize, Vec<DecodedI420>) {
    let config = make_config(width, height, bitrate_kbps);
    let mut encoder = VpxEncoder::new(config).expect("libvpx Encoder の生成に失敗");
    let mut packets = Vec::new();

    for (index, frame) in frames.iter().enumerate() {
        encoder
            .encode(
                &VpxImageData::I420 {
                    y: &frame.y,
                    u: &frame.u,
                    v: &frame.v,
                },
                &VpxEncodeOptions {
                    force_keyframe: index == 0,
                },
            )
            .expect("libvpx encode に失敗");
        while let Some(encoded) = encoder.next_frame() {
            packets.push(encoded.data().to_vec());
        }
    }

    encoder.finish().expect("libvpx finish に失敗");
    while let Some(encoded) = encoder.next_frame() {
        packets.push(encoded.data().to_vec());
    }

    let encoded_size: usize = packets.iter().map(|p| p.len()).sum();
    assert!(!packets.is_empty(), "{label} 符号化パケットが空");

    let decoded = decode_vpx_packets(codec, &packets);
    assert_eq!(
        decoded.len(),
        frames.len(),
        "{label} デコードフレーム数が一致しない: got {}, expected {}",
        decoded.len(),
        frames.len()
    );

    (encoded_size, decoded)
}

pub fn encode_decode_vp8(
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    frames: &[I420Frame],
) -> (usize, Vec<DecodedI420>) {
    encode_decode_vpx(
        DecoderCodec::Vp8,
        "libvpx VP8",
        realtime_vp8_encoder_config,
        width,
        height,
        bitrate_kbps,
        frames,
    )
}

pub fn encode_decode_vp9(
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    frames: &[I420Frame],
) -> (usize, Vec<DecodedI420>) {
    encode_decode_vpx(
        DecoderCodec::Vp9,
        "libvpx VP9",
        realtime_vp9_encoder_config,
        width,
        height,
        bitrate_kbps,
        frames,
    )
}

pub fn encode_decode(
    codec: Codec,
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    frames: &[I420Frame],
) -> (usize, Vec<DecodedI420>) {
    match codec {
        Codec::Aom => encode_decode_aom(width, height, bitrate_kbps, frames),
        Codec::Vp8 => encode_decode_vp8(width, height, bitrate_kbps, frames),
        Codec::Vp9 => encode_decode_vp9(width, height, bitrate_kbps, frames),
        #[cfg(target_os = "macos")]
        Codec::H264 => {
            crate::video_toolbox::encode_decode_h264(width, height, bitrate_kbps, frames)
        }
        #[cfg(target_os = "macos")]
        Codec::Hevc => {
            crate::video_toolbox::encode_decode_hevc(width, height, bitrate_kbps, frames)
        }
    }
}

pub fn measure_codec_at_bitrate(
    codec: Codec,
    width: u32,
    height: u32,
    bitrate_kbps: u32,
    frames: &[I420Frame],
) -> RoundtripMetrics {
    let (size, decoded) = encode_decode(codec, width, height, bitrate_kbps, frames);
    measure_roundtrip(width, height, frames, size, &decoded)
}
