use crate::theme::Theme;
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
    pub fn new(content: String) -> Self {
        Task {
            progress: false,
            title: content,
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
            Priority::Low => "Low"
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
