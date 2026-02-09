use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set, TransactionTrait,
};
use thiserror::Error;
use tracing::{error, info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::entities::{
    episode, episode_file, indexed_file, library, movie, movie_file, season, show,
};
use crate::models::Library;
use crate::services::hash::HashService;
use crate::utils::metadata::VideoFileMetadata;

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
            let size = indexed_file::Entity::find()
                .filter(indexed_file::Column::LibraryId.eq(l.id))
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
            media_type: Set("generic".to_string()), // Default
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
            let existing = indexed_file::Entity::find()
                .filter(indexed_file::Column::FilePath.eq(&path_str))
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
            let hash_str = format!("{:016x}", hash_value);

            // Start transaction for this file
            let txn = self.db.begin().await?;

            // Insert IndexedFile
            let file_id = Uuid::new_v4();
            let now = Utc::now();

            let indexed_file = indexed_file::ActiveModel {
                id: Set(file_id),
                library_id: Set(lib_uuid),
                file_path: Set(path_str.clone()),
                file_hash: Set(hash_str),
                file_size: Set(metadata.file_size as i64),
                mime_type: Set(Some(format!("video/{}", metadata.format_name))),
                duration_secs: Set(Some(metadata.duration_seconds())),
                scanned_at: Set(now.into()),
                ..Default::default()
            };

            let _ = indexed_file.insert(&txn).await?;

            // HEURISTIC: Check for TV Show pattern
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
                    .one(&txn)
                    .await?
                {
                    Some(s) => s,
                    None => {
                        let new_show = show::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            library_id: Set(lib_uuid),
                            title: Set(show_title.clone()),
                            created_at: Set(now.into()),
                            updated_at: Set(now.into()),
                            ..Default::default()
                        };
                        new_show.insert(&txn).await?
                    }
                };

                // Find or Create Season
                let season = match season::Entity::find()
                    .filter(season::Column::ShowId.eq(show.id))
                    .filter(season::Column::SeasonNumber.eq(season_num))
                    .one(&txn)
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
                        new_season.insert(&txn).await?
                    }
                };

                // Create Episode
                let episode = episode::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    season_id: Set(season.id),
                    episode_number: Set(episode_num),
                    title: Set(file_stem.to_string()), // Default title to filename
                    duration_secs: Set(Some(metadata.duration_seconds() as f64)),
                    // Episode table doesn't have created_at/updated_at
                    ..Default::default()
                };
                let created_episode = episode.insert(&txn).await?;

                // Link EpisodeFile
                let link = episode_file::ActiveModel {
                    episode_id: Set(created_episode.id),
                    file_id: Set(file_id),
                    is_primary: Set(true),
                };
                link.insert(&txn).await?;
            } else {
                // IT IS A MOVIE
                // Title guess: Filename
                let movie_title = file_stem.to_string();

                // Find or Create Movie
                let movie = match movie::Entity::find()
                    .filter(movie::Column::Title.eq(&movie_title))
                    .one(&txn)
                    .await?
                {
                    Some(m) => m,
                    None => {
                        let new_movie = movie::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            library_id: Set(lib_uuid),
                            title: Set(movie_title),
                            runtime_mins: Set(Some((metadata.duration_seconds() / 60.0) as i32)),
                            created_at: Set(now.into()),
                            updated_at: Set(now.into()),
                            ..Default::default()
                        };
                        new_movie.insert(&txn).await?
                    }
                };

                // Link MovieFile
                let link = movie_file::ActiveModel {
                    movie_id: Set(movie.id),
                    file_id: Set(file_id),
                    is_primary: Set(true),
                };
                link.insert(&txn).await?;
            }

            txn.commit().await?;
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
