use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use std::collections::HashMap;

use crate::task::{CompletedTask, Priority, Tag, Task, TaskStore};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LegacyTask {
    pub progress: bool,
    pub title: String,
    pub priority: Priority,
    pub tags: Vec<usize>,
    pub due_date: Option<NaiveDate>,

    // Ignored if sub_tasks is empty
    pub opened: bool,
    pub sub_tasks: Vec<LegacyTask>,
}

#[derive(Deserialize, Serialize)]
pub struct LegacyCompletedTask {
    pub task: LegacyTask,
    pub time_completed: NaiveDateTime,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LegacyTaskStore {
    pub tags: HashMap<usize, Tag>,
    pub tasks: Vec<LegacyTask>,
    pub completed_tasks: Vec<LegacyCompletedTask>,
    pub auto_sort: bool,
}

impl From<LegacyTask> for Task {
    fn from(t: LegacyTask) -> Self {
        Task {
            progress: t.progress,
            title: t.title,
            priority: t.priority,
            tags: t.tags.into_iter().map(|f| f.to_string()).collect(),
            due_date: t.due_date,
            opened: t.opened,
            sub_tasks: t.sub_tasks.into_iter().map(|f| f.into()).collect(),
        }
    }
}

impl From<LegacyCompletedTask> for CompletedTask {
    fn from(t: LegacyCompletedTask) -> Self {
        CompletedTask {
            task: t.task.into(),
            time_completed: t.time_completed,
        }
    }
}

impl From<LegacyTaskStore> for TaskStore {
    fn from(value: LegacyTaskStore) -> Self {
        TaskStore {
            tags: value
                .tags
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
            tasks: value.tasks.into_iter().map(|t| t.into()).collect(),
            completed_tasks: value
                .completed_tasks
                .into_iter()
                .map(|t| t.into())
                .collect(),
            auto_sort: value.auto_sort,
        }
    }
}
