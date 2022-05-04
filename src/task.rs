use crate::theme::Theme;
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tui::style::Color;

#[derive(Deserialize, Serialize)]
pub struct Task {
    pub progress: bool,
    pub title: String,
    pub priority: Priority,
}

impl Task {
    pub fn from_string(content: String) -> Self {
        Task {
            progress: false,
            title: content,
            priority: Priority::Normal,
        }
    }

    pub fn from_completed_task(completed_task: CompletedTask) -> Self {
        Task {
            progress: false,
            title: completed_task.title,
            priority: completed_task.priority,
        }
    }
}

pub struct CompletedTask {
    pub title: String,
    pub time_completed: NaiveTime,
    pub priority: Priority,
}

impl CompletedTask {
    pub fn from_task(task: Task, time_completed: NaiveTime) -> Self {
        CompletedTask {
            title: task.title,
            time_completed,
            priority: task.priority,
        }
    }

    pub fn from_string(content: String, time_completed: NaiveTime) -> Self {
        CompletedTask {
            title: content,
            time_completed,
            priority: Priority::Normal,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum Priority {
    High,
    Normal,
    Low,
}

impl Priority {
    pub fn get_display_string(&self) -> &str {
        match *self {
            Priority::High => "High",
            Priority::Normal => "Normal",
            Priority::Low => "Low",
        }
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Priority::High => write!(f, "High"),
            Priority::Normal => write!(f, "Normal"),
            Priority::Low => write!(f, "Low"),
        }
    }
}

impl Priority {
    pub fn get_colour(&self, theme: &Theme) -> Color {
        match self {
            Priority::High => theme.high_priority_colour,
            Priority::Normal => theme.normal_priority_colour,
            Priority::Low => theme.low_priority_colour,
        }
    }

    pub fn get_next(&self) -> Priority {
        match self {
            Priority::High => Priority::Low,
            Priority::Normal => Priority::High,
            Priority::Low => Priority::Normal,
        }
    }
}
