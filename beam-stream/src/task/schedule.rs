//! /task/schedule API

use axum::{extract::State, routing::get, Json, Router};

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utoipa::ToSchema;

type Store = Mutex<TaskSchedule>;

pub trait GenericTaskSchedule {
    fn is_enabled(&self) -> bool;
    fn frequency(&self) -> &TaskScheduleFrequency;
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct TaskScheduleFrequency {
    pub interval: u64,
    pub unit: TaskScheduleFrequencyUnit,
}

impl PartialOrd for TaskScheduleFrequency {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskScheduleFrequency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare value based on unit
        let self_value = match self.unit {
            TaskScheduleFrequencyUnit::Minutes => self.interval,
            TaskScheduleFrequencyUnit::Hours => self.interval * 60,
            TaskScheduleFrequencyUnit::Days => self.interval * 60 * 24,
        };

        let other_value = match other.unit {
            TaskScheduleFrequencyUnit::Minutes => other.interval,
            TaskScheduleFrequencyUnit::Hours => other.interval * 60,
            TaskScheduleFrequencyUnit::Days => other.interval * 60 * 24,
        };

        self_value.cmp(&other_value)
    }
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub enum TaskScheduleFrequencyUnit {
    #[serde(rename = "m")]
    Minutes,
    #[serde(rename = "h")]
    Hours,
    #[serde(rename = "d")]
    Days,
}

#[derive(
    Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Default,
)]
pub struct TaskSchedule {
    collection_scan: CollectionScanTaskSchedule,
}

/// Update Task Schedule item
#[derive(
    Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Default,
)]
pub struct UpdateTaskSchedule {
    collection_scan: Option<UpdateCollectionScanTaskSchedule>,
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct CollectionScanTaskSchedule {
    pub enabled: bool,
    pub frequency: TaskScheduleFrequency,
}

impl Default for CollectionScanTaskSchedule {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: TaskScheduleFrequency {
                interval: 1,
                unit: TaskScheduleFrequencyUnit::Days,
            },
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct UpdateCollectionScanTaskSchedule {
    pub enabled: Option<bool>,
    pub frequency: Option<TaskScheduleFrequency>,
}

impl GenericTaskSchedule for CollectionScanTaskSchedule {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn frequency(&self) -> &TaskScheduleFrequency {
        &self.frequency
    }
}

pub fn task_schedule_router() -> Router {
    let store = Arc::new(Store::default());

    axum::Router::new()
        .route("/", get(get_schedule).patch(patch_schedule))
        .with_state(store)
}

/// Get Task Schedule item from in-memory storage.
#[utoipa::path(
    get,
    path = "/task/schedule",
    responses(
        (status = 200, description = "Get task schedule successfully", body = [TaskSchedule])
    )
)]
async fn get_schedule(State(store): State<Arc<Store>>) -> Json<TaskSchedule> {
    let tasks = store.lock().await.clone();

    Json(tasks)
}

/// Update Task Schedule item from in-memory storage.
#[utoipa::path(
    patch,
    path = "/task/schedule",
    request_body = UpdateTaskSchedule,
    responses(
        (status = 200, description = "Updated task schedule successfully", body = [TaskSchedule])
    )
)]
async fn patch_schedule(
    State(store): State<Arc<Store>>,
    Json(update_schedule): Json<UpdateTaskSchedule>,
) -> Json<TaskSchedule> {
    let mut tasks = store.lock().await;

    if let Some(collection_scan) = update_schedule.collection_scan {
        if let Some(enabled) = collection_scan.enabled {
            tasks.collection_scan.enabled = enabled;
        }

        if let Some(frequency) = collection_scan.frequency {
            tasks.collection_scan.frequency = frequency;
        }
    }

    Json(tasks.clone())
}
