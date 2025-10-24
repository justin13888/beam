// use parking_lot::RwLock;

use std::sync::Arc;

use async_graphql::*;

use schema::*;

use crate::{
    config::Config,
    graphql::schema::media::{MediaMutation, MediaQuery},
};
use beam_stream::services::metadata::METADATA_SERVICE;
use beam_stream::services::{
    hash::{HASH_SERVICE, HashService},
    metadata::MetadataService,
};

pub mod schema;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<Config>,
    pub services: AppServices,
}
pub type SharedAppState = Arc<AppState>;

#[derive(Clone, Debug)]
pub struct AppServices {
    pub hash: Arc<HashService>,
    pub metadata: Arc<MetadataService>,
}

impl AppServices {
    pub fn new(config: &Config) -> Self {
        Self {
            hash: HASH_SERVICE.clone(),
            metadata: METADATA_SERVICE.clone(),
        }
    }
}

pub fn create_schema(config: &Config) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    let app_state = AppState {
        config: Arc::new(config.clone()),
        services: AppServices::new(&config),
    };

    Schema::build(
        QueryRoot { media: MediaQuery },
        MutationRoot {
            media: MediaMutation,
        },
        EmptySubscription,
    )
    .data(app_state)
    .finish()
}
