use std::{
    cmp::Ordering,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::Local;
use itertools::Itertools;
use tokio::join;

use crate::{
    data::todoist::{
        todoist_command::TodoistCommand,
        todoist_response::{
            SyncStatus, TodoistGetAllCompletedItemResponse, TodoistResponse, TodoistSync,
        },
    },
    task::{CompletedTask, Task},
};

use super::todoist_data_store::TodoistDataStore;

pub async fn get_initial_tasks<T: Into<String>>(
    todoist_auth: T,
) -> (
    Vec<String>,
    HashMap<String, Task>,
    Vec<String>,
    HashMap<String, CompletedTask>,
    HashMap<String, Vec<String>>,
    String,
) {
    let token = todoist_auth.into();
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("sync_token", "*");
    params.insert("resource_types", "[\"all\"]");
    let sync = client
        .post("https://api.todoist.com/sync/v9/sync")
        .header("Authorization", format!("Bearer {}", &token))
        .form(&params);

    let completed_items = client
        .post("https://api.todoist.com/sync/v9/completed/get_all")
        .header("Authorization", format!("Bearer {}", &token))
        .form(&params);

    let (Ok(completed_items), Ok(sync)) = join!(completed_items.send(), sync.send()) else {
        panic!("A connection error occured");
    };

    let sync: TodoistSync = sync.json().await.unwrap();
    let completed_items: TodoistGetAllCompletedItemResponse = completed_items.json().await.unwrap();

    let mut subtasks: HashMap<String, Vec<(usize, String)>> = HashMap::new();
    let mut root_tasks = Vec::new();
    let tasks: HashMap<String, Task> = sync
        .items
        .unwrap_or_default()
        .into_iter()
        .map(|f| {
            if let Some(ref parent_id) = f.parent_id {
                let subtasks = subtasks.entry(parent_id.clone()).or_default();
                subtasks.push((f.child_order, f.id.clone()));
            } else {
                root_tasks.push((f.child_order, f.id.clone()));
            }
            (f.id.clone(), f.into())
        })
        .collect();

    let subtasks = subtasks
        .into_iter()
        .map(|(id, vec)| {
            (
                id,
                vec.into_iter()
                    .sorted_by_key(|x| x.0)
                    .map(|(_, subtask)| subtask)
                    .collect_vec(),
            )
        })
        .collect();
    let root_tasks = root_tasks
        .into_iter()
        .sorted_by_key(|x| x.0)
        .map(|(_, child)| child)
        .collect_vec();

    let completed_tasks: HashMap<String, CompletedTask> = completed_items
        .items
        .into_iter()
        .map(|f| (f.task_id.clone().expect("awf"), f.into()))
        .collect();

    let completed_root: Vec<String> = completed_tasks.keys().cloned().collect_vec();

    (
        root_tasks,
        tasks,
        completed_root,
        completed_tasks,
        subtasks,
        sync.sync_token,
    )
}

pub async fn handle_sync<T: Into<String>>(
    data_store: &mut TodoistDataStore,
    todoist_auth: T,
    sync_token: &str,
) {
    let token = todoist_auth.into();
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("sync_token", sync_token);
    params.insert("resource_types", "[\"all\"]");
    let sync = client
        .post("https://api.todoist.com/sync/v9/sync")
        .header("Authorization", format!("Bearer {}", &token))
        .form(&params);

    let todoist_sync: TodoistSync = sync.send().await.unwrap().json().await.unwrap();

    if let Some(items) = todoist_sync.items {
        for item in items.into_iter() {
            if let Some(task) = data_store.tasks.get_mut(item.id.as_str()) {
                *task = item.into();
            }
        }
    }
}

pub async fn sync<T: Into<String>>(todoist_auth: T) -> TodoistDataStore {
    println!("Attempting to connect to Todoist");

    let token = todoist_auth.into();

    let (root_tasks, tasks, completed_root, completed_tasks, subtasks, sync_token) =
        get_initial_tasks(&token).await;

    let (send, mut recv) = tokio::sync::mpsc::channel::<TodoistCommand>(100);
    let mutex = Arc::new(Mutex::new(false));
    let curr_syncing = mutex.clone();

    let mut previous_token = sync_token;

    tokio::spawn(async move {
        // FIXME: update the tasks here.
        let mut temp_id_mapping = HashMap::new();

        let mut buffer = Vec::with_capacity(100);

        let mut send_time = Local::now();

        while !recv.is_closed() {
            let size = recv.recv_many(&mut buffer, 100).await;
            if let Ok(mut currently_syncing) = curr_syncing.lock() {
                *currently_syncing = true;
            }

            let has_passed = send_time.cmp(&Local::now()) == Ordering::Less;

            if !has_passed {
                tokio::time::sleep(
                    send_time
                        .signed_duration_since(Local::now())
                        .to_std()
                        .expect("`send_time` is less than the current time when it should not be."),
                )
                .await;
            }

            send_time = Local::now()
                .checked_add_signed(chrono::Duration::milliseconds(500))
                .expect("Send time date is out of range");

            for command in buffer.iter_mut().take(size) {
                command.update_id(&temp_id_mapping);
            }

            if !buffer.is_empty() {
                let client = reqwest::Client::new();
                let mut params = HashMap::new();
                params.insert("commands", serde_json::to_string(&buffer[..size]).unwrap());
                let sync = client
                    .post("https://api.todoist.com/sync/v9/sync")
                    .header("Authorization", format!("Bearer {}", token))
                    .form(&params);

                let response = match sync.send().await {
                    Ok(response) => response,
                    Err(err) => {
                        tracing::error!("Could not send the command because: {}", err);

                        if let Ok(mut currently_syncing) = curr_syncing.lock() {
                            *currently_syncing = false;
                        }
                        continue;
                    }
                };

                let response = response.text().await.unwrap();

                match serde_json::from_str::<TodoistResponse>(&response) {
                    Ok(todoist_response) => {
                        temp_id_mapping.extend(todoist_response.temp_id_mapping.into_iter());

                        for (sync_status_id, response) in todoist_response.sync_status {
                            if let SyncStatus::Err(response) = response {
                                tracing::error!(
                                    "Got an error(id = {}): {:?}, body: {:?}",
                                    sync_status_id,
                                    response,
                                    serde_json::to_string(&buffer[..size]).unwrap()
                                );
                            }
                        }

                        let mut params = HashMap::new();
                        params.insert("sync_token", previous_token);
                        params.insert("resource_types", "[\"all\"]".to_string());
                        let sync = client
                            .post("https://api.todoist.com/sync/v9/sync")
                            .header("Authorization", format!("Bearer {}", &token))
                            .form(&params)
                            .send()
                            .await
                            .unwrap()
                            .text()
                            .await
                            .unwrap();

                        previous_token = todoist_response.sync_token;
                        tracing::info!("Got an sync request: {:?}", sync)
                    }
                    Err(e) => {
                        tracing::error!("{}", e.to_string() + "\n " + &response);
                    }
                }
            }

            if let Ok(mut currently_syncing) = curr_syncing.lock() {
                *currently_syncing = false;
            }

            buffer.clear();
        }
    });

    TodoistDataStore {
        root: root_tasks,
        tasks,
        completed_tasks,
        subtasks,
        completed_root,
        tags: HashMap::new(),
        task_count: 0,
        currently_syncing: mutex,
        command_sender: send,
    }
}
