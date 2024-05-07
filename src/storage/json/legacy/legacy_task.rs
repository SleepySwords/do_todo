use chrono::{NaiveDate, NaiveDateTime};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use std::collections::HashMap;

use crate::{
    data::json_data_store::JsonDataStore,
    task::{CompletedTask, Priority, Tag, Task},
};

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

fn add_to_task(
    tasks: &mut HashMap<String, Task>,
    subtasks: &mut HashMap<String, Vec<String>>,
    name: &LegacyTask,
    parent_id: usize,
    id_gen: &mut usize,
) {
    *id_gen += 1;
    if let Some(subtask) = subtasks.get_mut(&parent_id.to_string()) {
        subtask.push((*id_gen).to_string());
    } else {
        subtasks.insert(parent_id.to_string(), vec![(*id_gen).to_string()]);
    }
    tasks.insert(id_gen.to_string(), (*name).clone().into());
    let curr = *id_gen;
    for subtask in &name.sub_tasks {
        add_to_task(tasks, subtasks, subtask, curr, id_gen);
    }
}

impl From<LegacyTaskStore> for JsonDataStore {
    fn from(value: LegacyTaskStore) -> Self {
        let mut tasks = HashMap::new();
        let mut completed_tasks: HashMap<String, CompletedTask> = HashMap::new();
        let mut subtasks = HashMap::new();
        let mut id_gen = 0;

        let roots = value
            .tasks
            .into_iter()
            .map(|f| {
                let curr_id = id_gen;
                for subtask in &f.sub_tasks {
                    add_to_task(&mut tasks, &mut subtasks, subtask, curr_id, &mut id_gen);
                }
                id_gen += 1;
                tasks.insert(curr_id.to_string(), f.into());
                curr_id.to_string()
            })
            .collect_vec();

        let completed_root = value
            .completed_tasks
            .into_iter()
            .map(|f| {
                id_gen += 1;
                completed_tasks.insert(id_gen.to_string(), (f).into());
                id_gen.to_string()
            })
            .collect_vec();

        JsonDataStore {
            tags: value
                .tags
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
            tasks,
            completed_tasks,
            subtasks,
            root: roots,
            completed_root,
            task_count: id_gen,
        }
    }
}
