use crate::{app::App, theme::Theme};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tui::style::Color;

#[derive(Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
    pub colour: Color,
}

#[derive(Deserialize, Serialize)]
pub struct Task {
    pub progress: bool,
    pub title: String,
    pub priority: Priority,

    pub tags: Vec<u32>,
}

impl Task {
    pub fn from_string(content: String) -> Self {
        Task {
            progress: false,
            title: content,
            priority: Priority::None,
            tags: Vec::new(),
        }
    }

    pub fn first_tag<'a>(&self, app: &'a App) -> Option<&'a Tag> {
        app.task_data.tags.get(self.tags.first().unwrap())
    }

    // PERF: Potentially expensive
    pub fn flip_tag(&mut self, tag: u32) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag)
        } else {
            self.tags.retain(|x| x != &tag);
        }
    }

    pub fn from_completed_task(completed_task: CompletedTask) -> Self {
        Task {
            progress: false,
            title: completed_task.title,
            priority: completed_task.priority,
            tags: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CompletedTask {
    pub title: String,
    pub time_completed: NaiveDateTime,
    pub priority: Priority,
}

impl CompletedTask {
    pub fn from_task(task: Task, time_completed: NaiveDateTime) -> Self {
        CompletedTask {
            title: task.title,
            time_completed,
            priority: task.priority,
        }
    }

    pub fn from_string(content: String, time_completed: NaiveDateTime) -> Self {
        CompletedTask {
            title: content,
            time_completed,
            priority: Priority::None,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum Priority {
    None,
    High,
    Normal,
    Low,
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

    pub fn short_hand(&self) -> &str {
        match *self {
            Priority::None => "    ",
            Priority::High => "!!! ",
            Priority::Normal => "!!  ",
            Priority::Low => "!   ",
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
    pub fn colour(&self, theme: &Theme) -> Color {
        match self {
            Priority::None => Color::White,
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
