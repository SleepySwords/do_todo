use crate::task::{CompletedTask, Task};
use crate::theme::Theme;

#[derive(Default)]
pub struct App {
    pub add_mode: bool,
    pub theme: Theme,
    pub selected_window: Windows,
    pub mode: Mode,
    pub words: String,
    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<CompletedTask>,
}

impl App {
    pub fn new(theme: Theme, tasks: Vec<Task>) -> App {
        App {
            theme,
            tasks,
            ..Default::default()
        }
    }
}

pub enum Windows {
    CurrentTasks(usize),
    CompletedTasks(usize),
}

impl Default for Windows {
    fn default() -> Self {
        Self::CurrentTasks(0)
    }
}

pub enum Mode {
    Normal,
    Input,
    // Perhaps replace with a referance for clarity.
    Edit(usize),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}
