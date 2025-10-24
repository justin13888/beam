use async_graphql::Enum;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema, Enum)]
pub enum OutputVideoCodec {
    H264,
    H265,
    AV1,
    UNKNOWN,
}

impl From<&beam_stream::utils::codec::OutputVideoCodec> for OutputVideoCodec {
    fn from(codec: &beam_stream::utils::codec::OutputVideoCodec) -> Self {
        match codec {
            beam_stream::utils::codec::OutputVideoCodec::Remuxed(c) => {
                match c.to_ascii_lowercase().as_str() {
                    "h264" => OutputVideoCodec::H264,
                    "h265" => OutputVideoCodec::H265,
                    "av1" => OutputVideoCodec::AV1,
                    _ => OutputVideoCodec::UNKNOWN,
                }
            }
            beam_stream::utils::codec::OutputVideoCodec::H264 => OutputVideoCodec::H264,
            beam_stream::utils::codec::OutputVideoCodec::H265 => OutputVideoCodec::H265,
            beam_stream::utils::codec::OutputVideoCodec::AV1 => OutputVideoCodec::AV1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema, Enum)]
pub enum OutputAudioCodec {
    Aac,
    Opus,
    Unknown,
}

impl From<&beam_stream::utils::codec::OutputAudioCodec> for OutputAudioCodec {
    fn from(codec: &beam_stream::utils::codec::OutputAudioCodec) -> Self {
        match codec {
            beam_stream::utils::codec::OutputAudioCodec::Remuxed(c) => {
                match c.to_ascii_lowercase().as_str() {
                    "aac" => OutputAudioCodec::Aac,
                    "opus" => OutputAudioCodec::Opus,
                    _ => OutputAudioCodec::Unknown,
                }
            }
            beam_stream::utils::codec::OutputAudioCodec::AacLc => OutputAudioCodec::Aac,
            beam_stream::utils::codec::OutputAudioCodec::Opus => OutputAudioCodec::Opus,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema, Enum)]
pub enum OutputSubtitleCodec {
    WebVTT,
}

impl From<&beam_stream::utils::codec::OutputSubtitleCodec> for OutputSubtitleCodec {
    fn from(codec: &beam_stream::utils::codec::OutputSubtitleCodec) -> Self {
        match codec {
            beam_stream::utils::codec::OutputSubtitleCodec::WebVTT => OutputSubtitleCodec::WebVTT,
        }
    }
}
