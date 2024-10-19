use crate::task::{Priority, Task};

use super::todoist_command::TodoistDue;

#[derive(serde::Deserialize, Debug)]
pub struct TodoistItem {
    pub id: String,
    content: Option<String>,
    pub parent_id: Option<String>,
    pub child_order: usize,
    description: String,
    collapsed: bool,
    priority: usize,
    due: Option<TodoistDue>
}

impl From<TodoistItem> for Task {
    fn from(value: TodoistItem) -> Self {
        Task {
            progress: false,
            title: value.content.unwrap_or_else(|| String::from("")) + "\n" + &value.description,
            priority: todoist_to_priority(value.priority),
            tags: Vec::new(),
            due_date: value.due.map(|d| d.date),
            opened: !value.collapsed,
        }
    }
}
fn todoist_to_priority(priority: usize) -> Priority {
    match priority {
        2 => Priority::Low,
        3 => Priority::Normal,
        4 => Priority::High,
        _ => Priority::None,
    }
}
