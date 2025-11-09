// use parking_lot::RwLock;

use std::sync::Arc;

use async_graphql::*;

use schema::*;

use crate::{
    config::Config,
    graphql::schema::{
        library::{LibraryMutation, LibraryQuery},
        media::{MediaMutation, MediaQuery},
    },
    services::{hash::HashService, library::LibraryService, metadata::MetadataService},
};

pub mod schema;

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub services: AppServices,
}
pub type SharedAppState = Arc<AppState>;

#[derive(Debug)]
pub struct AppServices {
    pub hash: HashService,
    pub library: LibraryService,
    pub metadata: MetadataService,
}

impl AppServices {
    pub fn new(config: &Config) -> Self {
        // TODO: Make sure services aren't automatically initialized statically. it should be done here.
        Self {
            hash: HashService::new(),
            library: LibraryService::new(),
            metadata: MetadataService::new(),
        }
    }
}

pub fn create_schema(config: &Config) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    let app_state = AppState {
        config: config.clone(),
        services: AppServices::new(config),
    };
    let shared_app_state: SharedAppState = Arc::new(app_state);

    Schema::build(
        QueryRoot {
            library: LibraryQuery,
            media: MediaQuery,
        },
        MutationRoot {
            library: LibraryMutation,
            media: MediaMutation,
        },
        EmptySubscription,
    )
    .data(shared_app_state)
    .finish()
}
