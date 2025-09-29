use axum::Router;

use crate::{media, task, utils};

use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

// Define the OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        task::list_tasks,
        task::get_task,
        task::search_tasks,
        task::create_task,
        task::delete_task,

        task::schedule::get_schedule,
        task::schedule::patch_schedule,

        media::get_media,
        media::get_related_media,
        media::get_media_history,
        // TODO: Add endpoints for /media/*
    ),
    components(
        schemas(
            task::Task,
            task::TaskType,
            task::TaskTrigger,
            task::TaskStatus,
            task::ScanType,
            task::CollectionScanTask,
            task::TaskError,
            task::TaskSearchQuery,
            task::TaskSearchQuerySort,
            task::CreateTask,
            task::CreateCollectionScanTask,

            task::schedule::TaskSchedule,
            task::schedule::TaskScheduleFrequency,
            task::schedule::TaskScheduleFrequencyUnit,
            task::schedule::CollectionScanTaskSchedule,
            task::schedule::UpdateTaskSchedule,
            task::schedule::UpdateCollectionScanTaskSchedule,

            media::MediaLibrary,
            media::MediaItem,
            media::MovieItem,
            media::MovieMetadata,
            media::SeriesItem,
            media::SeriesMetadata,
            media::MediaItemFilter,
            media::MediaItemSort,
            media::WatchHistory,
            media::Episode,
            media::WatchProgress,

            media::collection::MediaCollection,

            utils::SortDirection,
            utils::DateFilter,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "task", description = "Task Management API"),
        (name = "task::schedule", description = "Task Schedule API"),
        (name = "media", description = "Media Query API"),
        (name = "media::collection", description = "Media Collection API"),
        (name = "media::search", description = "Media Search API"),
    )
)]
struct ApiDoc;

// Define a security addon to add API key security scheme
// TODO: Modify for JWT
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("task_apikey"))),
            )
        }
    }
}

pub fn openapi_router() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
        // via SwaggerUi instead we only make rapidoc to point to the existing doc.
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    // Alternative to above
    // .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", ApiDoc::openapi()).path("/rapidoc"))
}
