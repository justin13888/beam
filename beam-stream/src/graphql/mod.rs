// use parking_lot::RwLock;

use std::sync::Arc;

use async_graphql::*;

use schema::*;

use sea_orm::DatabaseConnection;

use crate::{
    config::ServerConfig,
    graphql::schema::{
        library::{LibraryMutation, LibraryQuery},
        media::{MediaMutation, MediaQuery},
    },
    services::{
        auth::{AuthService, LocalAuthService},
        hash::{HashConfig, HashService, LocalHashService},
        library::{LibraryConfig, LibraryService, LocalLibraryService},
        metadata::{MetadataConfig, MetadataService, StubMetadataService},
        session_store::{RedisSessionStore, SessionStore},
        transcode::{LocalTranscodeService, TranscodeService},
    },
};

pub mod guard;
pub mod schema;

pub use guard::AuthGuard;

#[derive(Debug)]
pub struct AppState {
    pub config: ServerConfig,
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
    pub auth: Arc<dyn AuthService>,
    pub session_store: Arc<dyn SessionStore>,
    pub hash: Arc<dyn HashService>,
    pub library: Arc<dyn LibraryService>,
    pub metadata: Arc<dyn MetadataService>,
    pub transcode: Arc<dyn TranscodeService>,
}

impl AppServices {
    pub async fn new(config: &ServerConfig, db: DatabaseConnection) -> Self {
        let hash_config = HashConfig::default();
        let library_config = LibraryConfig {
            video_dir: config.video_dir.clone(),
        };
        let metadata_config = MetadataConfig {
            cache_dir: config.cache_dir.clone(),
        };

        // Create repository implementations
        let library_repo = Arc::new(crate::repositories::SqlLibraryRepository::new(db.clone()));
        let file_repo = Arc::new(crate::repositories::SqlFileRepository::new(db.clone()));
        let movie_repo = Arc::new(crate::repositories::SqlMovieRepository::new(db.clone()));
        let show_repo = Arc::new(crate::repositories::SqlShowRepository::new(db.clone()));
        let stream_repo = Arc::new(crate::repositories::SqlMediaStreamRepository::new(
            db.clone(),
        ));
        let user_repo = Arc::new(crate::repositories::SqlUserRepository::new(db.clone()));

        let hash_service = Arc::new(LocalHashService::new(hash_config));
        let media_info_service =
            Arc::new(crate::services::media_info::LocalMediaInfoService::default());
        let transcode_service = Arc::new(LocalTranscodeService::new(
            hash_service.clone(),
            media_info_service.clone(),
        ));

        // Initialize Redis session store
        let session_store = Arc::new(
            RedisSessionStore::new(&config.redis_url)
                .await
                .expect("Failed to connect to Redis"),
        );

        let auth_service = Arc::new(LocalAuthService::new(
            user_repo,
            session_store.clone(),
            config.clone(),
        ));

        Self {
            auth: auth_service,
            session_store,
            hash: hash_service.clone() as Arc<dyn HashService>,
            library: Arc::new(LocalLibraryService::new(
                library_repo,
                file_repo,
                movie_repo,
                show_repo,
                stream_repo,
                library_config,
                hash_service.clone(),
                media_info_service,
            )),
            metadata: Arc::new(StubMetadataService::new(metadata_config)),
            transcode: transcode_service,
        }
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub async fn create_schema(
    config: &ServerConfig,
    db: DatabaseConnection,
) -> (AppSchema, SharedAppState) {
    let app_state = AppState {
        config: config.clone(),
        services: AppServices::new(config, db).await,
    };
    let shared_app_state: SharedAppState = Arc::new(app_state);

    let schema = Schema::build(
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
    .data(shared_app_state.clone())
    .finish();

    (schema, shared_app_state)
}
