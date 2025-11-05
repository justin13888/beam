// use std::collections::HashMap;

use async_graphql::*;

use library::{LibraryMutation, LibraryQuery};
use media::{MediaMutation, MediaQuery};

pub mod library;
pub mod media;

pub struct QueryRoot {
    pub library: LibraryQuery,
    pub media: MediaQuery,
}

#[Object]
impl QueryRoot {
    async fn library(&self) -> &LibraryQuery {
        &self.library
    }

    async fn media(&self) -> &MediaQuery {
        &self.media
    }
}

pub struct MutationRoot {
    pub library: LibraryMutation,
    pub media: MediaMutation,
}

#[Object]
impl MutationRoot {
    async fn library(&self) -> &LibraryMutation {
        &self.library
    }

    async fn media(&self) -> &MediaMutation {
        &self.media
    }
}
