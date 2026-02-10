use std::sync::Arc;

use regex::Regex;
use sea_orm::DbErr;
use thiserror::Error;
use tracing::{error, info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::models::Library;
use crate::services::hash::HashService;
use crate::services::media_info::MediaInfoService;
use crate::utils::metadata::{StreamMetadata, VideoFileMetadata};

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LibraryConfig {
    pub video_dir: PathBuf,
}

#[async_trait::async_trait]
pub trait LibraryService: Send + Sync + std::fmt::Debug {
    /// Get all libraries by user ID
    /// Returns None if user is not found
    async fn get_libraries(&self, user_id: String) -> Result<Vec<Library>, LibraryError>;

    /// Create a new library
    async fn create_library(
        &self,
        name: String,
        root_path: String,
    ) -> Result<Library, LibraryError>;

    /// Scan a library for new content
    async fn scan_library(&self, library_id: String) -> Result<u32, LibraryError>;
}

#[derive(Debug)]
pub struct LocalLibraryService {
    library_repo: Arc<dyn crate::repositories::LibraryRepository>,
    file_repo: Arc<dyn crate::repositories::FileRepository>,
    movie_repo: Arc<dyn crate::repositories::MovieRepository>,
    show_repo: Arc<dyn crate::repositories::ShowRepository>,
    stream_repo: Arc<dyn crate::repositories::MediaStreamRepository>,
    config: LibraryConfig,
    hash_service: Arc<dyn HashService>,
    media_info_service: Arc<dyn MediaInfoService>,
}

impl LocalLibraryService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        library_repo: Arc<dyn crate::repositories::LibraryRepository>,
        file_repo: Arc<dyn crate::repositories::FileRepository>,
        movie_repo: Arc<dyn crate::repositories::MovieRepository>,
        show_repo: Arc<dyn crate::repositories::ShowRepository>,
        stream_repo: Arc<dyn crate::repositories::MediaStreamRepository>,
        config: LibraryConfig,
        hash_service: Arc<dyn HashService>,
        media_info_service: Arc<dyn MediaInfoService>,
    ) -> Self {
        LocalLibraryService {
            library_repo,
            file_repo,
            movie_repo,
            show_repo,
            stream_repo,
            config,
            hash_service,
            media_info_service,
        }
    }

    /// Helper to extract and insert media streams for a file
    async fn insert_media_streams(
        &self,
        file_id: Uuid,
        metadata: &VideoFileMetadata,
    ) -> Result<u32, LibraryError> {
        use crate::models::domain::stream::{
            AudioStreamMetadata, SubtitleStreamMetadata, VideoStreamMetadata,
        };
        use crate::models::domain::{
            CreateMediaStream, StreamMetadata as DomainStreamMetadata, StreamType,
        };

        let mut streams_to_insert = Vec::new();

        for stream in &metadata.streams {
            let (stream_metadata, stream_type) = match stream {
                StreamMetadata::Video(v) => {
                    let metadata = DomainStreamMetadata::Video(VideoStreamMetadata {
                        width: v.video.width,
                        height: v.video.height,
                        frame_rate: v.frame_rate(),
                        bit_rate: Some(v.video.bit_rate as u64),
                        color_space: None,
                        color_range: None,
                        hdr_format: None,
                    });
                    (metadata, StreamType::Video)
                }
                StreamMetadata::Audio(a) => {
                    let metadata = DomainStreamMetadata::Audio(AudioStreamMetadata {
                        language: Some(a.audio.language.clone()).filter(|s| !s.is_empty()),
                        title: Some(a.audio.title.clone()).filter(|s| !s.is_empty()),
                        channels: a.audio.channels,
                        sample_rate: a.audio.rate,
                        channel_layout: Some(a.audio.channel_layout_description().to_string()),
                        bit_rate: Some(a.audio.bit_rate as u64),
                        is_default: false,
                        is_forced: false,
                    });
                    (metadata, StreamType::Audio)
                }
                StreamMetadata::Subtitle(s) => {
                    let metadata = DomainStreamMetadata::Subtitle(SubtitleStreamMetadata {
                        language: s.language(),
                        title: s.title(),
                        is_default: false,
                        is_forced: false,
                    });
                    (metadata, StreamType::Subtitle)
                }
            };

            streams_to_insert.push(CreateMediaStream {
                file_id,
                index: stream.index() as u32,
                stream_type,
                codec: match stream {
                    StreamMetadata::Video(v) => v.video.codec_name.clone(),
                    StreamMetadata::Audio(a) => a.audio.codec_name.clone(),
                    StreamMetadata::Subtitle(s) => format!("{:?}", s.codec_id),
                },
                metadata: stream_metadata,
            });
        }

        let count = self.stream_repo.insert_streams(streams_to_insert).await?;
        Ok(count as u32)
    }

    /// Process a single file to add it to the library
    async fn process_file(
        &self,
        path: &std::path::Path,
        lib_uuid: Uuid,
    ) -> Result<bool, LibraryError> {
        use crate::models::domain::{
            CreateEpisode, CreateMediaFile, CreateMovie, CreateMovieEntry, MediaFileContent,
        };
        use std::time::Duration;

        // Check if already indexed
        if self
            .file_repo
            .find_by_path(path.to_string_lossy().as_ref())
            .await?
            .is_some()
        {
            return Ok(false);
        }

        info!("Processing file: {}", path.display());

        // Extract Metadata
        let metadata = self
            .media_info_service
            .get_video_metadata(path)
            .await
            .map_err(|e| {
                warn!("Failed to extract metadata for {}: {}", path.display(), e);
                LibraryError::PathNotFound(format!("Metadata extraction failed: {}", e)) // TODO: Better error
            })?;

        // Calculate Hash
        let hash_value = self
            .hash_service
            .hash_async(path.to_path_buf())
            .await
            .map_err(|e| {
                error!("Failed to hash file {}: {}", path.display(), e);
                LibraryError::PathNotFound(format!("Hash failed: {}", e)) // TODO: Better error
            })?;

        // Regex for detecting SxxExx pattern
        let episode_regex = Regex::new(r"(?i)S(\d+)E(\d+)").unwrap();

        // Determine file type and content
        let file_stem = path
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();

        let content = if let Some(captures) = episode_regex.captures(&file_stem) {
            // IT IS AN EPISODE
            let season_num: u32 = captures[1].parse().unwrap_or(1);
            let episode_num: i32 = captures[2].parse().unwrap_or(1);

            // Show title guess: Parent directory name
            let show_title = path
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown Show".to_string());

            // Find or create show using repository
            let show = match self.show_repo.find_by_title(&show_title).await? {
                Some(s) => s,
                None => self.show_repo.create(show_title.clone()).await?,
            };

            // Ensure library-show association exists
            self.show_repo
                .ensure_library_association(lib_uuid, show.id)
                .await?;

            // Find or create season
            let season = self
                .show_repo
                .find_or_create_season(show.id, season_num)
                .await?;

            // Create episode
            let create_episode = CreateEpisode {
                season_id: season.id,
                episode_number: episode_num as u32,
                title: file_stem.to_string(),
                runtime: Some(Duration::from_secs_f64(metadata.duration_seconds())),
            };
            let episode = self.show_repo.create_episode(create_episode).await?;

            MediaFileContent::Episode {
                episode_id: episode.id,
            }
        } else {
            // IT IS A MOVIE
            let movie_title = file_stem.to_string();

            // Find or create movie using repository
            let movie = match self.movie_repo.find_by_title(&movie_title).await? {
                Some(m) => m,
                None => {
                    let create_movie = CreateMovie {
                        title: movie_title,
                        runtime: Some(Duration::from_secs_f64(metadata.duration_seconds())),
                    };
                    self.movie_repo.create(create_movie).await?
                }
            };

            // Ensure library-movie association exists
            self.movie_repo
                .ensure_library_association(lib_uuid, movie.id)
                .await?;

            // Create movie entry
            let create_entry = CreateMovieEntry {
                library_id: lib_uuid,
                movie_id: movie.id,
                edition: None,
                is_primary: true,
            };
            let entry = self.movie_repo.create_entry(create_entry).await?;

            MediaFileContent::Movie {
                movie_entry_id: entry.id,
            }
        };

        // Create media file using repository
        let create_file = CreateMediaFile {
            library_id: lib_uuid,
            path: path.to_path_buf(),
            hash: hash_value,
            size_bytes: metadata.file_size,
            mime_type: Some(format!("video/{}", metadata.format_name)),
            duration: Some(Duration::from_secs_f64(metadata.duration_seconds())),
            container_format: Some(metadata.format_name.clone()),
            content,
        };

        let file = self.file_repo.create(create_file).await?;

        // Extract and insert media streams
        self.insert_media_streams(file.id, &metadata).await?;

        Ok(true)
    }
}

