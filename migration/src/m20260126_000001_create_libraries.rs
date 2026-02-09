use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Use Postgres-specific gen_random_uuid() for UUID generation
        manager
            .create_table(
                Table::create()
                    .table(Libraries::Table)
                    .if_not_exists()
                    .col(
                        uuid(Libraries::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(string_len(Libraries::Name, 255).not_null())
                    .col(text_null(Libraries::Description))
                    .col(text(Libraries::RootPath).not_null())
                    .col(
                        string_len(Libraries::MediaType, 20)
                            .not_null()
                            .check(Expr::col(Libraries::MediaType).is_in(["movies", "shows", "mixed"])),
                    )
                    .col(
                        timestamp_with_time_zone(Libraries::CreatedAt)
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        timestamp_with_time_zone(Libraries::UpdatedAt)
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on root_path for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_libraries_root_path")
                    .table(Libraries::Table)
                    .col(Libraries::RootPath)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Libraries::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Libraries {
    Table,
    Id,
    Name,
    Description,
    RootPath,
    MediaType,
    CreatedAt,
    UpdatedAt,
}
