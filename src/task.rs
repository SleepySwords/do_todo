use tui::style::Color;

pub struct Task {
    pub progress: bool,
    pub content: String,
    pub priority: Priority,
}

impl Task {
    pub fn new(content: String) -> Self {
        Task {
            progress: false,
            content,
            priority: Priority::Normal,
        }
    }
}

pub enum Priority {
    High,
    Normal,
    Low,
}

impl Priority {
    pub fn get_colour(&self) -> Color {
        match self {
            Priority::High => Color::Red,
            Priority::Normal => Color::White,
            Priority::Low => Color::Green,
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
