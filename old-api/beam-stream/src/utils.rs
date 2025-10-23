use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct DateFilter {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}
