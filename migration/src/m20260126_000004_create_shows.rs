use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260126_000001_create_libraries::Libraries;
use crate::m20260126_000002_create_indexed_files::IndexedFiles;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Shows table
        manager
            .create_table(
                Table::create()
                    .table(Shows::Table)
                    .if_not_exists()
                    .col(
                        uuid(Shows::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(uuid(Shows::LibraryId).not_null())
                    .col(string_len(Shows::Title, 500).not_null())
                    .col(string_len_null(Shows::TitleLocalized, 500))
                    .col(ColumnDef::new(Shows::TitleAlternatives).array(ColumnType::Text).null())
                    .col(text_null(Shows::Description))
                    .col(integer_null(Shows::Year))
                    .col(string_len_null(Shows::ImdbId, 20))
                    .col(integer_null(Shows::TmdbId))
                    .col(integer_null(Shows::TvdbId))
                    .col(
                        timestamp_with_time_zone(Shows::CreatedAt)
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        timestamp_with_time_zone(Shows::UpdatedAt)
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_shows_library")
                            .from(Shows::Table, Shows::LibraryId)
                            .to(Libraries::Table, Libraries::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shows_library")
                    .table(Shows::Table)
                    .col(Shows::LibraryId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shows_title")
                    .table(Shows::Table)
                    .col(Shows::Title)
                    .to_owned(),
            )
            .await?;

        // Seasons table
        manager
            .create_table(
                Table::create()
                    .table(Seasons::Table)
                    .if_not_exists()
                    .col(
                        uuid(Seasons::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(uuid(Seasons::ShowId).not_null())
                    .col(integer(Seasons::SeasonNumber).not_null())
                    .col(date_null(Seasons::FirstAired))
                    .col(date_null(Seasons::LastAired))
                    .col(integer_null(Seasons::EpisodeRuntimeMins))
                    .col(text_null(Seasons::PosterUrl))
                    .col(ColumnDef::new(Seasons::Genres).array(ColumnType::Text).null())
                    .col(integer_null(Seasons::RatingTmdb))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_seasons_show")
                            .from(Seasons::Table, Seasons::ShowId)
                            .to(Shows::Table, Shows::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint on (show_id, season_number)
        manager
            .create_index(
                Index::create()
                    .name("idx_seasons_show_number")
                    .table(Seasons::Table)
                    .col(Seasons::ShowId)
                    .col(Seasons::SeasonNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Episodes table
        manager
            .create_table(
                Table::create()
                    .table(Episodes::Table)
                    .if_not_exists()
                    .col(
                        uuid(Episodes::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(uuid(Episodes::SeasonId).not_null())
                    .col(integer(Episodes::EpisodeNumber).not_null())
                    .col(string_len(Episodes::Title, 500).not_null())
                    .col(text_null(Episodes::Description))
                    .col(date_null(Episodes::AirDate))
                    .col(text_null(Episodes::ThumbnailUrl))
                    .col(double_null(Episodes::DurationSecs))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_episodes_season")
                            .from(Episodes::Table, Episodes::SeasonId)
                            .to(Seasons::Table, Seasons::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint on (season_id, episode_number)
        manager
            .create_index(
                Index::create()
                    .name("idx_episodes_season_number")
                    .table(Episodes::Table)
                    .col(Episodes::SeasonId)
                    .col(Episodes::EpisodeNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Episode files junction table
        manager
            .create_table(
                Table::create()
                    .table(EpisodeFiles::Table)
                    .if_not_exists()
                    .col(uuid(EpisodeFiles::EpisodeId).not_null())
                    .col(uuid(EpisodeFiles::FileId).not_null())
                    .col(boolean(EpisodeFiles::IsPrimary).not_null().default(false))
                    .primary_key(
                        Index::create()
                            .col(EpisodeFiles::EpisodeId)
                            .col(EpisodeFiles::FileId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_episode_files_episode")
                            .from(EpisodeFiles::Table, EpisodeFiles::EpisodeId)
                            .to(Episodes::Table, Episodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_episode_files_file")
                            .from(EpisodeFiles::Table, EpisodeFiles::FileId)
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
            .drop_table(Table::drop().table(EpisodeFiles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Episodes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Seasons::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Shows::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Shows {
    Table,
    Id,
    LibraryId,
    Title,
    TitleLocalized,
    TitleAlternatives,
    Description,
    Year,
    ImdbId,
    TmdbId,
    TvdbId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Seasons {
    Table,
    Id,
    ShowId,
    SeasonNumber,
    FirstAired,
    LastAired,
    EpisodeRuntimeMins,
    PosterUrl,
    Genres,
    RatingTmdb,
}

#[derive(DeriveIden)]
pub enum Episodes {
    Table,
    Id,
    SeasonId,
    EpisodeNumber,
    Title,
    Description,
    AirDate,
    ThumbnailUrl,
    DurationSecs,
}

#[derive(DeriveIden)]
pub enum EpisodeFiles {
    Table,
    EpisodeId,
    FileId,
    IsPrimary,
}
