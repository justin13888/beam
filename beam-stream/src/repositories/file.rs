use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DbErr};

use crate::models::domain::{CreateMediaFile, MediaFile};

/// Repository for managing media file persistence operations.
///
/// This trait defines the data access layer for media files (videos) within libraries.
/// It handles file metadata storage including paths, hashes, sizes, and associations
/// with movies or TV episodes.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FileRepository: Send + Sync + std::fmt::Debug {
    /// Finds a media file by its filesystem path.
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(MediaFile))` - If a file with the given path exists
    /// * `Ok(None)` - If no file with the given path exists
    /// * `Err(DbErr)` - If a database error occurs
    async fn find_by_path(&self, path: &str) -> Result<Option<MediaFile>, DbErr>;

    /// Creates a new media file record in the database.
    ///
    /// # Arguments
    ///
    /// * `create` - The parameters for creating the media file
    ///
    /// # Returns
    ///
    /// The newly created media file with its generated ID and timestamps,
    /// or a database error if creation fails.
    async fn create(&self, create: CreateMediaFile) -> Result<MediaFile, DbErr>;
}

/// SQL-based implementation of the FileRepository trait.
///
/// Uses SeaORM to interact with a relational database (PostgreSQL).
#[derive(Debug, Clone)]
pub struct SqlFileRepository {
    db: DatabaseConnection,
}

impl SqlFileRepository {
    /// Creates a new SQL file repository.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection to use for all operations
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FileRepository for SqlFileRepository {
    async fn find_by_path(&self, path: &str) -> Result<Option<MediaFile>, DbErr> {
        use crate::entities::files;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let model = files::Entity::find()
            .filter(files::Column::FilePath.eq(path))
            .one(&self.db)
            .await?;

        Ok(model.map(MediaFile::from))
    }

    async fn create(&self, create: CreateMediaFile) -> Result<MediaFile, DbErr> {
        use crate::entities::files;
        use crate::models::domain::MediaFileContent;
        use chrono::Utc;
        use sea_orm::{ActiveModelTrait, Set};

        let now = Utc::now();
        let (movie_entry_id, episode_id) = match create.content {
            MediaFileContent::Movie { movie_entry_id } => (Some(movie_entry_id), None),
            MediaFileContent::Episode { episode_id } => (None, Some(episode_id)),
        };

        let new_file = files::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            library_id: Set(create.library_id),
            file_path: Set(create.path.to_string_lossy().to_string()),
            hash_xxh3: Set(create.hash as i64),
            file_size: Set(create.size_bytes as i64),
            mime_type: Set(create.mime_type),
            duration_secs: Set(create.duration.map(|d| d.as_secs_f64())),
            container_format: Set(create.container_format),
            language: Set(None),
            quality: Set(None),
            release_group: Set(None),
            is_primary: Set(true),
            movie_entry_id: Set(movie_entry_id),
            episode_id: Set(episode_id),
            scanned_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        let result = new_file.insert(&self.db).await?;
        Ok(MediaFile::from(result))
    }
}
