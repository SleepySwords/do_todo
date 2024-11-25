use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::task::{Priority, Task};

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TodoistCommand {
    #[serde(rename = "item_add")]
    ItemAdd {
        uuid: String,
        temp_id: String,
        args: TodoistItemAddCommand,
    },
    #[serde(rename = "item_delete")]
    ItemDelete {
        uuid: String,
        args: TodoistItemDeleteCommand,
    },
    #[serde(rename = "item_reorder")]
    ItemReorder {
        uuid: String,
        args: TodoistItemReorderCommand,
    },
    #[serde(rename = "item_update")]
    ItemUpdate {
        uuid: String,
        args: TodoistUpdateItem,
    },
    #[serde(rename = "item_complete")]
    ItemComplete {
        uuid: String,
        args: TodoistItemCompleteCommand,
    }
}

impl TodoistCommand {
    pub fn update_id(&mut self, temp_id_mapping: &HashMap<String, String>) {
        if let TodoistCommand::ItemDelete { args, .. } = self {
            if let Some(new_id) = temp_id_mapping.get(&args.id) {
                args.id = new_id.to_string();
            }
        }
    }
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TodoistItemAddCommand {
    pub content: String,
    pub parent_id: Option<String>,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TodoistItemDeleteCommand {
    pub id: String,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TodoistItemReorder {
    pub id: String,
    pub child_order: usize,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TodoistItemReorderCommand {
    pub items: Vec<TodoistItemReorder>,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TodoistUpdateItem {
    pub id: String,
    pub content: Option<String>,
    pub collapsed: bool,
    pub priority: usize,
    pub due: Option<TodoistDue>,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TodoistItemCompleteCommand {
    pub id: String,
    pub date_completed: Option<NaiveDate>
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TodoistDue {
    pub date: NaiveDate,
}

pub fn task_to_todoist(id: String, task: &Task) -> TodoistUpdateItem {
    return TodoistUpdateItem {
        id,
        content: Some(task.title.clone()),
        collapsed: !task.opened,
        priority: priority_to_todoist(task.priority),
        due: task.due_date.map(|date| TodoistDue { date }),
    };
}

pub fn priority_to_todoist(priority: Priority) -> usize {
    match priority {
        Priority::None => 1,
        Priority::Low => 2,
        Priority::Normal => 3,
        Priority::High => 4,
    }
}
