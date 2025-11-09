use async_graphql::*;

use crate::graphql::SharedAppState;
use crate::models::Library;

pub struct LibraryQuery;

#[Object]
impl LibraryQuery {
    /// Fetch list of all libraries
    async fn libraries(&self, ctx: &Context<'_>) -> Result<Vec<Library>> {
        let state = ctx.data::<SharedAppState>()?;
        let user_id = "user123"; // TODO: Replace with actual user ID from context/session
        let libraries = state.services.library.get_libraries(user_id.to_string())?;

        Ok(libraries)
    }
}