#[async_trait::async_trait]
impl LibraryService for LocalLibraryService {
    /// Get all libraries by user ID
    /// Returns None if user is not found
    async fn get_libraries(&self, _user_id: String) -> Result<Vec<Library>, LibraryError> {
        let domain_libraries = self.library_repo.find_all().await?;

        // Convert domain models to GraphQL models
        let mut result = Vec::new();
        for lib in domain_libraries {
            let size = self.library_repo.count_files(lib.id).await?;

            result.push(Library {
                id: lib.id.to_string(),
                name: lib.name,
                description: lib.description,
                size: size as u32,
            });
        }

        Ok(result)
    }

    /// Create a new library
    async fn create_library(
        &self,
        name: String,
        root_path: String,
    ) -> Result<Library, LibraryError> {
        use crate::models::domain::CreateLibrary;
        use std::path::PathBuf;

        let create = CreateLibrary {
            name: name.clone(),
            root_path: PathBuf::from(&root_path),
            description: None,
        };

        let domain_library = self.library_repo.create(create).await?;

        Ok(Library {
            id: domain_library.id.to_string(),
            name: domain_library.name,
            description: domain_library.description,
            size: 0,
        })
    }

    /// Scan a library for new content
    async fn scan_library(&self, library_id: String) -> Result<u32, LibraryError> {
        use crate::models::domain::{
            CreateEpisode, CreateMediaFile, CreateMovie, CreateMovieEntry, MediaFileContent,
        };
        use std::time::Duration;

        let lib_uuid = Uuid::parse_str(&library_id).map_err(|_| LibraryError::InvalidId)?;

        // Fetch Library
        let library = self
            .library_repo
            .find_by_id(lib_uuid)
            .await?
            .ok_or(LibraryError::LibraryNotFound)?;

        info!(
            "Scanning library: {} ({:?})",
            library.name, library.root_path
        );

        if !library.root_path.exists() {
            return Err(LibraryError::PathNotFound(
                library.root_path.to_string_lossy().to_string(),
            ));
        }

        // Regex for detecting SxxExx pattern
        let episode_regex = Regex::new(r"(?i)S(\d+)E(\d+)").unwrap();
        let mut added_count = 0;

        for entry in WalkDir::new(&library.root_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // TODO: Review these file extensions
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !["mp4", "mkv", "avi", "mov", "webm"].contains(&ext.to_lowercase().as_str()) {
                    continue;
                }
            } else {
                continue;
            }

            match self.process_file(path, lib_uuid).await {
                Ok(true) => added_count += 1,
                Ok(false) => {}
                Err(e) => error!("Failed to process file {}: {}", path.display(), e),
            }
        }

        Ok(added_count)
    }
}

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    Db(#[from] DbErr),

    #[error("Library not found")]
    LibraryNotFound,
    #[error("Invalid Library ID")]
    InvalidId,
    #[error("Path not found: {0}")]
    PathNotFound(String),
}

#[cfg(test)]
#[path = "library_tests.rs"]
mod library_tests;
