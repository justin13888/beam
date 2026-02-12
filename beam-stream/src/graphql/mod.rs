// use parking_lot::RwLock;

use async_graphql::*;

use schema::*;

use crate::{
    graphql::schema::{
        library::{LibraryMutation, LibraryQuery},
        media::{MediaMutation, MediaQuery},
    },
    state::AppState,
};

pub mod guard;
pub mod schema;

pub use guard::AuthGuard;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(state: AppState) -> AppSchema {
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
    .data(state)
    .finish()
}
