use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260126_000001_create_libraries::Libraries;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(IndexedFiles::Table)
                    .if_not_exists()
                    .col(
                        uuid(IndexedFiles::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(uuid(IndexedFiles::LibraryId).not_null())
                    .col(text(IndexedFiles::FilePath).not_null())
                    .col(string_len(IndexedFiles::FileHash, 32).not_null()) // XXH3 hash (128-bit = 32 hex chars)
                    .col(big_integer(IndexedFiles::FileSize).not_null())
                    .col(string_len_null(IndexedFiles::MimeType, 100))
                    .col(double_null(IndexedFiles::DurationSecs))
                    .col(
                        timestamp_with_time_zone(IndexedFiles::ScannedAt)
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_indexed_files_library")
                            .from(IndexedFiles::Table, IndexedFiles::LibraryId)
                            .to(Libraries::Table, Libraries::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint on (library_id, file_path)
        manager
            .create_index(
                Index::create()
                    .name("idx_indexed_files_library_path")
                    .table(IndexedFiles::Table)
                    .col(IndexedFiles::LibraryId)
                    .col(IndexedFiles::FilePath)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index for file hash lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_indexed_files_hash")
                    .table(IndexedFiles::Table)
                    .col(IndexedFiles::FileHash)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(IndexedFiles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum IndexedFiles {
    Table,
    Id,
    LibraryId,
    FilePath,
    FileHash,
    FileSize,
    MimeType,
    DurationSecs,
    ScannedAt,
}
