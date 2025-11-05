use async_graphql::*;
use beam_stream::services::metadata::MediaFilter;

use crate::graphql::SharedAppState;

pub struct LibraryMutation;

#[Object]
impl LibraryMutation {
    /// Refresh metadata for media library
    async fn refresh_metadata(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let state = ctx.data::<SharedAppState>()?;
        state
            .services
            .metadata
            .refresh_metadata(MediaFilter::ByLibraryId(id.to_string()))?;
        Ok(true)
    }
}
