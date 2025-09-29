use ffmpeg_next as ffmpeg;
use std::{collections::HashMap, path::Path};

// TODO: Abstract away the ffmpeg types and methods

fn parse_duration_string(duration_str: &str) -> Option<f64> {
    // Parse duration strings like "00:45:23.000000000"
    let parts: Vec<&str> = duration_str.split(':').collect();
    if parts.len() == 3
        && let (Ok(hours), Ok(minutes), Ok(seconds)) = (
            parts[0].parse::<f64>(),
            parts[1].parse::<f64>(),
            parts[2].parse::<f64>(),
        )
    {
        return Some(hours * 3600.0 + minutes * 60.0 + seconds);
    }
    None
}

#[derive(Debug)]
pub struct VideoMetadata {
    pub bit_rate: usize,
    pub max_rate: usize,
    pub delay: usize,
    pub width: u32,
    pub height: u32,
    pub format: ffmpeg::format::Pixel,
    pub has_b_frames: bool,
    pub aspect_ratio: ffmpeg::Rational,
    pub color_space: ffmpeg::color::Space,
    pub color_range: ffmpeg::color::Range,
    pub color_primaries: ffmpeg::color::Primaries,
    pub color_transfer_characteristic: ffmpeg::color::TransferCharacteristic,
    pub chroma_location: ffmpeg::chroma::Location,
    pub references: usize,
    pub intra_dc_precision: u8,
    pub bit_depth: u8,
    pub profile: String,
    pub level: String,
    pub codec_name: String,
}

impl VideoMetadata {
    /// Get the actual bitrate, using metadata fallback if the primary bitrate is 0
    pub fn actual_bit_rate(&self, stream_metadata: &HashMap<String, String>) -> f64 {
        if self.bit_rate > 0 {
            self.bit_rate as f64
        } else if let Some(bps_str) = stream_metadata.get("BPS") {
            bps_str.parse::<f64>().unwrap_or(0.0)
        } else {
            0.0
        }
    }
}

#[derive(Debug)]
pub struct AudioMetadata {
    pub bit_rate: usize,
    pub max_rate: usize,
    pub delay: usize,
    pub rate: u32,
    pub channels: u16,
    pub format: ffmpeg::format::Sample,
    pub frames: usize,
    pub align: usize,
    pub channel_layout: ffmpeg::channel_layout::ChannelLayout,
    pub codec_name: String,
    pub profile: String,
    pub title: String,
    pub language: String,
}

impl AudioMetadata {
    /// Get the actual bitrate, using metadata fallback if the primary bitrate is 0
    pub fn actual_bit_rate(&self, stream_metadata: &HashMap<String, String>) -> f64 {
        if self.bit_rate > 0 {
            self.bit_rate as f64
        } else if let Some(bps_str) = stream_metadata.get("BPS") {
            bps_str.parse::<f64>().unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Get the actual frame count, using metadata fallback if frames is 0
    pub fn actual_frames(&self, stream_metadata: &HashMap<String, String>) -> i64 {
        if self.frames > 0 {
            self.frames as i64
        } else if let Some(frames_str) = stream_metadata.get("NUMBER_OF_FRAMES") {
            frames_str.parse::<i64>().unwrap_or(0)
        } else {
            0
        }
    }

    /// Get a human-readable description of the channel layout
    pub fn channel_layout_description(&self) -> &'static str {
        match self.channels {
            1 => "Mono",
            2 => "Stereo",
            6 => "5.1",
            8 => "7.1",
            _ => "Multi-channel",
        }
    }
}

#[derive(Debug)]
pub struct StreamMetadata {
    pub index: usize,
    pub time_base: ffmpeg::Rational,
    pub start_time: i64,
    pub duration: i64,
    pub frames: i64,
    pub disposition: ffmpeg::format::stream::Disposition,
    pub discard: ffmpeg::Discard,
    pub rate: ffmpeg::Rational,
    pub medium: ffmpeg::media::Type,
    pub codec_id: ffmpeg::codec::Id,
    pub video: Option<VideoMetadata>,
    pub audio: Option<AudioMetadata>,
    pub metadata: HashMap<String, String>,
}

impl StreamMetadata {
    /// Compute duration in seconds from duration and time_base
    pub fn duration_seconds(&self) -> f64 {
        self.duration as f64 * f64::from(self.time_base)
    }

    /// Compute frame rate from the stream rate
    pub fn frame_rate(&self) -> f64 {
        f64::from(self.rate)
    }

    /// Get the actual duration, using metadata fallback if duration is 0
    pub fn actual_duration_seconds(&self, file_duration_seconds: f64) -> f64 {
        if self.duration_seconds() > 0.0 {
            self.duration_seconds()
        } else {
            // Try to get duration from metadata or fall back to file duration
            if let Some(duration_str) = self.metadata.get("DURATION") {
                parse_duration_string(duration_str).unwrap_or(file_duration_seconds)
            } else {
                file_duration_seconds
            }
        }
    }

    /// Get the actual frame count, using metadata fallback if frames is 0
    pub fn actual_frames(&self) -> i64 {
        if self.frames > 0 {
            self.frames
        } else {
            // Try to get frame count from metadata
            if let Some(frames_str) = self.metadata.get("NUMBER_OF_FRAMES") {
                frames_str.parse::<i64>().unwrap_or(0)
            } else {
                0
            }
        }
    }
}

#[derive(Debug)]
pub struct FileMetadata {
    pub metadata: HashMap<String, String>,
    pub best_video_stream: Option<usize>,
    pub best_audio_stream: Option<usize>,
    pub best_subtitle_stream: Option<usize>,
    pub duration: i64,
    pub streams: Vec<StreamMetadata>,
    pub format_name: String,
    pub format_long_name: String,
    pub file_size: u64,
    pub bit_rate: i64,
    pub probe_score: i32,
}

impl FileMetadata {
    /// Compute duration in seconds from duration in AV_TIME_BASE units
    pub fn duration_seconds(&self) -> f64 {
        self.duration as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE)
    }
}

