use chrono::{Local, NaiveTime};

use crate::{
    component::status_line::StatusLine,
    component::{
        completed_list::CompletedListContext, overlay::Overlay, task_list::TaskListContext,
    },
    config::Config,
    draw::{Action, PostEvent},
    error::AppError,
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

    should_shutdown: bool,
}

// Above should be data, this should be map.
pub struct ScreenManager {
    pub app: App,
    pub overlays: Vec<Overlay<'static>>,
}

impl ScreenManager {
    pub fn push_layer(&mut self, component: Overlay<'static>) {
        self.overlays.push(component);
    }

    pub fn pop_layer(&mut self) -> Option<Overlay<'static>> {
        self.overlays.pop()
    }

    pub(crate) fn handle_post_event(&mut self, post_event: PostEvent) {
        match post_event.action {
            Action::PopOverlay(fun) => {
                if let Some(overlay) = self.pop_layer() {
                    let result = (fun)(&mut self.app, overlay);
                    self.handle_post_event(result);
                }
            }
            Action::PushLayer(overlay) => self.push_layer(overlay),
            Action::Noop => {}
        }
    }
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
