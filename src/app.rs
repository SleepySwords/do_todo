use crate::task::Task;
use crate::theme::Theme;

pub struct App {
    pub add_mode: bool,
    pub theme: Theme,
    pub selected_index: usize,
    pub words: String,
    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<Task>,
}

impl Default for App {
    fn default() -> Self {
        App {
            add_mode: false,
            theme: Theme::default(),
            selected_index: 0,
            words: String::new(),
            tasks: Vec::new(),
            completed_tasks: Vec::new(),
        }
    }
}
