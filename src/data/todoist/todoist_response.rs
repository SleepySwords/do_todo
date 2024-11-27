use std::collections::HashMap;

use serde::Deserialize;

use super::todoist_task::{TodoistCompletedItem, TodoistItem};

#[derive(Deserialize, Debug)]
pub struct TodoistResponse {
    pub temp_id_mapping: HashMap<String, String>,
    pub sync_status: HashMap<String, SyncStatus>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SyncStatus {
    Ok(String),
    Err(TodoistError),
}

#[derive(serde::Deserialize, Debug)]
pub struct TodoistError {
    error: String,
}

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

#[derive(serde::Deserialize, Debug)]
pub struct TodoistGetAllCompletedItemResponse {
    pub items: Vec<TodoistCompletedItem>,
}
