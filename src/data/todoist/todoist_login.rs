use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{data::todoist::todoist_command::TodoistCommand, task::Task};

use super::{todoist_data_store::TodoistDataStore, todoist_task::TodoistTask};

#[derive(serde::Deserialize, Debug)]
pub struct TodoistSync {
    pub items: Option<Vec<TodoistTask>>,
}

pub async fn sync<T: Into<String>>(todoist_auth: T) -> TodoistDataStore {
    let token = todoist_auth.into();
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("sync_token", "*");
    params.insert("resource_types", "[\"all\"]");
    let sync = client
        .post("https://api.todoist.com/sync/v9/sync")
        .header("Authorization", format!("Bearer {}", &token))
        .form(&params);

    let sync = sync
        .send()
        .await
        .unwrap()
        .json::<TodoistSync>()
        .await
        .unwrap();

    let mut subtasks: HashMap<String, Vec<String>> = HashMap::new();
    let mut root_tasks = Vec::new();
    let tasks: HashMap<String, Task> = sync
        .items
        .unwrap_or_default()
        .into_iter()
        .map(|f| {
            if let Some(ref parent_id) = f.parent_id {
                println!("ok");
                let subtasks = subtasks.entry(parent_id.clone()).or_default();
                subtasks.push(f.id.clone());
            } else {
                root_tasks.push(f.id.clone());
            }
            (f.id.clone(), f.into())
        })
        .collect();

    println!("{:?}", subtasks);
    println!("{:?}", tasks);

    // FIXME: use channels, we are required to do things sequentially.
    let (send, mut recv) = tokio::sync::mpsc::channel::<TodoistCommand>(10);
    let mutex = Arc::new(Mutex::new(false));

    TodoistDataStore {
        root: root_tasks,
        tasks,
        completed_tasks: HashMap::new(),
        subtasks,
        completed_root: Vec::new(),
        tags: HashMap::new(),
        task_count: 0,
        token,
        currently_syncing: mutex,
    }
}
