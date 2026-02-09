//! Library entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: Review and potentially edit this entity

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "libraries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub root_path: String,
    pub media_type: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::indexed_file::Entity")]
    IndexedFiles,
    #[sea_orm(has_many = "super::movie::Entity")]
    Movies,
    #[sea_orm(has_many = "super::show::Entity")]
    Shows,
}

impl Related<super::indexed_file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IndexedFiles.def()
    }
}

impl Related<super::movie::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Movies.def()
    }
}

impl Related<super::show::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Shows.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
