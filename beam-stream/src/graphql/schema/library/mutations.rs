use async_graphql::*;

use crate::graphql::{AuthGuard, SharedAppState};
use crate::models::Library;
use crate::services::metadata::MediaFilter;

pub struct LibraryMutation;

#[Object]
impl LibraryMutation {
    /// Refresh metadata for media library
    #[graphql(guard = "AuthGuard")]
    async fn refresh_metadata(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let state = ctx.data::<SharedAppState>()?;
        state
            .services
            .metadata
            .refresh_metadata(MediaFilter::ByLibraryId(id.to_string()))
            .await?;
        Ok(true)
    }

    /// Create a new library
    #[graphql(guard = "AuthGuard")]
    async fn create_library(
        &self,
        ctx: &Context<'_>,
        name: String,
        root_path: String,
    ) -> Result<Library> {
        let state = ctx.data::<SharedAppState>()?;
        let library = state
            .services
            .library
            .create_library(name, root_path)
            .await?;
        Ok(library)
    }
    /// Scan a library for new content
    #[graphql(guard = "AuthGuard")]
    async fn scan_library(&self, ctx: &Context<'_>, id: ID) -> Result<u32> {
        let state = ctx.data::<SharedAppState>()?;
        let count = state.services.library.scan_library(id.to_string()).await?;
        Ok(count)
    }
}
