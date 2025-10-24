use async_graphql::SimpleObject;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, ToSchema, SimpleObject)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl From<&beam_stream::utils::format::Resolution> for Resolution {
    fn from(res: &beam_stream::utils::format::Resolution) -> Self {
        Self {
            width: res.width,
            height: res.height,
        }
    }
}
