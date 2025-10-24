// use parking_lot::RwLock;

use std::sync::Arc;

use async_graphql::*;

use beam_stream::config::Config;
use schema::*;

use crate::graphql::schema::media::{MediaMutation, MediaQuery};

pub mod schema;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<Config>,
}
pub type SharedAppState = Arc<AppState>;

pub fn create_schema(app_state: SharedAppState) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
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
