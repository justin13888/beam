use serde::Serialize;
use utoipa::ToSchema;

mod movie;
mod show;

pub use movie::*;
pub use show::*;

/// Media metadata
#[derive(Serialize, ToSchema)]
pub enum MediaMetadata {
    Show(ShowMetadata),
    Movie(MovieMetadata),
}

#[derive(Serialize, ToSchema)]
pub struct Title {
    /// Original title
    pub original: String,
    /// Localized title, if available and different from original
    pub localized: Option<String>,
    /// Alternative titles, if any
    pub alternatives: Option<Vec<String>>,
}

#[derive(Serialize, ToSchema)]
pub struct ExternalIdentifiers {
    /// IMDb ID (e.g., tt1234567)
    pub imdb_id: Option<String>,
    /// TMDb ID (e.g., 12345)
    pub tmdb_id: Option<u32>,
    /// TVDb ID (e.g., 12345)
    pub tvdb_id: Option<u32>,
}

#[derive(Serialize, ToSchema)]
pub struct Ratings {
    /// TMDB rating as a percentage (0-100)
    pub tmdb: Option<u32>,
    // TODO: Add more ratings sources if needed
}
