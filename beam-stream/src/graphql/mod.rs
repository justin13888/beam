// use parking_lot::RwLock;

use std::sync::Arc;

use async_graphql::*;

use schema::*;

use sea_orm::DatabaseConnection;

use crate::{
    config::Config,
    graphql::schema::{
        library::{LibraryMutation, LibraryQuery},
        media::{MediaMutation, MediaQuery},
    },
    services::{
        hash::{HashConfig, HashService, HashServiceImpl},
        library::{LibraryConfig, LibraryService, LibraryServiceImpl},
        metadata::{MetadataConfig, MetadataService, MetadataServiceImpl},
        transcode::{TranscodeService, TranscodeServiceImpl},
    },
};

pub mod guard;
pub mod schema;

pub use guard::AuthGuard;

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub services: AppServices,
}
pub type SharedAppState = Arc<AppState>;

#[derive(Clone, Debug)]
pub struct UserContext {
    pub user_id: String,
}

#[derive(Clone, Debug)]
pub struct AppContextInner {
    pub user_context: Option<UserContext>,
}
pub struct AppContext(Arc<AppContextInner>);

impl AppContext {
    pub fn new(user_context: Option<UserContext>) -> Self {
        Self(Arc::new(AppContextInner { user_context }))
    }

    pub fn user_context(&self) -> Option<UserContext> {
        self.0.user_context.clone()
    }
}

#[derive(Debug)]
pub struct AppServices {
    pub hash: Arc<dyn HashService>,
    pub library: Arc<dyn LibraryService>,
    pub metadata: Arc<dyn MetadataService>,
    pub transcode: Arc<dyn TranscodeService>,
}

impl AppServices {
    pub fn new(config: &Config, db: DatabaseConnection) -> Self {
        let hash_config = HashConfig::default();
        let library_config = LibraryConfig {
            video_dir: config.video_dir.clone(),
        };
        let metadata_config = MetadataConfig {
            cache_dir: config.cache_dir.clone(),
        };

        let hash_service = Arc::new(HashServiceImpl::new(hash_config));
        let transcode_service = Arc::new(TranscodeServiceImpl::new(
            hash_service.clone() as Arc<dyn HashService>
        ));
        Self {
            hash: hash_service.clone() as Arc<dyn HashService>,
            library: Arc::new(LibraryServiceImpl::new(
                db.clone(),
                library_config,
                hash_service.clone() as Arc<dyn HashService>,
            )),
            metadata: Arc::new(MetadataServiceImpl::new(metadata_config)),
            transcode: transcode_service,
        }
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(config: &Config, db: DatabaseConnection) -> AppSchema {
    let app_state = AppState {
        config: config.clone(),
        services: AppServices::new(config, db),
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