pub fn extract_metadata(file_path: &Path) -> Result<FileMetadata, ffmpeg::Error> {
    match ffmpeg::format::input(file_path) {
        Ok(context) => {
            // Collect file-level metadata
            let mut metadata = HashMap::new();
            for (k, v) in context.metadata().iter() {
                metadata.insert(k.to_string(), v.to_string());
            }

            // Find best streams
            let best_video_stream = context
                .streams()
                .best(ffmpeg::media::Type::Video)
                .map(|s| s.index());
            let best_audio_stream = context
                .streams()
                .best(ffmpeg::media::Type::Audio)
                .map(|s| s.index());
            let best_subtitle_stream = context
                .streams()
                .best(ffmpeg::media::Type::Subtitle)
                .map(|s| s.index());

            // Get duration in AV_TIME_BASE units
            let duration = context.duration();

            // Process all streams
            let mut streams = Vec::new();
            for stream in context.streams() {
                let codec = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
                let medium = codec.medium();
                let codec_id = codec.id();

                let mut video = None;
                let mut audio = None;

                if medium == ffmpeg::media::Type::Video {
                    if let Ok(video_decoder) = codec.decoder().video() {
                        let codec_name = format!("{:?}", codec_id);
                        let profile = format!("{:?}", video_decoder.profile());
                        let level = "Unknown".to_string(); // Level not directly available in ffmpeg-next

                        // Try to get bit depth from format
                        let bit_depth = match video_decoder.format() {
                            ffmpeg::format::Pixel::YUV420P10LE
                            | ffmpeg::format::Pixel::YUV420P10BE => 10,
                            ffmpeg::format::Pixel::YUV420P12LE
                            | ffmpeg::format::Pixel::YUV420P12BE => 12,
                            ffmpeg::format::Pixel::YUV420P16LE
                            | ffmpeg::format::Pixel::YUV420P16BE => 16,
                            _ => 8, // Default to 8-bit
                        };

                        video = Some(VideoMetadata {
                            bit_rate: video_decoder.bit_rate(),
                            max_rate: video_decoder.max_bit_rate(),
                            delay: video_decoder.delay(),
                            width: video_decoder.width(),
                            height: video_decoder.height(),
                            format: video_decoder.format(),
                            has_b_frames: video_decoder.has_b_frames(),
                            aspect_ratio: video_decoder.aspect_ratio(),
                            color_space: video_decoder.color_space(),
                            color_range: video_decoder.color_range(),
                            color_primaries: video_decoder.color_primaries(),
                            color_transfer_characteristic: video_decoder
                                .color_transfer_characteristic(),
                            chroma_location: video_decoder.chroma_location(),
                            references: video_decoder.references(),
                            intra_dc_precision: video_decoder.intra_dc_precision(),
                            bit_depth,
                            profile,
                            level,
                            codec_name,
                        });
                    }
                } else if medium == ffmpeg::media::Type::Audio {
                    if let Ok(audio_decoder) = codec.decoder().audio() {
                        let codec_name = format!("{:?}", codec_id);
                        let profile = format!("{:?}", audio_decoder.profile());

                        // Get title and language from stream metadata
                        let mut title = String::new();
                        let mut language = String::new();

                        for (k, v) in stream.metadata().iter() {
                            match k {
                                "title" => title = v.to_string(),
                                "language" => language = v.to_string(),
                                _ => {}
                            }
                        }

                        audio = Some(AudioMetadata {
                            bit_rate: audio_decoder.bit_rate(),
                            max_rate: audio_decoder.max_bit_rate(),
                            delay: audio_decoder.delay(),
                            rate: audio_decoder.rate(),
                            channels: audio_decoder.channels(),
                            format: audio_decoder.format(),
                            frames: audio_decoder.frames(),
                            align: audio_decoder.align(),
                            channel_layout: audio_decoder.channel_layout(),
                            codec_name,
                            profile,
                            title,
                            language,
                        });
                    }
                }

                // Collect stream metadata
                let mut stream_metadata = HashMap::new();
                for (k, v) in stream.metadata().iter() {
                    stream_metadata.insert(k.to_string(), v.to_string());
                }

                streams.push(StreamMetadata {
                    index: stream.index(),
                    time_base: stream.time_base(),
                    start_time: stream.start_time(),
                    duration: stream.duration(),
                    frames: stream.frames(),
                    disposition: stream.disposition(),
                    discard: stream.discard(),
                    rate: stream.rate(),
                    medium,
                    codec_id,
                    video,
                    audio,
                    metadata: stream_metadata,
                });
            }

            // Get format information
            let format_name = context.format().name().to_string();
            let format_long_name = context.format().description().to_string();
            let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
            let bit_rate = context.bit_rate();
            let probe_score = context.probe_score();

            Ok(FileMetadata {
                metadata,
                best_video_stream,
                best_audio_stream,
                best_subtitle_stream,
                duration,
                streams,
                format_name,
                format_long_name,
                file_size,
                bit_rate,
                probe_score,
            })
        }

        Err(error) => Err(error),
    }
}
