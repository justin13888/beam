//! Export GraphQL schema to a file for code generation

use beam_stream::config::Config;
use beam_stream::graphql::create_schema;
use eyre::Result;
use std::fs;

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    // Load configuration (or use defaults)
    let config = Config::from_env()?;

    // Create the GraphQL schema
    let schema = create_schema(&config);

    // Export as SDL
    let sdl = schema.sdl();

    // Write to beam-stream directory
    let output_path = "schema.graphql";
    fs::write(output_path, sdl)?;

    println!("GraphQL schema exported to: beam-stream/{output_path}");

    Ok(())
}
