use std::{
    cmp::Ordering,
    collections::HashMap,
    panic,
    sync::{Arc, Mutex},
    usize,
};

use chrono::{Local, Months, NaiveDate};
use itertools::Itertools;
use serde_json::Value;
use tokio::{join, sync::mpsc::Sender};
use tracing::debug;

use crate::{
    data::{
        data_store::DataTaskStore,
        todoist::{
            todoist_command::TodoistCommand,
            todoist_response::{SyncStatus, TodoistGetAllCompletedItemResponse, TodoistResponse, TodoistSync},
        },
    },
    task::{CompletedTask, Task},
};

use super::todoist_data_store::TodoistDataStore;

pub const API_GATEWAY: &str = "https://api.todoist.com/api/v1/sync";

pub async fn get_initial_tasks<T: Into<String>>(
    todoist_auth: T,
) -> (
    Vec<String>,
    HashMap<String, Task>,
    Vec<String>,
    HashMap<String, CompletedTask>,
    HashMap<String, Vec<String>>,
    String,
    Option<String>,
) {
    let token = todoist_auth.into();
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("sync_token", "*");
    params.insert("resource_types", "[\"all\"]");
    let sync = client
        .post(API_GATEWAY)
        .header("Authorization", format!("Bearer {}", &token))
        .form(&params);

    let completed_items = client
        .get("https://api.todoist.com/api/v1/tasks/completed/by_completion_date")
        .header("Authorization", format!("Bearer {}", &token))
        .query(&[
            (
                "since",
                Local::now()
                    .date_naive()
                    .checked_sub_months(Months::new(3))
                    .unwrap()
                    .to_string(),
            ),
            ("until", Local::now().date_naive().to_string()),
        ]);

    let (Ok(completed_items), Ok(sync)) = join!(completed_items.send(), sync.send()) else {
        panic!("A connection error occured");
    };

    let s = sync.text().await.unwrap();
    let sync: TodoistSync = match serde_json::from_str(&s) {
        Ok(s) => s,
        Err(e) => {
            panic!("Could not deserialise: {:?} \n because {:?}", s, e);
        }
    };
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
        .map(|f| (f.id.clone(), f.into()))
        .collect();

    let completed_root: Vec<String> = completed_tasks.keys().cloned().collect_vec();

    (
        root_tasks,
        tasks,
        completed_root,
        completed_tasks,
        subtasks,
        sync.sync_token,
        sync.projects
            .and_then(|f| f.into_iter().find_or_first(|f| f.name == "Inbox"))
            .map(|f| f.id),
    )
}

pub type TaskSync = (TodoistSync, HashMap<String, String>);

pub fn handle_sync(data_store: &mut TodoistDataStore, (todoist_sync, temp_id_mapping): TaskSync) {
    // FIXME: it might be better to map the todoist sync actual ids to temp ids?
    for (temp_id, actual_id) in temp_id_mapping.iter() {
        if let Some(v) = data_store.tasks.remove(temp_id) {
            data_store.tasks.insert(actual_id.to_string(), v);
        }

        for (_, k) in data_store.subtasks.iter_mut() {
            if let Some(position) = k.iter().position(|a| temp_id == a) {
                k[position] = actual_id.to_string();
            }
        }

        if let Some(position) = data_store.root.iter().position(|a| temp_id == a) {
            data_store.root[position] = actual_id.to_string();
        }
    }

    if let Some(mut items) = todoist_sync.items {
        // The way child order is done is as a cursor like thing.
        // Must sort by the child order and then append each to the subtasks.
        items.sort_by_key(|f| f.child_order);
        let is_move = items.get(0).map(|f| f.child_order).is_some_and(|f| f == 0);
        for item in items.into_iter() {
            if item.completed_at.is_some() {
                continue;
            }
            if "" == item.content.as_str() {
                data_store.root.retain(|f| *f != item.id);
                data_store
                    .subtasks
                    .values_mut()
                    .for_each(|val| val.retain(|f| *f != item.id));

                data_store.tasks.remove(&item.id);

                continue;
            }

            // Either we are inserting at the very start or this is a move
            // which in that case every item is sent
            if is_move && item.child_order == 0 {
                data_store.root.clear();
            }

            if let Some(task) = data_store.tasks.get_mut(item.id.as_str()) {
                let parent_id = item.parent_id.clone();
                if task.opened && is_move {
                    if let Some(subtasks) = data_store.subtasks.get_mut(&item.id) {
                        subtasks.clear();
                    }
                }

                tracing::debug!(
                    "Not equal {:?} {:?}",
                    parent_id,
                    data_store.find_parent(&item.id).and_then(|f| f.parent_id)
                );
                if is_move
                    || parent_id != data_store.find_parent(&item.id).and_then(|f| f.parent_id)
                {
                    data_store.append_internal(&item.id, parent_id, Some(()));
                }

                let task = data_store.tasks.get_mut(item.id.as_str()).unwrap();
                *task = item.into();
            } else {
                let parent_id = item.parent_id.clone();
                let subtasks = if let Some(parent_id) = parent_id {
                    data_store
                        .subtasks
                        .entry(parent_id)
                        .or_insert_with(Vec::new)
                } else {
                    &mut data_store.root
                };
                subtasks.push(item.id.clone());

                data_store.tasks.insert(item.id.clone(), item.into());
            }
        }
    }
}

