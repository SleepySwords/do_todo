use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::actions::HelpAction;
use crate::component::completed_list::CompletedList;
use crate::component::dialog::DialogComponent;
use crate::component::input_box::InputBoxComponent;
use crate::component::message_box::MessageBoxComponent;
use crate::component::status_line::StatusLineComponent;
use crate::component::task_list::TaskList;
use crate::task::{CompletedTask, Tag, Task};
use crate::theme::Theme;

#[derive(Default)]
pub struct App {
    pub popup_stack: Vec<PopUpComponents>,
    pub theme: Theme,
    pub words: String,
    pub task_store: TaskStore,

    pub status_line: StatusLineComponent,

    pub selected_task_index: usize,
    pub selected_completed_task_index: usize,
    pub selected_component: SelectedComponent,

    should_shutdown: bool,
}

pub enum PopUpComponents {
    InputBox(InputBoxComponent),
    DialogBox(DialogComponent),
    MessageBox(MessageBoxComponent),
}

#[derive(Deserialize, Default, Serialize)]
pub struct TaskStore {
    // eventually convert to vec
    pub tags: BTreeMap<u32, Tag>,

    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<CompletedTask>,
}

// TODO: Refactor drawing system to allow screens or something, more complex modules
impl App {
    pub fn new(theme: Theme, task_data: TaskStore) -> App {
        App {
            theme,
            task_store: task_data,
            status_line: StatusLineComponent::new(String::from(
                "Press x for help. Press q to exit.",
            )),
            ..Default::default()
        }
    }

    pub fn shutdown(&mut self) {
        self.should_shutdown = true
    }

    pub fn should_shutdown(&mut self) -> bool {
        self.should_shutdown
    }

    pub fn popup_context(&self) -> Option<&PopUpComponents> {
        self.popup_stack.last()
    }

    pub fn popup_context_mut(&mut self) -> Option<&mut PopUpComponents> {
        self.popup_stack.last_mut()
    }

    pub fn append_layer(&mut self, popup: PopUpComponents) {
        self.popup_stack.push(popup);
        self.selected_component = SelectedComponent::PopUpComponent;
    }

    pub fn pop_popup(&mut self) -> Option<PopUpComponents> {
        if self.popup_stack.len() == 1 {
            self.selected_component = SelectedComponent::CurrentTasks;
        }
        self.popup_stack.pop()
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
