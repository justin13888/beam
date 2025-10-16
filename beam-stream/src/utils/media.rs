use ffmpeg_next as ffmpeg;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct MediaType {
    inner: ffmpeg::media::Type,
}

impl MediaType {
    pub fn inner(&self) -> &ffmpeg::media::Type {
        &self.inner
    }

    /// Check if this is a video stream
    pub fn is_video(&self) -> bool {
        matches!(self.inner, ffmpeg::media::Type::Video)
    }

    /// Check if this is an audio stream
    pub fn is_audio(&self) -> bool {
        matches!(self.inner, ffmpeg::media::Type::Audio)
    }

    /// Check if this is a subtitle stream
    pub fn is_subtitle(&self) -> bool {
        matches!(self.inner, ffmpeg::media::Type::Subtitle)
    }

    /// Check if this is a data stream
    pub fn is_data(&self) -> bool {
        matches!(self.inner, ffmpeg::media::Type::Data)
    }

    /// Check if this is an attachment stream
    pub fn is_attachment(&self) -> bool {
        matches!(self.inner, ffmpeg::media::Type::Attachment)
    }

    /// Get a human-readable description of the media type
    pub fn description(&self) -> &'static str {
        match self.inner {
            ffmpeg::media::Type::Video => "Video",
            ffmpeg::media::Type::Audio => "Audio",
            ffmpeg::media::Type::Subtitle => "Subtitle",
            ffmpeg::media::Type::Data => "Data",
            ffmpeg::media::Type::Attachment => "Attachment",
            ffmpeg::media::Type::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Debug for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl From<ffmpeg::media::Type> for MediaType {
    fn from(media_type: ffmpeg::media::Type) -> Self {
        MediaType { inner: media_type }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct CodecId {
    inner: ffmpeg::codec::Id,
}

impl CodecId {
    pub fn inner(&self) -> &ffmpeg::codec::Id {
        &self.inner
    }

    /// Get the media type for this codec
    pub fn media_type(&self) -> MediaType {
        match self.inner {
            // Video codecs
            ffmpeg::codec::Id::H264
            | ffmpeg::codec::Id::H265
            | ffmpeg::codec::Id::VP8
            | ffmpeg::codec::Id::VP9
            | ffmpeg::codec::Id::AV1
            | ffmpeg::codec::Id::MPEG1VIDEO
            | ffmpeg::codec::Id::MPEG2VIDEO
            | ffmpeg::codec::Id::MPEG4 => MediaType::from(ffmpeg::media::Type::Video),
            // Audio codecs
            ffmpeg::codec::Id::AAC
            | ffmpeg::codec::Id::MP3
            | ffmpeg::codec::Id::AC3
            | ffmpeg::codec::Id::EAC3
            | ffmpeg::codec::Id::DTS
            | ffmpeg::codec::Id::TRUEHD
            | ffmpeg::codec::Id::FLAC
            | ffmpeg::codec::Id::VORBIS
            | ffmpeg::codec::Id::OPUS => MediaType::from(ffmpeg::media::Type::Audio),
            // Subtitle codecs
            ffmpeg::codec::Id::SUBRIP | ffmpeg::codec::Id::ASS | ffmpeg::codec::Id::WEBVTT => {
                MediaType::from(ffmpeg::media::Type::Subtitle)
            }
            _ => MediaType::from(ffmpeg::media::Type::Unknown),
        }
    }

    /// Check if this is a video codec
    pub fn is_video(&self) -> bool {
        self.media_type().is_video()
    }

    /// Check if this is an audio codec
    pub fn is_audio(&self) -> bool {
        self.media_type().is_audio()
    }

    /// Check if this is a subtitle codec
    pub fn is_subtitle(&self) -> bool {
        self.media_type().is_subtitle()
    }

    /// Get a human-readable name for the codec
    pub fn name(&self) -> &'static str {
        match self.inner {
            // Video codecs
            ffmpeg::codec::Id::H264 => "H.264/AVC",
            ffmpeg::codec::Id::H265 => "H.265/HEVC",
            ffmpeg::codec::Id::VP8 => "VP8",
            ffmpeg::codec::Id::VP9 => "VP9",
            ffmpeg::codec::Id::AV1 => "AV1",
            ffmpeg::codec::Id::MPEG1VIDEO => "MPEG-1",
            ffmpeg::codec::Id::MPEG2VIDEO => "MPEG-2",
            ffmpeg::codec::Id::MPEG4 => "MPEG-4",
            // Audio codecs
            ffmpeg::codec::Id::AAC => "AAC",
            ffmpeg::codec::Id::MP3 => "MP3",
            ffmpeg::codec::Id::AC3 => "AC-3",
            ffmpeg::codec::Id::EAC3 => "E-AC-3",
            ffmpeg::codec::Id::DTS => "DTS",
            ffmpeg::codec::Id::TRUEHD => "TrueHD",
            ffmpeg::codec::Id::FLAC => "FLAC",
            ffmpeg::codec::Id::VORBIS => "Vorbis",
            ffmpeg::codec::Id::OPUS => "Opus",
            // Subtitle codecs
            ffmpeg::codec::Id::SUBRIP => "SubRip",
            ffmpeg::codec::Id::ASS => "ASS/SSA",
            ffmpeg::codec::Id::WEBVTT => "WebVTT",
            _ => "Unknown",
        }
    }

    /// Check if this codec supports hardware acceleration
    pub fn supports_hardware_acceleration(&self) -> bool {
        matches!(
            self.inner,
            ffmpeg::codec::Id::H264
                | ffmpeg::codec::Id::H265
                | ffmpeg::codec::Id::VP9
                | ffmpeg::codec::Id::AV1
        )
    }

    /// Check if this is a lossless codec
    pub fn is_lossless(&self) -> bool {
        matches!(
            self.inner,
            ffmpeg::codec::Id::FLAC
                | ffmpeg::codec::Id::SUBRIP
                | ffmpeg::codec::Id::ASS
                | ffmpeg::codec::Id::WEBVTT
        )
    }
}

impl From<ffmpeg::codec::Id> for CodecId {
    fn from(codec_id: ffmpeg::codec::Id) -> Self {
        CodecId { inner: codec_id }
    }
}

impl std::fmt::Display for CodecId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::fmt::Debug for CodecId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Discard {
    inner: ffmpeg::Discard,
}

impl Discard {
    pub fn inner(&self) -> &ffmpeg::Discard {
        &self.inner
    }

    /// Check if this stream should be discarded
    pub fn should_discard(&self) -> bool {
        !matches!(self.inner, ffmpeg::Discard::Default)
    }

    /// Get a human-readable description of the discard setting
    pub fn description(&self) -> &'static str {
        match self.inner {
            ffmpeg::Discard::None => "None",
            ffmpeg::Discard::Default => "Default",
            ffmpeg::Discard::NonReference => "Non-Reference",
            ffmpeg::Discard::Bidirectional => "Bidirectional",
            ffmpeg::Discard::NonIntra => "Non-Intra",
            ffmpeg::Discard::NonKey => "Non-Key",
            ffmpeg::Discard::All => "All",
        }
    }
}

impl From<ffmpeg::Discard> for Discard {
    fn from(discard: ffmpeg::Discard) -> Self {
        Discard { inner: discard }
    }
}

impl std::fmt::Debug for Discard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
