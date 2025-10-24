use async_graphql::*;
use tracing::info;

use crate::{graphql::SharedAppState, models::MediaMetadata};

pub struct MediaQuery;

#[Object]
impl MediaQuery {
    /// Fetch media metadata by ID
    async fn metadata(&self, ctx: &Context<'_>, id: ID) -> Result<Option<MediaMetadata>> {
        unimplemented!() // TODO
    }
}
