use chrono::{Local, NaiveTime};

use serde::{Deserialize, Serialize};

use std::{cmp, collections::BTreeMap};

use crate::{
    actions::HelpAction,
    component::completed_list::CompletedList,
    component::status_line::StatusLine,
    component::{
        completed_list::CompletedListContext,
        overlay::Overlay,
        task_list::{TaskList, TaskListContext},
    },
    task::{CompletedTask, Tag, Task},
    theme::Theme,
};

#[derive(Default)]
pub struct App {
    pub theme: Theme,
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
    pub fn new(theme: Theme, task_data: TaskStore) -> App {
        App {
            theme,
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

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct TaskStore {
    pub tags: BTreeMap<u32, Tag>,
    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<CompletedTask>,
    pub auto_sort: bool,
}

impl TaskStore {
    pub fn delete_tag(&mut self, tag_id: u32) {
        self.tags.remove(&tag_id);
        for task in &mut self.tasks {
            task.tags.retain(|f| f != &tag_id);
        }
    }

    pub fn sort(&mut self) {
        self.tasks.sort_by_key(|t| cmp::Reverse(t.priority));
    }

    pub fn add_task(&mut self, task: Task) {
        if self.auto_sort {
            self.tasks.push(task);
            self.sort();
        } else {
            self.tasks.push(task);
        }
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
    pub fn available_help_actions(&self, theme: &Theme) -> Vec<HelpAction> {
        match self {
            Mode::CurrentTasks => TaskList::available_actions(theme),
            Mode::CompletedTasks => CompletedList::available_actions(theme),
            Mode::Overlay => vec![],
        }
    }
}
