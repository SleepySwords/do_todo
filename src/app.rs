use crate::task::Task;
use crate::theme::Theme;

#[derive(Default)]
pub struct App {
    pub add_mode: bool,
    pub theme: Theme,
    pub selected_index: usize,
    pub words: String,
    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<Task>,
}
