use std::collections::{BTreeMap, VecDeque};

use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

use crate::actions::HelpAction;
use crate::component::completed_list::CompletedList;
use crate::component::status_line::StatusLine;
use crate::component::task_list::TaskList;
use crate::task::{CompletedTask, Tag, Task};
use crate::theme::Theme;
use crate::view::{DrawableComponent, StackLayout};

// PERF: Wow the technical debt is insane, have to rewrite all this :(
// Basic structure
// Renderer -> Calls individual render on each section, pass the context
// Where should the context be stored? Perhaps there is all the context with an is_visable tag?
// Universal variables should be in app (Tasks, themes)

// TODO: Refactor drawing system to allow screens or something, more complex modules
// Use a queue like system to basically ensure that no modules are removed.
// IDs?
//
// Maybe a root node system with children would be better.

#[derive(Default)]
pub struct App {
    pub theme: Theme,
    pub task_store: TaskStore,

    pub status_line: StatusLine,

    pub callbacks: VecDeque<Box<dyn FnOnce(&mut App, &mut StackLayout) -> ()>>,
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

    pub fn pop_stack(&mut self) {
        self.callbacks.push_back(Box::new(|_, x| x.pop_layer()));
    }

    pub fn append_stack(&mut self, component: Box<dyn DrawableComponent>) {
        self.callbacks
            .push_back(Box::new(|_, x| x.append_layer(component)));
    }

    pub fn execute_event(&mut self, key_code: KeyCode) {
        self.callbacks.push_back(Box::new(move |app, x| {
            x.event(app, key_code);
        }));
    }
}

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
