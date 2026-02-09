use async_graphql::*;

use crate::graphql::{AppContext, AuthGuard, SharedAppState};
use crate::models::Library;

pub struct LibraryQuery;

#[Object]
impl LibraryQuery {
    /// Fetch list of all libraries
    #[graphql(guard = "AuthGuard")]
    async fn libraries(&self, ctx: &Context<'_>) -> Result<Vec<Library>> {
        let state = ctx.data::<SharedAppState>()?;
        let items = ctx.data::<AppContext>()?;
        let user_ctx = items
            .user_context()
            .ok_or_else(|| Error::new("Unauthorized"))?;

        let libraries = state
            .services
            .library
            .get_libraries(user_ctx.user_id.clone())
            .await?;

        Ok(libraries)
    }
}
