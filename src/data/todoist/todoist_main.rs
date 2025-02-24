use std::{
    cmp::Ordering,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::{Local, NaiveTime};
use itertools::Itertools;
use tokio::sync::mpsc::Sender;

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

pub async fn sync<T: Into<String>>(
    todoist_auth: T,
    log_sender: Sender<(String, NaiveTime)>,
) -> TodoistDataStore {
    println!("Attempting to connect to Todoist");
    let token = todoist_auth.into();
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("sync_token", "*");
    params.insert("resource_types", "[\"all\"]");
    let sync = client
        .post("https://api.todoist.com/sync/v9/sync")
        .header("Authorization", format!("Bearer {}", &token))
        .form(&params);

    let sync: TodoistSync = sync.send().await.unwrap().json().await.unwrap();

    let mut subtasks: HashMap<String, Vec<(usize, String)>> = HashMap::new();
    let mut root_tasks = Vec::new();
    let tasks: HashMap<String, Task> = sync
        .items
        .unwrap_or_default()
        .into_iter()
        .map(|f| {
            // FIXME: decide on one insertion sort, or quicksort after.
            if let Some(ref parent_id) = f.parent_id {
                let subtasks = subtasks.entry(parent_id.clone()).or_default();
                subtasks.push((f.child_order, f.id.clone()));
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
    let root_tasks = root_tasks.into_iter().map(|(_, child)| child).collect_vec();

    let completed_items = client
        .post("https://api.todoist.com/sync/v9/completed/get_all")
        .header("Authorization", format!("Bearer {}", &token))
        .form(&params);

    let completed_items: TodoistGetAllCompletedItemResponse =
        completed_items.send().await.unwrap().json().await.unwrap();

    let completed_tasks: HashMap<String, CompletedTask> = completed_items
        .items
        .into_iter()
        .map(|f| (f.id.clone(), f.into()))
        .collect();

    let completed_root: Vec<String> = completed_tasks.keys().map(|f| f.clone()).collect_vec();

    let (send, mut recv) = tokio::sync::mpsc::channel::<TodoistCommand>(100);
    let mutex = Arc::new(Mutex::new(false));
    let curr_syncing = mutex.clone();

    let clone_token = token.clone();
    tokio::spawn(async move {
        // FIXME: update the tasks here.
        let mut temp_id_mapping = HashMap::new();

        let mut buffer = Vec::with_capacity(100);

        let mut previous_time = Local::now();

        while !recv.is_closed() {
            let size = recv.recv_many(&mut buffer, 100).await;
            if let Ok(mut currently_syncing) = curr_syncing.lock() {
                *currently_syncing = true;
            }

            // FIXME: ew expects here....
            let has_passed = previous_time.cmp(&Local::now()) == Ordering::Less;

            if !has_passed {
                tokio::time::sleep(
                    previous_time
                        .signed_duration_since(Local::now())
                        .to_std()
                        .expect("?"),
                )
                .await;
            }

            previous_time = Local::now()
                .checked_add_signed(chrono::Duration::milliseconds(500))
                .expect("?");

            for i in 0..size {
                let command = &mut buffer[i];

                command.update_id(&temp_id_mapping);
            }

            if !buffer.is_empty() {
                let client = reqwest::Client::new();
                let mut params = HashMap::new();
                params.insert("commands", serde_json::to_string(&buffer[..size]).unwrap());
                let sync = client
                    .post("https://api.todoist.com/sync/v9/sync")
                    .header("Authorization", format!("Bearer {}", clone_token))
                    .form(&params);

                let response = sync.send().await.unwrap();

                let response = response.text().await.unwrap();

                match serde_json::from_str::<TodoistResponse>(&response) {
                    Ok(todoist_response) => {
                        temp_id_mapping.extend(todoist_response.temp_id_mapping.into_iter());

                        for (sync_status_id, response) in todoist_response.sync_status {
                            if let SyncStatus::Err(response) = response {
                                log_sender
                                    .send((
                                        format!(
                                            "Got an error(id = {}): {:?}",
                                            sync_status_id, response
                                        ),
                                        NaiveTime::default(),
                                    ))
                                    .await
                                    .expect("Cannot send a message to the log.");
                            }
                        }
                    }
                    Err(e) => {
                        log_sender
                            .send((e.to_string() + "\n" + &response, NaiveTime::default()))
                            .await
                            .expect("Cannot send a message to the log.");
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
