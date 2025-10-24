use async_graphql::*;

pub struct MediaMutation;

#[Object]
impl MediaMutation {
    /// Refresh media metadata by ID
    async fn refresh_metadata(&self, _ctx: &Context<'_>, id: ID) -> Result<bool> {
        todo!()
    }
}
