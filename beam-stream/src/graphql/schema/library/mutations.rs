use async_graphql::*;

use crate::graphql::SharedAppState;
use crate::services::metadata::MediaFilter;

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