pub async fn sync<T: Into<String>>(
    todoist_auth: T,
    sync_send: Sender<TaskSync>,
) -> TodoistDataStore {
    println!("Attempting to connect to Todoist");

    let token = todoist_auth.into();

    let (root_tasks, tasks, completed_root, completed_tasks, subtasks, sync_token, inbox_project) =
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
                if let TodoistCommand::Send(command) = command {
                    command.update_id(&temp_id_mapping);
                }
            }

            if !buffer.is_empty() {
                let client = reqwest::Client::new();
                let mut params = HashMap::new();

                let commands = buffer[..size]
                    .iter()
                    .flat_map(|f| {
                        if let TodoistCommand::Send(command) = f {
                            Some(command)
                        } else {
                            None
                        }
                    })
                    .collect_vec();

                let should_refresh = buffer[..size].contains(&TodoistCommand::Refresh);
                debug!(should_refresh);

                if should_refresh {
                    let mut params = HashMap::new();
                    params.insert("sync_token", previous_token);
                    params.insert("resource_types", "[\"all\"]".to_string());
                    let sync = client
                        .post(API_GATEWAY)
                        .header("Authorization", format!("Bearer {}", &token))
                        .form(&params)
                        .send()
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap();

                    let json = serde_json::from_str::<TodoistSync>(&sync).unwrap();

                    previous_token = json.sync_token;

                    let _ = sync_send
                        .send((
                            serde_json::from_str::<TodoistSync>(&sync).unwrap(),
                            temp_id_mapping.clone(),
                        ))
                        .await;
                    if let Ok(sync_req) = &serde_json::from_str::<Value>(&sync)
                        .and_then(|f| serde_json::to_string_pretty(&f))
                    {
                        tracing::info!("Got an sync request: {}", sync_req)
                    }
                } else {
                    params.insert("commands", serde_json::to_string(&commands).unwrap());
                    tracing::debug!("Sending command: {:?}", params);
                    let sync = client
                        .post(API_GATEWAY)
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
                                        serde_json::to_string(&commands).unwrap()
                                    );
                                }
                            }

                            let mut params = HashMap::new();
                            params.insert("sync_token", previous_token);
                            params.insert("resource_types", "[\"all\"]".to_string());
                            let sync = client
                                .post(API_GATEWAY)
                                .header("Authorization", format!("Bearer {}", &token))
                                .form(&params)
                                .send()
                                .await
                                .unwrap()
                                .text()
                                .await
                                .unwrap();

                            previous_token = todoist_response.sync_token;
                            let _ = sync_send
                                .send((
                                    serde_json::from_str::<TodoistSync>(&sync).unwrap(),
                                    temp_id_mapping.clone(),
                                ))
                                .await;
                            tracing::info!("Got an sync request: {:?}", sync)
                        }
                        Err(e) => {
                            tracing::error!("{}", e.to_string() + "\n " + &response);
                        }
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
        tasks,
        completed_tasks,
        subtasks,
        root: root_tasks,
        completed_root,
        tags: HashMap::new(),
        task_count: 0,
        currently_syncing: mutex,
        command_sender: send,
        inbox_project,
    }
}
