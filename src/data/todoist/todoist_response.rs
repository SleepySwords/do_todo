use std::collections::HashMap;

use serde::Deserialize;

use super::{
    todoist_project::TodoistProject,
    todoist_task::{TodoistCompletedItem, TodoistItem},
};

#[derive(Deserialize, Debug)]
pub struct TodoistResponse {
    pub temp_id_mapping: HashMap<String, String>,
    pub sync_status: HashMap<String, SyncStatus>,
    pub sync_token: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SyncStatus {
    #[allow(dead_code)]
    Ok(String),
    Err(TodoistError),
}

#[derive(serde::Deserialize, Debug)]
pub struct TodoistError {
    #[allow(dead_code)]
    error: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct TodoistSync {
    pub items: Option<Vec<TodoistItem>>,
    pub projects: Option<Vec<TodoistProject>>,
    pub sync_token: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct TodoistGetAllCompletedItemResponse {
    pub items: Vec<TodoistCompletedItem>,
}
