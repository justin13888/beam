//! Indexed file entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: Review and potentially edit this entity

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "indexed_files")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub library_id: Uuid,
    pub file_path: String,
    pub file_hash: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub duration_secs: Option<f64>,
    pub scanned_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::library::Entity",
        from = "Column::LibraryId",
        to = "super::library::Column::Id"
    )]
    Library,
    #[sea_orm(has_many = "super::movie_file::Entity")]
    MovieFiles,
    #[sea_orm(has_many = "super::episode_file::Entity")]
    EpisodeFiles,
    #[sea_orm(has_many = "super::stream_cache::Entity")]
    StreamCaches,
}

impl Related<super::library::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Library.def()
    }
}

impl Related<super::movie_file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MovieFiles.def()
    }
}

impl Related<super::episode_file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EpisodeFiles.def()
    }
}

impl Related<super::stream_cache::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StreamCaches.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
