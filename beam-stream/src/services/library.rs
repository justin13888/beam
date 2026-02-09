use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use thiserror::Error;
use tracing::{error, info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::entities::{
    episode, files, library, library_movie, library_show, media_stream, movie, movie_entry, season,
    show,
};
use crate::models::Library;
use crate::services::hash::HashService;
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
pub struct LibraryServiceImpl {
    db: DatabaseConnection,
    config: LibraryConfig,
    hash_service: Arc<dyn HashService>,
}

impl LibraryServiceImpl {
    pub fn new(
        db: DatabaseConnection,
        config: LibraryConfig,
        hash_service: Arc<dyn HashService>,
    ) -> Self {
        LibraryServiceImpl {
            db,
            config,
            hash_service,
        }
    }

    /// Helper to extract and insert media streams for a file
    async fn insert_media_streams(
        &self,
        file_id: Uuid,
        metadata: &VideoFileMetadata,
    ) -> Result<u32, LibraryError> {
        let mut inserted_count = 0;

        for stream in &metadata.streams {
            let (
                stream_type,
                codec,
                language,
                title,
                width,
                height,
                frame_rate,
                bit_rate,
                channels,
                sample_rate,
                channel_layout,
            ) = match stream {
                StreamMetadata::Video(v) => (
                    media_stream::StreamType::Video,
                    v.video.codec_name.clone(),
                    None,
                    None,
                    Some(v.video.width as i32),
                    Some(v.video.height as i32),
                    v.frame_rate(),
                    Some(v.video.bit_rate as i64),
                    None,
                    None,
                    None,
                ),
                StreamMetadata::Audio(a) => (
                    media_stream::StreamType::Audio,
                    a.audio.codec_name.clone(),
                    Some(a.audio.language.clone()).filter(|s| !s.is_empty()),
                    Some(a.audio.title.clone()).filter(|s| !s.is_empty()),
                    None,
                    None,
                    None,
                    Some(a.audio.bit_rate as i64),
                    Some(a.audio.channels as i32),
                    Some(a.audio.rate as i32),
                    Some(a.audio.channel_layout_description().to_string()),
                ),
                StreamMetadata::Subtitle(s) => (
                    media_stream::StreamType::Subtitle,
                    format!("{:?}", s.codec_id),
                    s.language(),
                    s.title(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            };

            let stream_model = media_stream::ActiveModel {
                id: Set(Uuid::new_v4()),
                file_id: Set(file_id),
                stream_index: Set(stream.index() as i32),
                stream_type: Set(stream_type),
                codec: Set(codec),
                language: Set(language),
                title: Set(title),
                is_default: Set(false),
                is_forced: Set(false),
                width: Set(width),
                height: Set(height),
                frame_rate: Set(frame_rate),
                bit_rate: Set(bit_rate),
                color_space: Set(None),
                color_range: Set(None),
                hdr_format: Set(None),
                channels: Set(channels),
                sample_rate: Set(sample_rate),
                channel_layout: Set(channel_layout),
            };

            stream_model.insert(&self.db).await?;
            inserted_count += 1;
        }

        Ok(inserted_count)
    }
}

#[async_trait::async_trait]
impl LibraryService for LibraryServiceImpl {
    /// Get all libraries by user ID
    /// Returns None if user is not found
    async fn get_libraries(&self, _user_id: String) -> Result<Vec<Library>, LibraryError> {
        let libraries = library::Entity::find().all(&self.db).await?;

        // TODO: This size calculation is N+1 and slow, should be a COUNT query or cached
        let mut result = Vec::new();
        for l in libraries {
            let size = files::Entity::find()
                .filter(files::Column::LibraryId.eq(l.id))
                .count(&self.db)
                .await?;

            result.push(Library {
                id: l.id.to_string(),
                name: l.name,
                description: l.description,
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
        let now = Utc::now();
        let new_library = library::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name),
            root_path: Set(root_path),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ..Default::default()
        };

        let result = new_library.insert(&self.db).await?;

        Ok(Library {
            id: result.id.to_string(),
            name: result.name,
            description: result.description,
            size: 0,
        })
    }

    /// Scan a library for new content
    async fn scan_library(&self, library_id: String) -> Result<u32, LibraryError> {
        let lib_uuid = Uuid::parse_str(&library_id).map_err(|_| LibraryError::InvalidId)?;

        // 1. Fetch Library
        let library = library::Entity::find_by_id(lib_uuid)
            .one(&self.db)
            .await?
            .ok_or(LibraryError::LibraryNotFound)?;

        info!("Scanning library: {} ({})", library.name, library.root_path);

        let root_path = Path::new(&library.root_path);
        if !root_path.exists() {
            return Err(LibraryError::PathNotFound(library.root_path));
        }

        // Regex for detecting SxxExx pattern
        let episode_regex = Regex::new(r"(?i)S(\d+)E(\d+)").unwrap();
        let mut added_count = 0;

        for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !["mp4", "mkv", "avi", "mov", "webm"].contains(&ext.to_lowercase().as_str()) {
                    continue;
                }
            } else {
                continue;
            }

            // Check if already indexed
            let path_str = path.to_string_lossy().to_string();
            let existing = files::Entity::find()
                .filter(files::Column::FilePath.eq(&path_str))
                .one(&self.db)
                .await?;

            if existing.is_some() {
                // file already parsed, skipping to re-scan for now
                // in future, could check modtime/size to see if re-scan needed
                continue;
            }

            info!("Processing file: {}", path_str);

            // Extract Metadata
            let metadata = match VideoFileMetadata::from_path(path) {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to extract metadata for {}: {}", path_str, e);
                    continue;
                }
            };

            // Calculate Hash
            let hash_value = match self.hash_service.hash_async(path.to_path_buf()).await {
                Ok(h) => h,
                Err(e) => {
                    error!("Failed to hash file {}: {}", path_str, e);
                    continue;
                }
            };

            // Insert File
            let file_id = Uuid::new_v4();
            let now = Utc::now();

            // NOTE: Polymorphic FKs will be set after determining if movie or episode
            let file_model = files::ActiveModel {
                id: Set(file_id),
                library_id: Set(lib_uuid),
                file_path: Set(path_str.clone()),
                hash_xxh3: Set(hash_value as i64), // Store XXH3 hash as BIGINT
                file_size: Set(metadata.file_size as i64),
                mime_type: Set(Some(format!("video/{}", metadata.format_name))),
                duration_secs: Set(Some(metadata.duration_seconds())),
                container_format: Set(Some(metadata.format_name.clone())),
                is_primary: Set(true),
                scanned_at: Set(now.into()),
                updated_at: Set(now.into()),
                // Polymorphic FKs: will be set below
                movie_entry_id: Set(None),
                episode_id: Set(None),
                ..Default::default()
            };

            //HEURISTIC: Check for TV Show pattern
            let file_stem = path
                .file_stem()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default();

            if let Some(captures) = episode_regex.captures(&file_stem) {
                // IT IS AN EPISODE
                let season_num: i32 = captures[1].parse().unwrap_or(1);
                let episode_num: i32 = captures[2].parse().unwrap_or(1);

                // Show title guess: Parent directory name
                let show_title = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown Show".to_string());

                // Find or Create Show
                let show = match show::Entity::find()
                    .filter(show::Column::Title.eq(&show_title))
                    .one(&self.db)
                    .await?
                {
                    Some(s) => s,
                    None => {
                        let new_show = show::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            title: Set(show_title.clone()),
                            created_at: Set(now.into()),
                            updated_at: Set(now.into()),
                            ..Default::default()
                        };
                        new_show.insert(&self.db).await?
                    }
                };

                // Ensure library-show association exists
                let lib_show_exists = library_show::Entity::find()
                    .filter(library_show::Column::LibraryId.eq(lib_uuid))
                    .filter(library_show::Column::ShowId.eq(show.id))
                    .one(&self.db)
                    .await?
                    .is_some();

                if !lib_show_exists {
                    let lib_show_link = library_show::ActiveModel {
                        library_id: Set(lib_uuid),
                        show_id: Set(show.id),
                    };
                    lib_show_link.insert(&self.db).await?;
                }

                // Find or Create Season
                let season = match season::Entity::find()
                    .filter(season::Column::ShowId.eq(show.id))
                    .filter(season::Column::SeasonNumber.eq(season_num))
                    .one(&self.db)
                    .await?
                {
                    Some(s) => s,
                    None => {
                        let new_season = season::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            show_id: Set(show.id),
                            season_number: Set(season_num),
                            ..Default::default()
                        };
                        new_season.insert(&self.db).await?
                    }
                };

                // Create Episode
                let episode_model = episode::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    season_id: Set(season.id),
                    episode_number: Set(episode_num),
                    title: Set(file_stem.to_string()), // Default title to filename
                    runtime_mins: Set(Some((metadata.duration_seconds() / 60.0) as i32)),
                    created_at: Set(now.into()),
                    ..Default::default()
                };
                let created_episode = episode_model.insert(&self.db).await?;

                // Update file with episode FK
                let mut file_active: files::ActiveModel = file_model.clone().into();
                file_active.episode_id = Set(Some(created_episode.id));
                let inserted_file = file_active.insert(&self.db).await?;

                // Extract and insert media streams
                self.insert_media_streams(inserted_file.id, &metadata)
                    .await?;
            } else {
                // IT IS A MOVIE
                // Title guess: Filename
                let movie_title = file_stem.to_string();

                // Find or Create Movie
                let movie = match movie::Entity::find()
                    .filter(movie::Column::Title.eq(&movie_title))
                    .one(&self.db)
                    .await?
                {
                    Some(m) => m,
                    None => {
                        let new_movie = movie::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            title: Set(movie_title),
                            runtime_mins: Set(Some((metadata.duration_seconds() / 60.0) as i32)),
                            created_at: Set(now.into()),
                            updated_at: Set(now.into()),
                            ..Default::default()
                        };
                        new_movie.insert(&self.db).await?
                    }
                };

                // Ensure library-movie association exists
                let lib_movie_exists = library_movie::Entity::find()
                    .filter(library_movie::Column::LibraryId.eq(lib_uuid))
                    .filter(library_movie::Column::MovieId.eq(movie.id))
                    .one(&self.db)
                    .await?
                    .is_some();

                if !lib_movie_exists {
                    let lib_movie_link = library_movie::ActiveModel {
                        library_id: Set(lib_uuid),
                        movie_id: Set(movie.id),
                    };
                    lib_movie_link.insert(&self.db).await?;
                }

                // Create movie_entry linking library and movie
                let movie_entry_model = movie_entry::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    library_id: Set(lib_uuid),
                    movie_id: Set(movie.id),
                    edition: Set(None),
                    is_primary: Set(true),
                    created_at: Set(now.into()),
                };
                let created_entry = movie_entry_model.insert(&self.db).await?;

                // Insert file with movie_entry FK
                let mut file_active: files::ActiveModel = file_model.into();
                file_active.movie_entry_id = Set(Some(created_entry.id));
                let inserted_file = file_active.insert(&self.db).await?;

                // Extract and insert media streams
                self.insert_media_streams(inserted_file.id, &metadata)
                    .await?;
            }

            added_count += 1;
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
