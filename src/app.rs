use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::actions::HelpAction;
use crate::component::completed_list::CompletedList;
use crate::component::input::dialog::DialogBox;
use crate::component::input::input_box::InputBox;
use crate::component::message_box::MessageBox;
use crate::component::status_line::StatusLine;
use crate::component::task_list::TaskList;
use crate::task::{CompletedTask, Tag, Task};
use crate::theme::Theme;

// PERF: Wow the technical debt is insane, have to rewrite all this :(
// Basic structure
// Renderer -> Calls individual render on each section, pass the context
// Where should the context be stored? Perhaps there is all the context with an is_visable tag?
// Universal variables should be in app (Tasks, themes)

// TODO: Refactor drawing system to allow screens or something, more complex modules
// Use a queue like system to basically ensure that no modules are removed.
// IDs? But that's a bit excessive.

#[derive(Default)]
pub struct App {
    pub popup_stack: Vec<UserInputType>,
    pub theme: Theme,
    pub task_store: TaskStore,

    pub status_line: StatusLine,

    pub selected_task_index: usize,
    pub selected_completed_task_index: usize,
    pub selected_component: SelectedComponent,

    should_shutdown: bool,
}

impl App {
    pub fn new(theme: Theme, task_data: TaskStore) -> App {
        App {
            theme,
            task_store: task_data,
            status_line: StatusLine::new(String::from("Press x for help. Press q to exit.")),
            ..Default::default()
        }
    }

    pub fn shutdown(&mut self) {
        self.should_shutdown = true
    }

    pub fn should_shutdown(&mut self) -> bool {
        self.should_shutdown
    }

    pub fn popup_context(&self) -> Option<&UserInputType> {
        self.popup_stack.last()
    }

    pub fn popup_context_mut(&mut self) -> Option<&mut UserInputType> {
        self.popup_stack.last_mut()
    }

    pub fn append_layer(&mut self, popup: UserInputType) {
        self.popup_stack.push(popup);
        self.selected_component = SelectedComponent::PopUpComponent;
    }

    pub fn pop_popup(&mut self) -> Option<UserInputType> {
        if self.popup_stack.len() == 1 {
            self.selected_component = SelectedComponent::CurrentTasks;
        }
        self.popup_stack.pop()
    }
}

pub enum UserInputType {
    Input(InputBox),
    Dialog(DialogBox),
    Message(MessageBox),
}

// TODO: This currently does not work due to how the component system is setup.
// impl UserInputType {
//     pub fn handle_event(&self, app: &mut App, key_code: KeyCode) {
//         let err = match self {
//             UserInputType::Input(_) => InputBox::handle_event(app, key_code),
//             UserInputType::Dialog(_) => DialogBox::handle_event(app, key_code),
//             UserInputType::Message(_) => MessageBox::handle_event(app, key_code),
//         };
//         if err.is_err() {
//             app.append_layer(UserInputType::Message(MessageBox::new(
//                 String::from("Error"),
//                 err.err().unwrap().to_string(),
//                 Color::Red,
//             )))
//         }
//     }
// }

#[derive(Default, Deserialize, Serialize)]
pub struct TaskStore {
    // eventually convert to vec
    pub tags: BTreeMap<u32, Tag>,

    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<CompletedTask>,
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
