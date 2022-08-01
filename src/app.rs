use serde::{Deserialize, Serialize};

use crate::components::dialog::DialogComponent;
use crate::components::input_box::InputBoxComponent;
use crate::task::{CompletedTask, Task};
use crate::theme::Theme;

// Consider either putting all the data in app or using something such as Rc and RefCells?
#[derive(Default)]
pub struct App {
    pub popup_stack: Vec<PopUpComponents>,
    pub theme: Theme,
    pub selected_component: SelectedComponent,
    pub words: String,
    pub task_data: TaskData,

    pub selected_task_index: usize,
    pub selected_completed_task_index: usize,

    should_shutdown: bool,
}

pub enum PopUpComponents {
    InputBox(InputBoxComponent),
    DialogBox(DialogComponent),
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

    pub fn shutdown(&mut self) {
        self.should_shutdown = true
    }

    pub fn should_shutdown(&mut self) -> bool {
        self.should_shutdown
    }
}

#[derive(PartialEq)]
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
