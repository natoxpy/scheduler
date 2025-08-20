use chrono::{DateTime, Utc};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskRecord {
    pub origin_name: String,
    pub origin_group: String,
    pub time: Duration,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleTask {
    pub origin_name: String,
    pub origin_group: String,
    pub time: Duration,
}

impl From<ScheduleTask> for TaskRecord {
    fn from(value: ScheduleTask) -> Self {
        Self {
            origin_name: value.origin_name.clone(),
            origin_group: value.origin_group.clone(),
            time: value.time.clone(),
        }
    }
}

impl From<TaskRecord> for ScheduleTask {
    fn from(value: TaskRecord) -> Self {
        Self {
            origin_name: value.origin_name.clone(),
            origin_group: value.origin_group.clone(),
            time: value.time.clone(),
        }
    }
}

// Tasks are things that can be added onto an
// schedule during the generation progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub group: String,
    pub config: TaskConfiguration,
    pub created: DateTime<Utc>,
    pub closed: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(name: &'static str, group: &'static str, config: TaskConfiguration) -> Self {
        Self {
            name: String::from(name),
            group: String::from(group),
            config,
            created: Utc::now(),
            closed: None,
        }
    }
}

// Here is where the things that will tell the
// scheduler how to deal with this task
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, Default)]
pub struct TaskConfiguration {
    #[derivative(Default(value = "Duration::from_secs(60*45)"))]
    pub time: Duration,
    #[derivative(Default(value = "true"))]
    pub repeat: bool,
}
