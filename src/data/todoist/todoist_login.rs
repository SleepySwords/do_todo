use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    data::todoist::{todoist_command::TodoistCommand, todoist_response::TodoistResponse},
    task::Task,
};

use super::{todoist_data_store::TodoistDataStore, todoist_task::TodoistTask};

#[derive(serde::Deserialize, Debug)]
pub struct TodoistSync {
    pub items: Option<Vec<TodoistTask>>,
    pub completed_info: Option<Vec<CompletedInfo>>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum CompletedInfo {
    ProjectCompletedInfo {
        project_id: String,
        completed_items: usize,
        archived_sections: usize,
    },
    SectionCompletedInfo {
        section_id: String,
        completed_items: usize,
    },
    ItemCompletedInfo {
        item_id: String,
        completed_items: usize,
    },
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
    println!("{:?}", sync.completed_info);

    // FIXME: use channels, we are required to do things sequentially.
    let (send, mut recv) = tokio::sync::mpsc::channel::<TodoistCommand>(100);
    let mutex = Arc::new(Mutex::new(false));
    let curr_syncing = mutex.clone();

    let clone_token = token.clone();
    tokio::spawn(async move {
        // FIXME: update the tasks here.
        // FIXME: add batching support (recv_many)
        let mut temp_id_mapping = HashMap::new();

        while let Some(mut command) = recv.recv().await {
            if let Ok(mut currently_syncing) = curr_syncing.lock() {
                *currently_syncing = true;
            }

            command.update_id(&temp_id_mapping);

            let client = reqwest::Client::new();
            let mut params = HashMap::new();
            params.insert("commands", serde_json::to_string(&vec![command]).unwrap());
            let sync = client
                .post("https://api.todoist.com/sync/v9/sync")
                .header("Authorization", format!("Bearer {}", clone_token))
                .form(&params);

            let response = sync
                .send()
                .await
                .unwrap()
                .json::<TodoistResponse>()
                .await
                .unwrap(); // FIXME: ew unwraps here!

            temp_id_mapping.extend(response.temp_id_mapping.into_iter());

            if let Ok(mut currently_syncing) = curr_syncing.lock() {
                *currently_syncing = false;
            }
        }
    });

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
        command_sender: send,
    }
}
