use crate::task::Task;
use crate::theme::Theme;

#[derive(Default)]
pub struct App {
    pub add_mode: bool,
    pub theme: Theme,
    pub selected_chunk: Selection,
    pub words: String,
    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<Task>,
}

impl App {
    pub fn new(theme: Theme, tasks: Vec<Task>) -> App {
        App {
            theme,
            tasks,
            ..Default::default()
        }
    }
}

pub enum Selection {
    CurrentTasks(usize),
    CompletedTasks(usize),
}

impl Default for Selection {
    fn default() -> Self {
        Self::CurrentTasks(0)
    }
}
