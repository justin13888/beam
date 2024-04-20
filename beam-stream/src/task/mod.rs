//! /task API

pub mod schedule;

use std::{cmp::Ordering, sync::Arc};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::utils::SortDirection;

pub trait GenericTask {
    fn uuid(&self) -> &Uuid;
    fn timestamp(&self) -> &DateTime<Utc>;
    fn trigger(&self) -> &TaskTrigger;
    fn status(&self) -> &TaskStatus;
    fn description(&self) -> &str;
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TaskTrigger {
    Manual,
    Scheduled,
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Queued,
    Pending,
    Failed,
    Success,
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct CollectionScanTask {
    uuid: Uuid,
    timestamp: DateTime<Utc>,
    trigger: TaskTrigger,
    status: TaskStatus,
    description: String,
    relative_file_path: String,
    scan_type: ScanType,
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum ScanType {
    PreferFSMetadata,
    AlwaysHash,
}

impl GenericTask for CollectionScanTask {
    fn uuid(&self) -> &Uuid {
        &self.uuid
    }
    fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
    fn trigger(&self) -> &TaskTrigger {
        &self.trigger
    }
    fn status(&self) -> &TaskStatus {
        &self.status
    }
    fn description(&self) -> &str {
        &self.description
    }
}

pub fn task_router() -> Router {
    let store = Arc::new(Store::default());

    Router::new()
        .merge(
            Router::new()
                .route("/", get(list_tasks).post(create_task))
                .route("/search", post(search_tasks))
                .route("/:id", get(get_task).delete(delete_task))
                .with_state(store),
        )
        .nest("/schedule", schedule::task_schedule_router())
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
#[serde(tag = "type")]
pub enum Task {
    CollectionScan(CollectionScanTask),
}

impl GenericTask for Task {
    fn uuid(&self) -> &Uuid {
        match self {
            Task::CollectionScan(task) => task.uuid(),
        }
    }
    fn timestamp(&self) -> &DateTime<Utc> {
        match self {
            Task::CollectionScan(task) => task.timestamp(),
        }
    }
    fn trigger(&self) -> &TaskTrigger {
        match self {
            Task::CollectionScan(task) => task.trigger(),
        }
    }
    fn status(&self) -> &TaskStatus {
        match self {
            Task::CollectionScan(task) => task.status(),
        }
    }
    fn description(&self) -> &str {
        match self {
            Task::CollectionScan(task) => task.description(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum TaskType {
    CollectionScan,
}

/// In-memory task store
type Store = Mutex<Vec<Task>>;

/// Task operation errors
#[derive(Serialize, Deserialize, ToSchema)]
pub enum TaskError {
    /// Task already exists conflict
    #[schema(example = "Task already exists")]
    Conflict(String),
    /// Task not found by id
    #[schema(example = "id = 1")]
    NotFound(String),
}

/// List all Task items
#[utoipa::path(
    get,
    path = "/task",
    responses(
        (status = 200, description = "List all tasks successfully", body = [Task])
    )
)]
async fn list_tasks(State(store): State<Arc<Store>>) -> Json<Vec<Task>> {
    let tasks = store.lock().await.clone();

    Json(tasks)
}

/// Get Task item by id
/// Returns either 200 success of 404 with TaskError if Task is not found.
#[utoipa::path(
    get,
    path = "/task/{id}",
    responses(
        (status = 200, description = "Task found"),
        (status = 404, description = "Task not found", body = TaskError, example = json!(TaskError::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id" = i32, Path, description = "Task uuid")
    )
)]
async fn get_task(Path(id): Path<Uuid>, State(store): State<Arc<Store>>) -> impl IntoResponse {
    let tasks = store.lock().await;

    // Find task by id
    let result = tasks.iter().find(|&task| task.uuid() == &id);

    // Return response
    match result {
        Some(task) => (StatusCode::OK, Json(task)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(TaskError::NotFound(format!("id = {id}"))),
        )
            .into_response(),
    }
}

/// Task search query
#[derive(Deserialize, Serialize, ToSchema)]
pub struct TaskSearchQuery {
    /// Search by date range
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,

    /// Search by trigger type
    trigger: Option<Vec<TaskTrigger>>,
    // TODO: ^^ Validate that clients can't abuse with a bunch of duplicate values ^^
    /// Search by status
    status: Option<Vec<TaskStatus>>,
    // TODO: ^^ Validate that clients can't abuse with a bunch of duplicate values ^^

    // Sort by
    sort: Option<TaskSearchQuerySort>,
    // TODO: Implement cursor based pagination
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct TaskSearchQuerySort {
    uuid: Option<SortDirection>,
    timestamp: Option<SortDirection>,
    trigger: Option<SortDirection>,
    status: Option<SortDirection>,
}

/// Search Tasks by query params.
///
/// Search `Task`s by query params and return matching `Task`s.
#[utoipa::path(
    post,
    path = "/task/search",
    request_body = TaskSearchQuery,
    responses(
        (status = 200, description = "List matching tasks by query", body = [Task])
    )
)]
async fn search_tasks(
    State(store): State<Arc<Store>>,
    Json(query): Json<TaskSearchQuery>,
) -> Json<Vec<Task>> {
    let mut include_manual_triggers = true;
    let mut include_scheduled_triggers = true;
    let mut include_queued_tasks = true;
    let mut include_pending_tasks = true;
    let mut include_success_tasks = true;
    let mut include_failed_tasks = true;
    if let Some(triggers) = query.trigger {
        triggers.iter().for_each(|trigger| match trigger {
            TaskTrigger::Manual => include_scheduled_triggers = false,
            TaskTrigger::Scheduled => include_manual_triggers = false,
        });
    }
    if let Some(statuses) = query.status {
        statuses.iter().for_each(|status| match status {
            TaskStatus::Queued => include_queued_tasks = false,
            TaskStatus::Pending => include_pending_tasks = false,
            TaskStatus::Success => include_success_tasks = false,
            TaskStatus::Failed => include_failed_tasks = false,
        });
    }

    let mut results = store
        .lock()
        .await
        .iter()
        .filter(|task| {
            let mut matches = true;

            if let Some(start) = query.start {
                matches &= task.timestamp() >= &start;
            }
            if let Some(end) = query.end {
                matches &= task.timestamp() <= &end;
            }
            match task.trigger() {
                TaskTrigger::Manual => matches &= include_manual_triggers,
                TaskTrigger::Scheduled => matches &= include_scheduled_triggers,
            }
            match task.status() {
                TaskStatus::Queued => matches &= include_queued_tasks,
                TaskStatus::Pending => matches &= include_pending_tasks,
                TaskStatus::Success => matches &= include_success_tasks,
                TaskStatus::Failed => matches &= include_failed_tasks,
            }

            matches
        })
        .cloned()
        .collect::<Vec<_>>();

    if let Some(sort) = &query.sort {
        if sort.uuid.is_some() || sort.trigger.is_some() || sort.status.is_some() {
            results.sort_by(|a, b| {
                let mut cmp = Ordering::Equal;

                if let Some(sort) = &sort.uuid {
                    cmp = match sort {
                        SortDirection::Asc => a.uuid().cmp(b.uuid()),
                        SortDirection::Desc => b.uuid().cmp(a.uuid()),
                    };
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }
                if let Some(sort) = &sort.timestamp {
                    cmp = match sort {
                        SortDirection::Asc => a.timestamp().cmp(b.timestamp()),
                        SortDirection::Desc => b.timestamp().cmp(a.timestamp()),
                    };
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }
                if let Some(sort) = &sort.trigger {
                    cmp = match sort {
                        SortDirection::Asc => a.trigger().cmp(b.trigger()),
                        SortDirection::Desc => b.trigger().cmp(a.trigger()),
                    };
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }
                if let Some(sort) = &sort.status {
                    cmp = match sort {
                        SortDirection::Asc => a.status().cmp(b.status()),
                        SortDirection::Desc => b.status().cmp(a.status()),
                    };
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }

                cmp
            });
        }
    }

    Json(results)
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
#[serde(tag = "type")]
pub enum CreateTask {
    CollectionScan(CreateCollectionScanTask),
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct CreateCollectionScanTask {
    relative_file_path: String,
    scan_type: ScanType,
}

// TODO: Make it so tasks can't be created with a specific UUID and enforce that UUIDs are unique
/// Create new Task
///
/// Tries to create a new Task item to in-memory storage or fails with 409 conflict if already exists.
#[utoipa::path(
    post,
    path = "/task",
    request_body = CreateTask,
    responses(
        (status = 201, description = "Task item created successfully", body = Task),
        (status = 409, description = "Task already exists", body = TaskError)
    )
)]
async fn create_task(
    State(store): State<Arc<Store>>,
    Json(task): Json<CreateTask>,
) -> impl IntoResponse {
    let mut tasks = store.lock().await;

    tasks.push(match task {
        CreateTask::CollectionScan(task) => Task::CollectionScan(CollectionScanTask {
            uuid: Uuid::new_v4(),
            timestamp: Utc::now(),
            trigger: TaskTrigger::Manual,
            status: TaskStatus::Queued,
            description: String::new(),
            relative_file_path: task.relative_file_path,
            scan_type: task.scan_type,
        }),
    });
}

/// Delete Task item by id
///
/// Delete Task item from in-memory storage by id. Returns either 200 success of 404 with TaskError if Task is not found.
#[utoipa::path(
    delete,
    path = "/task/{id}",
    responses(
        (status = 200, description = "Task marked done successfully"),
        (status = 404, description = "Task not found", body = TaskError, example = json!(TaskError::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id" = i32, Path, description = "Task database id")
    )
)]
async fn delete_task(Path(id): Path<Uuid>, State(store): State<Arc<Store>>) -> impl IntoResponse {
    let mut tasks = store.lock().await;

    let len = tasks.len();

    tasks.retain(|task| task.uuid() != &id);

    if tasks.len() != len {
        StatusCode::OK.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(TaskError::NotFound(format!("id = {id}"))),
        )
            .into_response()
    }
}
