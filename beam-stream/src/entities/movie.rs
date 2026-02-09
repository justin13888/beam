//! Movie entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: Review and potentially edit this entity

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "movies")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub library_id: Uuid,
    pub title: String,
    pub title_localized: Option<String>,
    pub title_alternatives: Option<Vec<String>>,
    pub description: Option<String>,
    pub year: Option<i32>,
    pub release_date: Option<Date>,
    pub runtime_mins: Option<i32>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub genres: Option<Vec<String>>,
    pub rating_tmdb: Option<i32>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i32>,
    pub tvdb_id: Option<i32>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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

impl ActiveModelBehavior for ActiveModel {}
