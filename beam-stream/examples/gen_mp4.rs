//! Example code to generate an MP4 file from a video file
//!
//! Usage:
//!   cargo run --example gen_mp4 -- <input_file> <output_file>
//!
//! Example:
//!   cargo run --example gen_mp4 -- input.mkv output.mp4

use beam_stream::utils::file::FileType;
use beam_stream::utils::stream::StreamBuilder;
use beam_stream::utils::stream::mp4::MP4StreamGenerator;
use eyre::Result;
use std::path::PathBuf;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();
    beam_stream::logging::init_tracing();

    // Get command-line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);

    if !input_path.exists() {
        eprintln!("Error: Input file does not exist: {:?}", input_path);
        std::process::exit(1);
    }

    println!("Input: {:?}", input_path);
    println!("Output: {:?}", output_path);
    println!();

    let start_time = Instant::now();

    // Initialize FFmpeg
    ffmpeg_next::init()?;

    // Build stream configuration
    println!("Building stream configuration...");
    let mut stream_builder = StreamBuilder::new();
    stream_builder.add_file(FileType::Video, &input_path);
    let stream_configuration = stream_builder.build().await?;

    println!("Stream configuration: {:#?}", stream_configuration);
    println!();

    // Create MP4 generator
    let mp4_generator = MP4StreamGenerator::from(stream_configuration);

    // Get metadata
    let metadata = mp4_generator.get_metadata();
    println!("MP4 Metadata: {:#?}", metadata);
    println!();

    // Generate MP4
    println!("Generating MP4 file...");
    mp4_generator.generate_mp4(&output_path).await?;
    println!("MP4 generation complete!");

    println!();
    println!("Output file: {:?}", output_path);
    println!(
        "File size: {} bytes",
        std::fs::metadata(&output_path)?.len()
    );
    println!("Total time: {:.2?}", start_time.elapsed());

    // Verify the output file can be opened
    println!();
    println!("Verifying output file...");
    let output_format = ffmpeg_next::format::input(&output_path)?;
    println!(
        "Duration: {:.2}s",
        output_format.duration() as f64 / 1_000_000.0
    );
    println!("Number of streams: {}", output_format.nb_streams());

    for stream in output_format.streams() {
        let codec = stream.parameters();
        println!(
            "  Stream #{}: {:?} ({:?})",
            stream.index(),
            codec.id(),
            stream.parameters().medium()
        );
    }

    println!();
    println!("✓ Successfully generated fragmented MP4 file!");
    println!("  The output is equivalent to running:");
    println!(
        "  ffmpeg -i {:?} -c copy -movflags frag_keyframe+empty_moov+default_base_moof -f mp4 {:?}",
        input_path, output_path
    );

    Ok(())
}
