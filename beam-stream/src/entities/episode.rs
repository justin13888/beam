//! Episode entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: Review and potentially edit this entity

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "episodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub season_id: Uuid,
    pub episode_number: i32,
    pub title: String,
    pub description: Option<String>,
    pub air_date: Option<Date>,
    pub thumbnail_url: Option<String>,
    pub duration_secs: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::season::Entity",
        from = "Column::SeasonId",
        to = "super::season::Column::Id"
    )]
    Season,
    #[sea_orm(has_many = "super::episode_file::Entity")]
    EpisodeFiles,
}

impl Related<super::season::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Season.def()
    }
}

impl Related<super::episode_file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EpisodeFiles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
