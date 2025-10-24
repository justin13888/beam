// use std::collections::HashMap;

use async_graphql::*;

use crate::graphql::schema::media::{MediaMutation, MediaQuery};

pub mod media;

pub struct QueryRoot {
    pub media: MediaQuery,
}

#[Object]
impl QueryRoot {
    async fn media(&self) -> &MediaQuery {
        &self.media
    }
}

pub struct MutationRoot {
    pub media: MediaMutation,
}

#[Object]
impl MutationRoot {
    async fn media(&self) -> &MediaMutation {
        &self.media
    }
}
