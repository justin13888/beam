//! Database entity modules
//!
//! These entities map to the database tables created by migrations.

pub mod episode;
pub mod episode_file;
pub mod indexed_file;
pub mod library;
pub mod movie;
pub mod movie_file;
pub mod season;
pub mod show;
pub mod stream_cache;

pub use episode::Entity as Episode;
pub use episode_file::Entity as EpisodeFile;
pub use indexed_file::Entity as IndexedFile;
pub use library::Entity as Library;
pub use movie::Entity as Movie;
pub use movie_file::Entity as MovieFile;
pub use season::Entity as Season;
pub use show::Entity as Show;
pub use stream_cache::Entity as StreamCache;
