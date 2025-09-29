use ffmpeg_next as ffmpeg;

use std::{env, path::Path};

mod metadata;

fn main() -> Result<(), ffmpeg::Error> {
    ffmpeg::init().unwrap();

    let file_path = env::args().nth(1).expect("missing file");
    let metadata = metadata::extract_metadata(Path::new(&file_path))?;

    println!("=== FILE INFORMATION ===");
    println!(
        "Format: {} ({})",
        metadata.format_name, metadata.format_long_name
    );
    println!(
        "File size: {:.2} GB",
        metadata.file_size as f64 / 1_000_000_000.0
    );
    println!(
        "Duration: {:.2} seconds ({:.0}:{:02.0}:{:02.0})",
        metadata.duration_seconds(),
        metadata.duration_seconds() / 3600.0,
        (metadata.duration_seconds() % 3600.0) / 60.0,
        metadata.duration_seconds() % 60.0
    );
    println!(
        "Bit rate: {:.1} Mbps",
        metadata.bit_rate as f64 / 1_000_000.0
    );
    println!("Probe score: {}", metadata.probe_score);

    // Print file-level metadata
    if !metadata.metadata.is_empty() {
        println!("\n=== FILE METADATA ===");
        for (k, v) in &metadata.metadata {
            println!("{}: {}", k, v);
        }
    }

    // Print best streams
    println!("\n=== BEST STREAMS ===");
    if let Some(index) = metadata.best_video_stream {
        println!("Best video stream: {}", index);
    }

    if let Some(index) = metadata.best_audio_stream {
        println!("Best audio stream: {}", index);
    }

    if let Some(index) = metadata.best_subtitle_stream {
        println!("Best subtitle stream: {}", index);
    }

    // Print detailed stream information
    println!("\n=== STREAM DETAILS ===");
    for stream in &metadata.streams {
        println!("\nStream #{} ({:?}):", stream.index, stream.medium);
        println!("\tCodec: {:?}", stream.codec_id);
        println!("\tTime base: {}", stream.time_base);
        println!("\tStart time: {}", stream.start_time);
        let actual_duration = stream.actual_duration_seconds(metadata.duration_seconds());
        println!("\tDuration: {:.2} seconds", actual_duration);

        let actual_frames = stream.actual_frames();
        println!("\tFrames: {}", actual_frames);
        println!("\tFrame rate: {}", stream.rate);
        println!("\tDisposition: {:?}", stream.disposition);
        println!("\tDiscard: {:?}", stream.discard);

        // Print stream metadata
        if !stream.metadata.is_empty() {
            println!("\tStream Metadata:");
            for (k, v) in &stream.metadata {
                println!("\t\t{}: {}", k, v);
            }
        }

        if let Some(ref video) = stream.video {
            println!("\t=== VIDEO PROPERTIES ===");
            println!(
                "\tCodec: {} (Profile: {}, Level: {})",
                video.codec_name, video.profile, video.level
            );
            println!(
                "\tResolution: {}x{} (Aspect ratio: {})",
                video.width, video.height, video.aspect_ratio
            );
            println!("\tFrame rate: {:.3} fps", stream.frame_rate());
            println!(
                "\tPixel format: {:?} ({}-bit)",
                video.format, video.bit_depth
            );
            let actual_bitrate = video.actual_bit_rate(&stream.metadata);
            println!("\tBit rate: {:.1} Mbps", actual_bitrate / 1_000_000.0);
            println!("\tDelay: {}", video.delay);
            println!("\tB-frames: {}", video.has_b_frames);
            println!("\tColor space: {:?}", video.color_space);
            println!("\tColor range: {:?}", video.color_range);
            println!("\tColor primaries: {:?}", video.color_primaries);
            println!(
                "\tTransfer characteristic: {:?}",
                video.color_transfer_characteristic
            );
            println!("\tChroma location: {:?}", video.chroma_location);
            println!("\tReference frames: {}", video.references);
            println!("\tIntra DC precision: {}", video.intra_dc_precision);
        }

        if let Some(ref audio) = stream.audio {
            println!("\t=== AUDIO PROPERTIES ===");
            println!("\tCodec: {} (Profile: {})", audio.codec_name, audio.profile);
            if !audio.title.is_empty() {
                println!("\tTitle: {}", audio.title);
            }
            if !audio.language.is_empty() {
                println!("\tLanguage: {}", audio.language);
            }
            println!("\tSample rate: {} Hz", audio.rate);
            println!(
                "\tChannels: {} ({})",
                audio.channels,
                audio.channel_layout_description()
            );
            println!("\tSample format: {:?}", audio.format);
            let actual_bitrate = audio.actual_bit_rate(&stream.metadata);
            println!("\tBit rate: {:.1} kbps", actual_bitrate / 1000.0);
            println!("\tMax bit rate: {} kbps", audio.max_rate / 1000);
            let actual_audio_frames = audio.actual_frames(&stream.metadata);
            println!("\tFrames: {}", actual_audio_frames);
            println!("\tAlign: {}", audio.align);
            println!("\tDelay: {}", audio.delay);
        }
    }

    Ok(())
}
