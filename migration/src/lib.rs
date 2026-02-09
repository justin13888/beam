pub use sea_orm_migration::prelude::*;

mod m20260126_000001_create_libraries;
mod m20260126_000002_create_indexed_files;
mod m20260126_000003_create_movies;
mod m20260126_000004_create_shows;
mod m20260126_000005_create_stream_cache;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260126_000001_create_libraries::Migration),
            Box::new(m20260126_000002_create_indexed_files::Migration),
            Box::new(m20260126_000003_create_movies::Migration),
            Box::new(m20260126_000004_create_shows::Migration),
            Box::new(m20260126_000005_create_stream_cache::Migration),
        ]
    }
}
