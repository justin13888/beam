use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260126_000002_create_indexed_files::IndexedFiles;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StreamCache::Table)
                    .if_not_exists()
                    .col(
                        uuid(StreamCache::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(uuid(StreamCache::FileId).not_null())
                    // Postgres JSONB for stream configuration
                    .col(ColumnDef::new(StreamCache::StreamConfig).json_binary().not_null())
                    .col(text(StreamCache::CachePath).not_null())
                    .col(
                        timestamp_with_time_zone(StreamCache::CreatedAt)
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stream_cache_file")
                            .from(StreamCache::Table, StreamCache::FileId)
                            .to(IndexedFiles::Table, IndexedFiles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index on file_id for lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_stream_cache_file")
                    .table(StreamCache::Table)
                    .col(StreamCache::FileId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StreamCache::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum StreamCache {
    Table,
    Id,
    FileId,
    StreamConfig,
    CachePath,
    CreatedAt,
}
