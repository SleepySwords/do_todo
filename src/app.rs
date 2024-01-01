use chrono::{Local, NaiveTime};

use crate::{
    actions::HelpAction,
    component::completed_list::CompletedList,
    component::status_line::StatusLine,
    component::{
        completed_list::CompletedListContext,
        overlay::Overlay,
        task_list::{TaskList, TaskListContext},
    },
    config::Config,
    task::TaskStore,
};

#[derive(Default)]
pub struct App {
    pub config: Config,
    pub task_store: TaskStore,

    pub status_line: StatusLine,

    pub mode: Mode,

    pub logs: Vec<(String, NaiveTime)>,

    pub task_list: TaskListContext,
    pub completed_list: CompletedListContext,
    pub overlays: Vec<Overlay<'static>>,

    should_shutdown: bool,
}

impl App {
    pub fn new(theme: Config, task_data: TaskStore) -> App {
        App {
            config: theme,
            task_store: task_data,
            status_line: StatusLine::new(String::from("Press x for help. Press q to exit.")),
            ..Default::default()
        }
    }

    pub fn selected_index(&mut self, mode: Mode) -> Option<&mut usize> {
        match mode {
            Mode::CurrentTasks => Some(&mut self.task_list.selected_index),
            Mode::CompletedTasks => Some(&mut self.completed_list.selected_index),
            Mode::Overlay => match self.overlays.last_mut() {
                Some(Overlay::Dialog(dialog)) => Some(&mut dialog.index),
                Some(Overlay::Fuzzy(fuzzy)) => Some(&mut fuzzy.index),
                _ => None,
            },
        }
    }

    pub fn shutdown(&mut self) {
        self.should_shutdown = true
    }

    pub fn should_shutdown(&mut self) -> bool {
        self.should_shutdown
    }

    // Perhaps should use a static variable.
    pub fn println(&mut self, line: String) {
        self.logs.push((line, Local::now().time()));
    }

    pub fn push_layer(&mut self, component: Overlay<'static>) {
        self.overlays.push(component);
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    CurrentTasks,
    CompletedTasks,
    Overlay,
}

impl Default for Mode {
    fn default() -> Self {
        Self::CurrentTasks
    }
}

impl Mode {
    pub fn available_help_actions(&self, theme: &Config) -> Vec<HelpAction> {
        match self {
            Mode::CurrentTasks => TaskList::available_actions(theme),
            Mode::CompletedTasks => CompletedList::available_actions(theme),
            Mode::Overlay => vec![],
        }
    }
}
