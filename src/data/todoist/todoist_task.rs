use chrono::DateTime;

use crate::task::{CompletedTask, Priority, Task};

use super::todoist_command::TodoistDue;

#[derive(serde::Deserialize, Debug)]
pub struct TodoistItem {
    pub id: String,
    pub content: String,
    pub parent_id: Option<String>,
    pub child_order: usize,
    description: String,
    is_collapsed: bool,
    priority: usize,
    due: Option<TodoistDue>,
    pub completed_at: Option<String>
}

impl From<TodoistItem> for Task {
    fn from(value: TodoistItem) -> Self {
        Task {
            progress: false,
            title: value.content + "\n" + &value.description,
            priority: todoist_to_priority(value.priority),
            tags: Vec::new(),
            due_date: value.due.map(|d| d.date),
            opened: !value.is_collapsed,
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct TodoistCompletedItem {
    pub id: String,
    pub task_id: Option<String>,
    pub project_id: Option<String>,
    pub section_id: Option<String>,
    content: Option<String>,
    completed_at: String,
}

impl From<TodoistCompletedItem> for CompletedTask {
    fn from(value: TodoistCompletedItem) -> Self {
        CompletedTask {
            task: Task {
                progress: false,
                title: value.content.unwrap_or_else(|| String::from("")),
                priority: Priority::None,
                tags: Vec::new(),
                due_date: None,
                opened: false,
            },
            time_completed: DateTime::parse_from_rfc3339(&value.completed_at)
                .unwrap()
                .naive_utc(),
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
