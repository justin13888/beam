// Repository traits from beam-domain
pub use beam_domain::repositories::AdminLogRepository;
pub use beam_domain::repositories::FileRepository;
pub use beam_domain::repositories::LibraryRepository;
pub use beam_domain::repositories::MediaStreamRepository;
pub use beam_domain::repositories::MovieRepository;
pub use beam_domain::repositories::ShowRepository;

// SQL implementations from beam-index
pub use beam_index::repositories::SqlAdminLogRepository;
pub use beam_index::repositories::SqlFileRepository;
pub use beam_index::repositories::SqlLibraryRepository;
pub use beam_index::repositories::SqlMediaStreamRepository;
pub use beam_index::repositories::SqlMovieRepository;
pub use beam_index::repositories::SqlShowRepository;

// Sub-modules exposing traits, mocks, and in-memory impls for tests
#[cfg(any(test, feature = "test-utils"))]
pub mod admin_log {
    pub use beam_domain::repositories::AdminLogRepository;
    pub mod in_memory {
        pub use beam_domain::repositories::admin_log::in_memory::*;
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod file {
    pub use beam_domain::repositories::FileRepository;
    pub use beam_domain::repositories::file::MockFileRepository;
    pub mod in_memory {
        pub use beam_domain::repositories::file::in_memory::*;
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod library {
    pub use beam_domain::repositories::LibraryRepository;
    pub use beam_domain::repositories::library::MockLibraryRepository;
    pub mod in_memory {
        pub use beam_domain::repositories::library::in_memory::*;
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod movie {
    pub use beam_domain::repositories::MovieRepository;
    pub use beam_domain::repositories::movie::MockMovieRepository;
    pub mod in_memory {
        pub use beam_domain::repositories::movie::in_memory::*;
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod show {
    pub use beam_domain::repositories::ShowRepository;
    pub use beam_domain::repositories::show::MockShowRepository;
    pub mod in_memory {
        pub use beam_domain::repositories::show::in_memory::*;
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod stream {
    pub use beam_domain::repositories::MediaStreamRepository;
    pub use beam_domain::repositories::stream::MockMediaStreamRepository;
    pub mod in_memory {
        pub use beam_domain::repositories::stream::in_memory::*;
    }
}
