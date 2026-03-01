pub mod admin_log;
pub mod file;
pub mod library;
pub mod movie;
pub mod show;
pub mod stream;

// Re-export repository traits from beam-domain
pub use beam_domain::repositories::AdminLogRepository;
pub use beam_domain::repositories::FileRepository;
pub use beam_domain::repositories::LibraryRepository;
pub use beam_domain::repositories::MediaStreamRepository;
pub use beam_domain::repositories::MovieRepository;
pub use beam_domain::repositories::ShowRepository;

// Re-export SQL implementations
pub use admin_log::SqlAdminLogRepository;
pub use file::SqlFileRepository;
pub use library::SqlLibraryRepository;
pub use movie::SqlMovieRepository;
pub use show::SqlShowRepository;
pub use stream::SqlMediaStreamRepository;
