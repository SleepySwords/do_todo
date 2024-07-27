use chrono::{Local, NaiveTime};

use crate::{
    component::{
        completed_list::CompletedListContext, status_line::StatusLine, task_list::TaskListContext,
    },
    config::Config,
    data::data_store::DataTaskStore,
    error::AppError,
    framework::event::PostEvent,
};

pub struct App {
    pub config: Config,
    pub task_store: Box<dyn DataTaskStore>,

    pub status_line: StatusLine,

    pub mode: Mode,

    pub logs: Vec<(String, NaiveTime)>,

    pub task_list: TaskListContext,
    pub completed_list: CompletedListContext,

    pub tick: usize,

    should_shutdown: bool,
}

impl App {
    pub fn new(theme: Config, task_data: Box<dyn DataTaskStore>) -> App {
        App {
            config: theme,
            task_store: task_data,
            status_line: StatusLine::new(String::from("Press x for help. Press q to exit.")),
            mode: Mode::CurrentTasks,
            logs: vec![],
            task_list: TaskListContext::default(),
            completed_list: CompletedListContext::default(),
            tick: 0,
            should_shutdown: false,
        }
    }

    /// Returns the selected index only for the current tasks and completed tasks
    /// This returns None for Overlays.
    pub fn selected_index(&mut self, mode: Mode) -> Option<&mut usize> {
        match mode {
            Mode::CurrentTasks => Some(&mut self.task_list.selected_index),
            Mode::CompletedTasks => Some(&mut self.completed_list.selected_index),
            Mode::Overlay => None,
        }
    }

    pub fn shutdown(&mut self) -> Result<PostEvent, AppError> {
        self.should_shutdown = true;
        Ok(PostEvent::noop(false))
    }

    pub fn should_shutdown(&self) -> bool {
        self.should_shutdown
    }

    // Perhaps should use a static variable.
    pub fn println(&mut self, line: String) {
        self.logs.push((line, Local::now().time()));
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
