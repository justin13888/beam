//! Example code to generate a HLS media playlist (to a hypothetical server) from a video file

// TODO: THIS EXAMPLE IS INCOMPLETE

use beam_stream::utils::metadata::{StreamMetadata, VideoFileMetadata};
use eyre::Result;
use m3u8_rs::{
    AlternativeMedia, MasterPlaylist, MediaPlaylist, MediaPlaylistType, Resolution, VariantStream,
    WRITE_OPT_FLOAT_PRECISION,
};
use num::integer::{gcd, lcm};
use num::rational::Rational32;
use num::traits::cast::ToPrimitive;
use std::{path::PathBuf, sync::atomic::Ordering};

// TODO: Note there are still inaccuracies with the generated playlists

fn main() -> Result<()> {
    // Get first argument as file path
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <video_file>", args[0]);
        std::process::exit(1);
    }
    let file_path = PathBuf::from(&args[1]);

    // Initialize FFmpeg
    ffmpeg_next::init().unwrap();

    // Generate metadata for video file
    let metadata = VideoFileMetadata::from_path(&file_path).expect("Failed to extract metadata");
    println!("Metadata: {:#?}", metadata);

    // Generate HLS playlist in directory
    WRITE_OPT_FLOAT_PRECISION.store(5, Ordering::Relaxed);

    let hls_version = Some(6); // HLS 6 is a good minimum for fMP4, CMAF, low-latency streaming, segment-related features

    // Pick best video, audio and subtitle streams for now
    // You may generate for all the streams later if desired.
    let best_video_stream_idx = metadata.best_video_stream;
    // let best_video_stream_metadata = best_video_stream_idx
    //     .and_then(|idx| metadata.streams.get(idx))
    //     .map(|s| match s {
    //         StreamMetadata::Video(v) => v,
    //         _ => {
    //             unreachable!("Best video stream turned out to be not a video stream: {s:?}!");
    //         }
    //     });
    let best_audio_stream_idx = metadata.best_audio_stream;
    // let best_audio_stream_metadata = best_audio_stream_idx
    //     .and_then(|idx| metadata.streams.get(idx))
    //     .map(|s| match s {
    //         StreamMetadata::Audio(a) => a,
    //         _ => {
    //             unreachable!("Best audio stream turned out to be not an audio stream: {s:?}!");
    //         }
    //     });
    let best_subtitle_stream_idx = metadata.best_subtitle_stream;
    // let best_subtitle_stream_metadata = best_subtitle_stream_idx
    //     .and_then(|idx| metadata.streams.get(idx))
    //     .map(|s| match s {
    //         StreamMetadata::Subtitle(s) => s,
    //         _ => {
    //             unreachable!("Best subtitle stream turned out to be not a subtitle stream: {s:?}!");
    //         }
    //     });

    // TODO: We need to split the video streams into video and audio pairs as well

    // Detect GOP sizes (in seconds) for each video stream
    let gop_sizes: Vec<Rational32> = vec![Rational32::new(48, 24)]; // TODO: Detect GOP sizes from video streams
    // We need to find the lowest common multiple of all GOP sizes to ensure all video streams can be aligned
    let lcm_size: Rational32 = gop_sizes.iter().fold(Rational32::new(1, 1), |acc, &gop| {
        Rational32::new(
            lcm(*acc.numer(), *gop.numer()),
            gcd(*acc.denom(), *gop.denom()),
        )
        // let gcd = acc.lcm(&gop);
        // (acc * gop) / gcd
    });
    // Determine target duration based on GOP sizes
    // 6 second is a good default but we want to make sure `lcm_size` strictly divides target duration
    // i.e., target_duration = k * lcm_size where k = ⌈6*b/a⌉
    let target_duration: u64 = {
        let six = Rational32::new(6, 1);
        let k = (six * lcm_size.recip()).ceil();
        (k * lcm_size).ceil().to_u64().unwrap()
    };

    // #EXT-X-STREAM-INF
    let mut variants: Vec<VariantStream> = vec![];
    // #EXT-X-MEDIA:<attribute-list>
    let mut alternatives: Vec<AlternativeMedia> = vec![];
    // Playlists for each variant stream (uri, MediaPlaylist)
    let mut playlists: Vec<(String, MediaPlaylist)> = vec![];

    // TODO: The variant naming is not aware of uniqueness (e.g. zh-cn and zh-tw may be grouped under language "chi")

    for (i, stream_metadata) in metadata.streams.iter().enumerate() {
        match stream_metadata {
            StreamMetadata::Video(stream_metadata) => {
                let variant_name: String = {
                    let resolution: u32 = stream_metadata.video.height;
                    format!("{resolution}p")
                };
                let bandwidth: u64 = stream_metadata.video.bit_rate as u64;
                let average_bandwidth: Option<u64> = Some(stream_metadata.video.bit_rate as u64);
                let playlist_uri = format!("video/{variant_name}/playlist.m3u8");

                let variant = VariantStream {
                    is_i_frame: false, // TODO: I think we need another variant for I-frames only
                    uri: playlist_uri.clone(),
                    bandwidth,
                    average_bandwidth,
                    codecs: Some(stream_metadata.video.codec_name.clone()),
                    resolution: Some(Resolution {
                        width: stream_metadata.video.width as u64,
                        height: stream_metadata.video.height as u64,
                    }),
                    frame_rate: stream_metadata.rate.to_f64(), // TODO: This should actually be the number of frames per second
                    hdcp_level: None, // We don't set HDCP as we do not use DRM.
                    audio: None,
                    video: None,
                    subtitles: None,
                    closed_captions: None,
                    other_attributes: None,
                };
                variants.push(variant);

                let stream_playlist = MediaPlaylist {
                    // TODO: Fill out the rest of the fields
                    version: hls_version,
                    target_duration,
                    media_sequence: 0,
                    segments: vec![], // TODO
                    discontinuity_sequence: 0,
                    end_list: true,
                    playlist_type: Some(MediaPlaylistType::Vod),
                    i_frames_only: false, // TODO: Set #EXT-X-I-FRAMES-ONLY to true after i-frame extraction is implemented for fast seeking
                    start: None, // This is typically only used if you client to start on a live edge or the beginning of a VOD stream
                    independent_segments: true, // Set to true because we assume segments are encoded to be independently decodable
                    unknown_tags: vec![],
                };
                playlists.push((playlist_uri, stream_playlist));
            }
            StreamMetadata::Audio(stream_metadata) => {
                let variant_name: String = {
                    let codec = stream_metadata.audio.codec_name.clone();
                    let language = {
                        let s = &stream_metadata.audio.language;
                        if s.is_empty() { None } else { Some(s.clone()) }
                    };
                    let mut n = codec;
                    if let Some(language) = language {
                        n += &format!("-{language}");
                    }

                    n
                };
                let group_id = {
                    let codec_id = stream_metadata.codec_id.to_string();
                    let language = {
                        let s = &stream_metadata.audio.language;
                        if s.is_empty() { None } else { Some(s.clone()) }
                    };

                    let mut s = format!("audio_{codec_id}");
                    if let Some(language) = language {
                        s += &format!("_{language}");
                    }

                    s
                };
                let playlist_uri = format!("audio/{variant_name}/index.m3u8");

                let alternative = AlternativeMedia {
                    media_type: m3u8_rs::AlternativeMediaType::Audio,
                    uri: Some(playlist_uri.clone()),
                    group_id,
                    language: Some(stream_metadata.audio.language.clone()),
                    assoc_language: None,
                    name: stream_metadata.audio.title.clone(),
                    default: Some(i) == best_audio_stream_idx, // TODO: Verify this works
                    autoselect: Some(i) == best_audio_stream_idx, // TODO: Verify this works
                    forced: false,                             // TODO: Detect forced subtitles
                    instream_id: None,                         // We do not use in-stream captions
                    characteristics: None,
                    channels: stream_metadata.audio.channel_layout.description(),
                    other_attributes: None,
                };
                alternatives.push(alternative);

                let stream_playlist = MediaPlaylist {
                    version: hls_version,
                    target_duration,
                    media_sequence: 0,
                    segments: vec![], // TODO
                    discontinuity_sequence: 0,
                    end_list: true,
                    playlist_type: Some(MediaPlaylistType::Vod),
                    i_frames_only: false, // TODO: Set #EXT-X-I-FRAMES-ONLY to true after i-frame extraction is implemented for fast seeking
                    start: None, // This is typically only used if you client to start on a live edge or the beginning of a VOD stream
                    independent_segments: true, // Set to true because we assume segments are encoded to be independently decodable
                    unknown_tags: vec![],
                };
                playlists.push((playlist_uri, stream_playlist));
            }
            StreamMetadata::Subtitle(stream_metadata) => {
                let group_id = stream_metadata.codec_id.to_string(); // TODO: Verify this is correct
                let language = stream_metadata.language();
                let title = stream_metadata.title();
                let variant_name: String = title.as_ref().cloned().unwrap_or(
                    language
                        .as_ref()
                        .cloned()
                        .unwrap_or(stream_metadata.index.to_string()),
                );
                let playlist_uri = format!("subtitles/{variant_name}/index.m3u8");

                let alternative = AlternativeMedia {
                    media_type: m3u8_rs::AlternativeMediaType::Subtitles,
                    uri: Some(playlist_uri.clone()),
                    group_id,
                    language,
                    assoc_language: None,
                    name: title.unwrap_or_default(),
                    default: Some(i) == best_subtitle_stream_idx,
                    autoselect: true,  // TODO: See if any subtitle should use false
                    forced: false,     // TODO: Detect forced subtitles
                    instream_id: None, // We do not use in-stream captions
                    characteristics: None,
                    channels: None,
                    other_attributes: None,
                };
                alternatives.push(alternative);

                let stream_playlist = MediaPlaylist {
                    version: hls_version,
                    target_duration,
                    media_sequence: 0,
                    segments: vec![], // TODO
                    discontinuity_sequence: 0,
                    end_list: true,
                    playlist_type: Some(MediaPlaylistType::Vod),
                    i_frames_only: false, // TODO: Set #EXT-X-I-FRAMES-ONLY to true after i-frame extraction is implemented for fast seeking
                    start: None, // This is typically only used if you client to start on a live edge or the beginning of a VOD stream
                    independent_segments: true, // Set to true because we assume segments are encoded to be independently decodable
                    unknown_tags: vec![],
                };
                playlists.push((playlist_uri, stream_playlist));
            }
        }
    }

    let master_playlist = MasterPlaylist {
        version: hls_version,
        variants,
        session_data: vec![], // We store metadata elsewhere instead of using this tag
        session_key: vec![],
        start: None,
        independent_segments: true, // Set to true because we assume segments are encoded to be independently decodable
        alternatives,
        unknown_tags: vec![],
    };
    // TODO: Write tests for correctness
    // TODO: Write this somewhere...: The encoder generating the fMP4 segments should use a GOP that strictly divides the target duration (e.g. 2s)

    let mut master_playlist_v: Vec<u8> = Vec::new();
    master_playlist.write_to(&mut master_playlist_v).unwrap();
    let master_playlist_str: &str = std::str::from_utf8(&master_playlist_v).unwrap();
    // assert!(master_playlist_str.contains("#EXTINF:2.90000,title"));
    // TODO: Could add more assertions here
    println!("Master playlist...\n{master_playlist_str}\n");

    let stream_playlists: Vec<_> = playlists
        .into_iter()
        .map(|(playlist_uri, playlist)| {
            let mut v: Vec<u8> = Vec::new();
            playlist.write_to(&mut v).unwrap();
            (playlist_uri, std::str::from_utf8(&v).unwrap().to_string())
        })
        .collect();
    println!("Stream playlists...");
    for (uri, pl) in stream_playlists.iter() {
        println!("{uri}:\n{pl}\n");
    }

    Ok(())
}
