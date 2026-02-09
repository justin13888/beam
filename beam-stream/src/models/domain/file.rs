use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

/// Represents a media file in the library
#[derive(Debug, Clone)]
pub struct MediaFile {
    pub id: Uuid,
    pub library_id: Uuid,
    pub path: PathBuf,
    pub hash: u64,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub duration: Option<Duration>,
    pub container_format: Option<String>,
    pub content: MediaFileContent,
    pub scanned_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The content type of a media file
#[derive(Debug, Clone)]
pub enum MediaFileContent {
    /// File is a movie
    Movie { movie_entry_id: Uuid },
    /// File is a TV episode
    Episode { episode_id: Uuid },
}

/// Parameters for creating a new media file
#[derive(Debug, Clone)]
pub struct CreateMediaFile {
    pub library_id: Uuid,
    pub path: PathBuf,
    pub hash: u64,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub duration: Option<Duration>,
    pub container_format: Option<String>,
    pub content: MediaFileContent,
}

impl From<crate::entities::files::Model> for MediaFile {
    fn from(model: crate::entities::files::Model) -> Self {
        // Determine content type from polymorphic foreign keys
        let content = if let Some(movie_entry_id) = model.movie_entry_id {
            MediaFileContent::Movie { movie_entry_id }
        } else if let Some(episode_id) = model.episode_id {
            MediaFileContent::Episode { episode_id }
        } else {
            // Default to movie if neither is set (shouldn't happen in well-formed data)
            panic!("MediaFile must have either movie_entry_id or episode_id set");
        };

        Self {
            id: model.id,
            library_id: model.library_id,
            path: PathBuf::from(model.file_path),
            hash: model.hash_xxh3 as u64,
            size_bytes: model.file_size as u64,
            mime_type: model.mime_type,
            duration: model.duration_secs.map(Duration::from_secs_f64),
            container_format: model.container_format,
            content,
            scanned_at: model.scanned_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }
}
