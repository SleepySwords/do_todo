use serde::{Deserialize, Serialize};

use crate::task::{CompletedTask, Task};
use crate::theme::Theme;

#[derive(Default)]
pub struct App {
    pub theme: Theme,
    pub selected_window: Windows,
    pub mode: Mode,
    pub words: String,
    pub task_data: TaskData
}

#[derive(Deserialize, Default, Serialize)]
pub struct TaskData {
    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<CompletedTask>

}

impl App {
    pub fn new(theme: Theme, task_data: TaskData) -> App {
        App {
            theme,
            task_data,
            ..Default::default()
        }
    }
}

pub enum Windows {
    CurrentTasks(usize),
    CompletedTasks(usize),
    // OptionPopUp(usize),
    // InputBox
}

impl Windows {
    pub fn get_selected(&mut self) -> Option<&mut usize> {
        match self {
            Windows::CurrentTasks(index) => Some(index),
            Windows::CompletedTasks(index) => Some(index),
            // Windows::OptionPopUp(index) => Some(index),
            // Windows::InputBox => None,
        }
    }
}

impl Default for Windows {
    fn default() -> Self {
        Self::CurrentTasks(0)
    }
}

pub enum Mode {
    Normal,
    Add,
    // Perhaps replace with a referance for clarity.
    Edit(usize),
    Delete(usize, usize),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}
