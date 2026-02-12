use crate::state::AppContext;
use async_graphql::{Context, Guard, Result};

pub struct AuthGuard;

impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let app_ctx = ctx.data::<AppContext>().map_err(|_| "AppContext missing")?;
        if app_ctx.user_context().is_some() {
            Ok(())
        } else {
            Err("Unauthorized".into())
        }
    }
}
