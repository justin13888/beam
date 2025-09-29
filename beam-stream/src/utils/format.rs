use ffmpeg_next as ffmpeg;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct SampleFormat {
    inner: ffmpeg::format::Sample,
}

impl SampleFormat {
    pub fn inner(&self) -> &ffmpeg::format::Sample {
        &self.inner
    }

    /// Get bit depth from sample format
    pub fn bit_depth(&self) -> Option<u8> {
        match self.inner {
            ffmpeg::format::Sample::U8(_) => Some(8),
            ffmpeg::format::Sample::I16(_) => Some(16),
            ffmpeg::format::Sample::I32(_) => Some(32),
            ffmpeg::format::Sample::I64(_) => Some(64),
            ffmpeg::format::Sample::F32(_) => Some(32),
            ffmpeg::format::Sample::F64(_) => Some(64),
            _ => None,
        }
    }

    /// Check if the sample format is planar
    pub fn is_planar(&self) -> bool {
        matches!(
            self.inner,
            ffmpeg::format::Sample::U8(ffmpeg::format::sample::Type::Planar)
                | ffmpeg::format::Sample::I16(ffmpeg::format::sample::Type::Planar)
                | ffmpeg::format::Sample::I32(ffmpeg::format::sample::Type::Planar)
                | ffmpeg::format::Sample::I64(ffmpeg::format::sample::Type::Planar)
                | ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar)
                | ffmpeg::format::Sample::F64(ffmpeg::format::sample::Type::Planar)
        )
    }

    /// Get a human-readable description of the sample format
    pub fn description(&self) -> &'static str {
        match self.inner {
            ffmpeg::format::Sample::U8(ffmpeg::format::sample::Type::Packed) => "8-bit unsigned",
            ffmpeg::format::Sample::U8(ffmpeg::format::sample::Type::Planar) => {
                "8-bit unsigned planar"
            }
            ffmpeg::format::Sample::I16(ffmpeg::format::sample::Type::Packed) => "16-bit signed",
            ffmpeg::format::Sample::I16(ffmpeg::format::sample::Type::Planar) => {
                "16-bit signed planar"
            }
            ffmpeg::format::Sample::I32(ffmpeg::format::sample::Type::Packed) => "32-bit signed",
            ffmpeg::format::Sample::I32(ffmpeg::format::sample::Type::Planar) => {
                "32-bit signed planar"
            }
            ffmpeg::format::Sample::I64(ffmpeg::format::sample::Type::Packed) => "64-bit signed",
            ffmpeg::format::Sample::I64(ffmpeg::format::sample::Type::Planar) => {
                "64-bit signed planar"
            }
            ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed) => "32-bit float",
            ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar) => {
                "32-bit float planar"
            }
            ffmpeg::format::Sample::F64(ffmpeg::format::sample::Type::Packed) => "64-bit float",
            ffmpeg::format::Sample::F64(ffmpeg::format::sample::Type::Planar) => {
                "64-bit float planar"
            }
            ffmpeg::format::Sample::None => "None",
        }
    }
}

impl From<ffmpeg::format::Sample> for SampleFormat {
    fn from(sample: ffmpeg::format::Sample) -> Self {
        SampleFormat { inner: sample }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ChannelLayout {
    inner: ffmpeg::channel_layout::ChannelLayout,
}

impl ChannelLayout {
    pub fn inner(&self) -> &ffmpeg::channel_layout::ChannelLayout {
        &self.inner
    }

    /// Get the number of channels
    pub fn channels(&self) -> u16 {
        self.inner.channels().try_into().unwrap_or(0)
    }

    /// Get a human-readable description of the channel layout
    pub fn description(&self) -> String {
        match self.channels() {
            1 => "Mono".to_string(),
            2 => "Stereo".to_string(),
            6 => "5.1 Surround".to_string(),
            8 => "7.1 Surround".to_string(),
            n => format!("{} channels", n),
        }
    }

    /// Check if this is a standard surround sound layout
    pub fn is_surround(&self) -> bool {
        self.channels() > 2
    }
}

impl From<ffmpeg::channel_layout::ChannelLayout> for ChannelLayout {
    fn from(layout: ffmpeg::channel_layout::ChannelLayout) -> Self {
        ChannelLayout { inner: layout }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Disposition {
    inner: ffmpeg::format::stream::Disposition,
}

impl Disposition {
    pub fn inner(&self) -> &ffmpeg::format::stream::Disposition {
        &self.inner
    }

    /// Check if this stream is the default stream
    pub fn is_default(&self) -> bool {
        self.inner
            .contains(ffmpeg::format::stream::Disposition::DEFAULT)
    }

    /// Check if this stream is forced
    pub fn is_forced(&self) -> bool {
        self.inner
            .contains(ffmpeg::format::stream::Disposition::FORCED)
    }

    /// Check if this stream contains hearing impaired content
    pub fn is_hearing_impaired(&self) -> bool {
        self.inner
            .contains(ffmpeg::format::stream::Disposition::HEARING_IMPAIRED)
    }

    /// Check if this stream contains visual impaired content
    pub fn is_visual_impaired(&self) -> bool {
        self.inner
            .contains(ffmpeg::format::stream::Disposition::VISUAL_IMPAIRED)
    }

    /// Get a human-readable description of the disposition flags
    pub fn description(&self) -> Vec<&'static str> {
        let mut flags = Vec::new();

        if self.is_default() {
            flags.push("Default");
        }
        if self.is_forced() {
            flags.push("Forced");
        }
        if self.is_hearing_impaired() {
            flags.push("Hearing Impaired");
        }
        if self.is_visual_impaired() {
            flags.push("Visual Impaired");
        }

        flags
    }
}

impl From<ffmpeg::format::stream::Disposition> for Disposition {
    fn from(disposition: ffmpeg::format::stream::Disposition) -> Self {
        Disposition { inner: disposition }
    }
}
