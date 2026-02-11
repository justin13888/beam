use async_graphql::SimpleObject;
use salvo::oapi::ToSchema;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, ToSchema, SimpleObject)]
pub struct Library {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// Number of items in the library
    pub size: u32,
}
