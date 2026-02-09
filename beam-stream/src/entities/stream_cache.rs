//! Stream cache entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: Review and potentially edit this entity

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stream_cache")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub file_id: Uuid,
    #[sea_orm(column_type = "JsonBinary")]
    pub stream_config: serde_json::Value,
    pub cache_path: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::indexed_file::Entity",
        from = "Column::FileId",
        to = "super::indexed_file::Column::Id"
    )]
    IndexedFile,
}

impl Related<super::indexed_file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IndexedFile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
