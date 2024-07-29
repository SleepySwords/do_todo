use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum TodoistCommand {
    #[serde(rename = "item_add")]
    ItemAdd {
        uuid: String,
        temp_id: String,
        args: TodoistItemAddCommand,
    },
}

#[derive(Serialize, Clone, Deserialize)]
pub struct TodoistItemAddCommand {
    pub content: String,
}
