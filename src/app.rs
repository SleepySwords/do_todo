use std::collections::{BTreeMap, VecDeque};

use chrono::{NaiveTime, Local};
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use tui::layout::Rect;

use crate::actions::HelpAction;
use crate::component::completed_list::CompletedList;
use crate::component::layout::stack_layout::StackLayout;
use crate::component::status_line::StatusLine;
use crate::component::task_list::TaskList;
use crate::task::{CompletedTask, Tag, Task};
use crate::theme::Theme;
use crate::view::DrawableComponent;

type Callback = dyn FnOnce(&mut App, &mut StackLayout);

#[derive(Default)]
pub struct App {
    pub theme: Theme,
    pub task_store: TaskStore,

    pub status_line: StatusLine,

    pub callbacks: VecDeque<Box<Callback>>,
    pub selected_component: SelectedComponent,

    pub app_size: Rect,
    pub logs: Vec<(String, NaiveTime)>,

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

    pub fn pop_layer(&mut self) {
        self.callbacks.push_back(Box::new(|_, x| {
            x.pop_layer();
        }));
    }

    pub fn println(&mut self, line: String) {
        self.logs.push((line, Local::now().time()));
    }

    pub fn pop_layer_callback<T>(&mut self, callback: T)
    where
        T: FnOnce(&mut App, &mut StackLayout, Option<Box<dyn DrawableComponent>>) + 'static,
    {
        self.callbacks.push_back(Box::new(|app, x| {
            let comp = x.pop_layer();
            callback(app, x, comp)
        }));
    }

    // FIX: use generics?!
    pub fn push_layer<T: DrawableComponent + 'static>(&mut self, component: T) {
        self.callbacks
            .push_back(Box::new(|_, x| x.append_layer(Box::new(component))));
    }

    pub fn execute_event(&mut self, key_code: KeyCode) {
        self.callbacks.push_back(Box::new(move |app, x| {
            x.key_pressed(app, key_code);
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

impl TaskStore {
    pub fn delete_tag(&mut self, tag_id: u32) {
        self.tags.remove(&tag_id);
        for task in &mut self.tasks {
            task.tags.retain(|f| f != &tag_id);
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum SelectedComponent {
    CurrentTasks,
    CompletedTasks,
    Overlay,
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
            SelectedComponent::Overlay => vec![],
        }
    }
}
