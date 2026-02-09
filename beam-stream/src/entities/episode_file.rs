//! Episode file junction entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: Review and potentially edit this entity

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "episode_files")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub episode_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub file_id: Uuid,
    pub is_primary: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::episode::Entity",
        from = "Column::EpisodeId",
        to = "super::episode::Column::Id"
    )]
    Episode,
    #[sea_orm(
        belongs_to = "super::indexed_file::Entity",
        from = "Column::FileId",
        to = "super::indexed_file::Column::Id"
    )]
    IndexedFile,
}

impl Related<super::episode::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Episode.def()
    }
}

impl Related<super::indexed_file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IndexedFile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
