use thiserror::Error;

use crate::models::Library;

#[derive(Debug)]
pub struct LibraryService {
    // /// File ID to Stream ID
    // file_to_stream: HashMap<String, String>,
    // /// Stream Metadata store
    // stream_metadata: HashMap<String, MediaMetadata>,
    // /// Stream ID to Media ID
    // stream_to_media: HashMap<String, String>,
    // /// Media Metadata store
    // media_metadata: HashMap<String, MediaMetadata>,
}

impl LibraryService {
    pub fn new() -> Self {
        // Scan metadata to build initial index

        // Initialize
        LibraryService {}
    }

    /// Get all libraries by user ID
    /// Returns None if user is not found
    pub fn get_libraries(&self, _user_id: String) -> Result<Vec<Library>, FetchLibrariesError> {
        // TODO: Implement for real
        Ok(vec![
            Library {
                id: "library1".to_string(),
                name: "My Movie Library".to_string(),
                description: Some("A collection of my favorite movies.".to_string()),
                size: 0,
            },
            Library {
                id: "library2".to_string(),
                name: "My TV Shows".to_string(),
                description: Some("All my binge-worthy TV shows.".to_string()),
                size: 0,
            },
        ])
    }
}

#[derive(Debug, Error)]
pub enum FetchLibrariesError {
    #[error("User not found")]
    UserNotFound,
}
