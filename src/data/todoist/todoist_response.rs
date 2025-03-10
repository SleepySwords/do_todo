use std::collections::HashMap;

use serde::Deserialize;

use super::todoist_task::{TodoistCompletedItem, TodoistItem};

#[derive(Deserialize, Debug)]
pub struct TodoistResponse {
    pub temp_id_mapping: HashMap<String, String>,
    pub sync_status: HashMap<String, SyncStatus>,
    pub sync_token: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SyncStatus {
    Ok(()),
    Err(TodoistError),
}

#[derive(serde::Deserialize, Debug)]
pub struct TodoistError {
    error: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct TodoistSync {
    pub items: Option<Vec<TodoistItem>>,
    pub sync_token: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct TodoistGetAllCompletedItemResponse {
    pub items: Vec<TodoistCompletedItem>,
}
