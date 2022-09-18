use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::actions::HelpAction;
use crate::component::completed_list::CompletedList;
use crate::component::dialog::DialogComponent;
use crate::component::input_box::InputBoxComponent;
use crate::component::task_list::TaskList;
use crate::task::{CompletedTask, Task, Tag};
use crate::theme::Theme;

#[derive(Default)]
pub struct App {
    pub popup_stack: Vec<PopUpComponents>,
    pub theme: Theme,
    pub words: String,
    pub task_data: TaskData,

    pub selected_task_index: usize,
    pub selected_completed_task_index: usize,
    pub selected_component: SelectedComponent,

    should_shutdown: bool,
}

pub enum PopUpComponents {
    InputBox(InputBoxComponent),
    DialogBox(DialogComponent),
}

#[derive(Deserialize, Default, Serialize)]
pub struct TaskData {
    // eventually convert to vec
    pub tags: HashMap<u32, Tag>,

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

    pub fn shutdown(&mut self) {
        self.should_shutdown = true
    }

    pub fn should_shutdown(&mut self) -> bool {
        self.should_shutdown
    }
}

#[derive(PartialEq, Eq)]
pub enum SelectedComponent {
    CurrentTasks,
    CompletedTasks,
    PopUpComponent,
}

impl Default for SelectedComponent {
    fn default() -> Self {
        Self::CurrentTasks
    }
}

impl SelectedComponent {
    pub fn available_help_actions(&self) -> Vec<HelpAction> {
        match self {
            SelectedComponent::CurrentTasks => TaskList::available_actions(),
            SelectedComponent::CompletedTasks => CompletedList::available_actions(),
            SelectedComponent::PopUpComponent => vec![],
        }
    }
}
