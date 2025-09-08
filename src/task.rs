use crate::data::data_store::DataTaskStore;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tui::style::Color;

use std::fmt::Display;

use crate::{
    app::App,
    config::{color_parser, Config},
    data::data_store::TaskID,
};

#[derive(Deserialize, Serialize, Clone)]
pub struct Tag {
    pub name: String,
    #[serde(with = "color_parser")]
    pub colour: Color,
}

#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Task {
    pub progress: bool,
    pub title: String,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub due_date: Option<NaiveDate>,

    // Ignored if sub_tasks is empty
    pub opened: bool,
}

impl Task {
    pub fn from_string<T: Into<String>>(content: T) -> Self {
        Task {
            progress: false,
            title: content.into(),
            priority: Priority::None,
            tags: Vec::new(),
            due_date: None,
            opened: true,
        }
    }

    pub fn first_tag<'a>(&self, app: &'a App) -> Option<&'a Tag> {
        app.task_store.tags().get(self.tags.first().unwrap())
    }

    pub fn iter_tags<'a>(&'a self, app: &'a App) -> impl Iterator<Item = &'a Tag> + 'a {
        self.tags
            .iter()
            .filter_map(|tag_index| app.task_store.tags().get(tag_index))
    }

    pub fn flip_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag)
        } else {
            self.tags.retain(|x| x != &tag);
        }
    }

    pub fn from_completed_task(completed_task: CompletedTask) -> Self {
        completed_task.task
    }
}

#[derive(Deserialize, Clone, Serialize)]
pub struct CompletedTask {
    pub task: Task,
    pub time_completed: NaiveDateTime,
}

impl CompletedTask {
    pub fn from_task(task: Task, time_completed: NaiveDateTime) -> Self {
        CompletedTask {
            task,
            time_completed,
        }
    }

    pub fn from_string(content: String, time_completed: NaiveDateTime) -> Self {
        CompletedTask {
            task: Task {
                progress: false,
                title: content,
                priority: Priority::None,
                due_date: None,
                tags: Vec::new(),
                opened: true,
            },
            time_completed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Default)]
pub enum Priority {
    #[default]
    None,
    Low,
    Normal,
    High,
}

impl Priority {
    pub fn display_string(&self) -> &str {
        match *self {
            Priority::None => "None",
            Priority::High => "High",
            Priority::Normal => "Normal",
            Priority::Low => "Low",
        }
    }

    pub fn short_hand<'a>(&self, config: &'a Config) -> &'a str {
        match *self {
            Priority::None => &config.none_priority_display,
            Priority::High => &config.high_priority_display,
            Priority::Normal => &config.normal_priority_display,
            Priority::Low => &config.low_priority_display,
        }
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Priority::None => write!(f, "None"),
            Priority::High => write!(f, "High"),
            Priority::Normal => write!(f, "Normal"),
            Priority::Low => write!(f, "Low"),
        }
    }
}

impl Priority {
    pub fn colour(&self, theme: &Config) -> Color {
        match self {
            Priority::None => theme.none_priority_colour,
            Priority::High => theme.high_priority_colour,
            Priority::Normal => theme.normal_priority_colour,
            Priority::Low => theme.low_priority_colour,
        }
    }

    pub fn next_priority(&self) -> Priority {
        match self {
            Priority::None => Priority::High,
            Priority::High => Priority::Normal,
            Priority::Normal => Priority::Low,
            Priority::Low => Priority::None,
        }
    }
}

pub struct FindParentResult {
    /// If there is a parent, the index of the parent within it's task.
    pub parent_id: Option<TaskID>,
    /// The index of the task within the top-level tasks or within the subtask.
    pub task_local_offset: usize,
}
