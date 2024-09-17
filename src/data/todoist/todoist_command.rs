use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TodoistItemDeleteCommand {
    pub id: String,
}
