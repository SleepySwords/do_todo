use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use itertools::Itertools;

use crate::{
    data::todoist::{todoist_command::TodoistCommand, todoist_response::TodoistResponse},
    task::Task,
};

use super::{todoist_data_store::TodoistDataStore, todoist_task::TodoistItem};

#[derive(serde::Deserialize, Debug)]
pub struct TodoistSync {
    pub items: Option<Vec<TodoistItem>>,
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
    println!("{:?}", sync.items);
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
                let position = root_tasks
                    .iter()
                    .position(|(child_order, _)| *child_order > f.child_order)
                    .unwrap_or(root_tasks.len());

                root_tasks.insert(position, (f.child_order, f.id.clone()));
            }
            (f.id.clone(), f.into())
        })
        .collect();

    let root_tasks = root_tasks.into_iter().map(|(_, child)| child).collect_vec();

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

            let response = sync.send().await.unwrap();

            // let text = response.text().await.unwrap();
            // println!("{}", text);

            let todoist_response = response.json::<TodoistResponse>().await.unwrap(); // FIXME: ew unwraps here!

            // let todoist_response = serde_json::from_str::<TodoistResponse>(&text).unwrap();

            temp_id_mapping.extend(todoist_response.temp_id_mapping.into_iter());

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
        currently_syncing: mutex,
        command_sender: send,
    }
}
