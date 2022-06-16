use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::components::dialog::DialogComponent;
use crate::task::{CompletedTask, Task};
use crate::theme::Theme;

pub struct App {
    pub theme: Theme,
    pub selected_window: SelectedComponent,
    pub action: Action,
    pub words: String,
    pub task_data: TaskData,
    pub dialog_stack: VecDeque<DialogComponent>,
}

#[derive(Deserialize, Default, Serialize)]
pub struct TaskData {
    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<CompletedTask>,
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

impl Default for App {
    fn default() -> Self {
        let queue = VecDeque::new();
        App {
            theme: Theme::default(),
            selected_window: SelectedComponent::default(),
            action: Action::default(),
            words: String::default(),
            task_data: TaskData::default(),
            dialog_stack: queue,
        }
    }
}

pub enum SelectedComponent {
    CurrentTasks(usize),
    CompletedTasks(usize),
    // OptionPopUp(usize),
    // InputBox
}

impl SelectedComponent {
    pub fn get_selected(&mut self) -> Option<&mut usize> {
        match self {
            SelectedComponent::CurrentTasks(index) => Some(index),
            SelectedComponent::CompletedTasks(index) => Some(index),
            // Windows::OptionPopUp(index) => Some(index),
            // Windows::InputBox => None,
        }
    }
}

impl Default for SelectedComponent {
    fn default() -> Self {
        Self::CurrentTasks(0)
    }
}

pub enum Action {
    Normal,
    Add,
    // Perhaps replace with a referance for clarity.
    Edit(usize),
    Delete(usize, usize),
}

impl Default for Action {
    fn default() -> Self {
        Self::Normal
    }
}
