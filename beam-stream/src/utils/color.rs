use ffmpeg_next as ffmpeg;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct PixelFormat {
    inner: ffmpeg::format::Pixel,
}

impl PixelFormat {
    pub fn inner(&self) -> &ffmpeg::format::Pixel {
        &self.inner
    }

    /// Get bit depth from pixel format
    /// Returns None if unknown.
    pub fn bit_depth(&self) -> Option<u8> {
        match self.inner {
            ffmpeg::format::Pixel::YUV420P10LE | ffmpeg::format::Pixel::YUV420P10BE => Some(10),
            ffmpeg::format::Pixel::YUV420P12LE | ffmpeg::format::Pixel::YUV420P12BE => Some(12),
            ffmpeg::format::Pixel::YUV420P16LE | ffmpeg::format::Pixel::YUV420P16BE => Some(16),
            _ => None,
        } // TODO: Include everything supported by FFmpeg 8.0+
    }
}

impl From<ffmpeg::format::Pixel> for PixelFormat {
    fn from(pixel: ffmpeg::format::Pixel) -> Self {
        PixelFormat { inner: pixel }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ColorSpace {
    inner: ffmpeg::color::Space,
}

impl ColorSpace {
    pub fn inner(&self) -> &ffmpeg::color::Space {
        &self.inner
    }

    /// Get a human-readable description of the color space
    pub fn description(&self) -> &'static str {
        match self.inner {
            ffmpeg::color::Space::RGB => "RGB",
            ffmpeg::color::Space::BT709 => "BT.709",
            ffmpeg::color::Space::BT470BG => "BT.470 BG",
            ffmpeg::color::Space::SMPTE170M => "SMPTE-170M",
            ffmpeg::color::Space::SMPTE240M => "SMPTE-240M",
            ffmpeg::color::Space::BT2020NCL => "BT.2020 Non-Constant Luminance",
            ffmpeg::color::Space::BT2020CL => "BT.2020 Constant Luminance",
            _ => "Unknown",
        } // TODO: Include everything supported by FFmpeg 8.0+
    }
}

impl From<ffmpeg::color::Space> for ColorSpace {
    fn from(space: ffmpeg::color::Space) -> Self {
        ColorSpace { inner: space }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ColorRange {
    inner: ffmpeg::color::Range,
}

impl ColorRange {
    pub fn inner(&self) -> &ffmpeg::color::Range {
        &self.inner
    }

    /// Get a human-readable description of the color range
    pub fn description(&self) -> &'static str {
        match self.inner {
            ffmpeg::color::Range::MPEG => "Limited (TV)",
            ffmpeg::color::Range::JPEG => "Full (PC)",
            _ => "Unknown",
        } // TODO: Include everything supported by FFmpeg 8.0+
    }
}

impl From<ffmpeg::color::Range> for ColorRange {
    fn from(range: ffmpeg::color::Range) -> Self {
        ColorRange { inner: range }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ColorPrimaries {
    inner: ffmpeg::color::Primaries,
}

impl ColorPrimaries {
    pub fn inner(&self) -> &ffmpeg::color::Primaries {
        &self.inner
    }

    /// Get a human-readable description of the color primaries
    pub fn description(&self) -> &'static str {
        match self.inner {
            ffmpeg::color::Primaries::BT709 => "BT.709",
            ffmpeg::color::Primaries::BT470BG => "BT.470 BG",
            ffmpeg::color::Primaries::SMPTE170M => "SMPTE-170M",
            ffmpeg::color::Primaries::SMPTE240M => "SMPTE-240M",
            ffmpeg::color::Primaries::BT2020 => "BT.2020",
            ffmpeg::color::Primaries::SMPTE428 => "SMPTE-428",
            ffmpeg::color::Primaries::SMPTE431 => "SMPTE-431",
            ffmpeg::color::Primaries::SMPTE432 => "SMPTE-432",
            _ => "Unknown",
        } // TODO: Include everything supported by FFmpeg 8.0+
    }
}

impl From<ffmpeg::color::Primaries> for ColorPrimaries {
    fn from(primaries: ffmpeg::color::Primaries) -> Self {
        ColorPrimaries { inner: primaries }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ColorTransferCharacteristic {
    inner: ffmpeg::color::TransferCharacteristic,
}

impl ColorTransferCharacteristic {
    pub fn inner(&self) -> &ffmpeg::color::TransferCharacteristic {
        &self.inner
    }

    /// Get a human-readable description of the transfer characteristic
    pub fn description(&self) -> &'static str {
        match self.inner {
            ffmpeg::color::TransferCharacteristic::BT709 => "BT.709",
            ffmpeg::color::TransferCharacteristic::SMPTE170M => "SMPTE-170M",
            ffmpeg::color::TransferCharacteristic::SMPTE240M => "SMPTE-240M", 
            ffmpeg::color::TransferCharacteristic::SMPTE2084 => "SMPTE-2084 (PQ)",
            ffmpeg::color::TransferCharacteristic::ARIB_STD_B67 => "HLG (Hybrid Log-Gamma)",
            _ => "Unknown",
        } // TODO: Include everything supported by FFmpeg 8.0+
    }

    /// Check if this is an HDR transfer characteristic
    pub fn is_hdr(&self) -> bool {
        matches!(
            self.inner,
            ffmpeg::color::TransferCharacteristic::SMPTE2084
                | ffmpeg::color::TransferCharacteristic::ARIB_STD_B67
        )
    }
}

impl From<ffmpeg::color::TransferCharacteristic> for ColorTransferCharacteristic {
    fn from(transfer: ffmpeg::color::TransferCharacteristic) -> Self {
        ColorTransferCharacteristic { inner: transfer }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ChromaLocation {
    inner: ffmpeg::chroma::Location,
}

impl ChromaLocation {
    pub fn inner(&self) -> &ffmpeg::chroma::Location {
        &self.inner
    }

    /// Get a human-readable description of the chroma location
    pub fn description(&self) -> &'static str {
        match self.inner {
            ffmpeg::chroma::Location::Left => "Left",
            ffmpeg::chroma::Location::Center => "Center",
            ffmpeg::chroma::Location::TopLeft => "Top Left",
            ffmpeg::chroma::Location::Top => "Top",
            ffmpeg::chroma::Location::BottomLeft => "Bottom Left", 
            ffmpeg::chroma::Location::Bottom => "Bottom",
            _ => "Unknown",
        } // TODO: Include everything supported by FFmpeg 8.0+
    }
}

impl From<ffmpeg::chroma::Location> for ChromaLocation {
    fn from(location: ffmpeg::chroma::Location) -> Self {
        ChromaLocation { inner: location }
    }
}
