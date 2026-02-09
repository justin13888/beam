use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260126_000001_create_libraries::Libraries;
use crate::m20260126_000002_create_indexed_files::IndexedFiles;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Movies table
        manager
            .create_table(
                Table::create()
                    .table(Movies::Table)
                    .if_not_exists()
                    .col(
                        uuid(Movies::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(uuid(Movies::LibraryId).not_null())
                    .col(string_len(Movies::Title, 500).not_null())
                    .col(string_len_null(Movies::TitleLocalized, 500))
                    // Postgres array type for alternative titles
                    .col(ColumnDef::new(Movies::TitleAlternatives).array(ColumnType::Text).null())
                    .col(text_null(Movies::Description))
                    .col(integer_null(Movies::Year))
                    .col(date_null(Movies::ReleaseDate))
                    .col(integer_null(Movies::RuntimeMins))
                    .col(text_null(Movies::PosterUrl))
                    .col(text_null(Movies::BackdropUrl))
                    // Postgres array type for genres
                    .col(ColumnDef::new(Movies::Genres).array(ColumnType::Text).null())
                    .col(integer_null(Movies::RatingTmdb))
                    .col(string_len_null(Movies::ImdbId, 20))
                    .col(integer_null(Movies::TmdbId))
                    .col(integer_null(Movies::TvdbId))
                    .col(
                        timestamp_with_time_zone(Movies::CreatedAt)
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        timestamp_with_time_zone(Movies::UpdatedAt)
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_movies_library")
                            .from(Movies::Table, Movies::LibraryId)
                            .to(Libraries::Table, Libraries::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index on library_id for filtering by library
        manager
            .create_index(
                Index::create()
                    .name("idx_movies_library")
                    .table(Movies::Table)
                    .col(Movies::LibraryId)
                    .to_owned(),
            )
            .await?;

        // Index on title for search
        manager
            .create_index(
                Index::create()
                    .name("idx_movies_title")
                    .table(Movies::Table)
                    .col(Movies::Title)
                    .to_owned(),
            )
            .await?;

        // Movie files junction table
        manager
            .create_table(
                Table::create()
                    .table(MovieFiles::Table)
                    .if_not_exists()
                    .col(uuid(MovieFiles::MovieId).not_null())
                    .col(uuid(MovieFiles::FileId).not_null())
                    .col(boolean(MovieFiles::IsPrimary).not_null().default(false))
                    .primary_key(
                        Index::create()
                            .col(MovieFiles::MovieId)
                            .col(MovieFiles::FileId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_movie_files_movie")
                            .from(MovieFiles::Table, MovieFiles::MovieId)
                            .to(Movies::Table, Movies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_movie_files_file")
                            .from(MovieFiles::Table, MovieFiles::FileId)
                            .to(IndexedFiles::Table, IndexedFiles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MovieFiles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Movies::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Movies {
    Table,
    Id,
    LibraryId,
    Title,
    TitleLocalized,
    TitleAlternatives,
    Description,
    Year,
    ReleaseDate,
    RuntimeMins,
    PosterUrl,
    BackdropUrl,
    Genres,
    RatingTmdb,
    ImdbId,
    TmdbId,
    TvdbId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum MovieFiles {
    Table,
    MovieId,
    FileId,
    IsPrimary,
}
