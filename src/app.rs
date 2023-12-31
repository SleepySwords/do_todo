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
    config::Config,
    task::{CompletedTask, Tag, Task},
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

#[derive(Deserialize, Serialize)]
enum DrawComponent {
    Category(usize),
    Task(usize),
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
    // FIXME: This is kinda badly structued :(
    pub fn task_mut(&mut self, mut selected: usize) -> Option<&mut Task> {
        self.tasks
            .iter_mut()
            .find_map(|t| t._find_selected_mut(&mut selected))
    }

    pub fn task(&self, mut selected: usize) -> Option<&Task> {
        self.tasks
            .iter()
            .find_map(|t| t._find_selected(&mut selected))
    }

    pub fn delete_task(&mut self, to_delete: usize) -> Option<Task> {
        Self::_delete_task(&mut self.tasks, &mut 0, to_delete)
    }

    fn _delete_task<'a>(
        tasks: &'a mut Vec<Task>,
        current_index: &mut usize,
        to_delete: usize,
    ) -> Option<Task> {
        for task_index in 0..tasks.len() {
            if *current_index == to_delete {
                return Some(tasks.remove(task_index));
            }

            *current_index += 1;

            if !tasks[task_index].opened {
                continue;
            }

            if let Some(task) =
                Self::_delete_task(&mut tasks[task_index].sub_tasks, current_index, to_delete)
            {
                return Some(task);
            }
        }
        None
    }

    pub fn find_task_size(&self) -> usize {
        self.tasks
            .iter()
            .map(|t| t.find_task_draw_size())
            .sum::<usize>()
    }

    pub fn translate_to_global(&self, to_find: &Vec<Task>, index: usize) -> Option<usize> {
        Self::_translate_to_global(&self.tasks, to_find, &mut 0).map(|f| {
            f + self
                .tasks
                .iter()
                .take(index)
                .map(|tsk| tsk.find_task_draw_size())
                .sum::<usize>()
        })
    }

    fn _translate_to_global<'a>(
        tasks: &'a Vec<Task>,
        to_find: &'a Vec<Task>,
        selected: &mut usize,
    ) -> Option<usize> {
        if to_find == tasks {
            return Some(*selected);
        }
        *selected += 1;
        tasks
            .iter()
            .find_map(|tsk| Self::_translate_to_global(&tsk.sub_tasks, to_find, selected));
        None
    }

    pub fn find_parent(&self, to_find: usize) -> Option<(&Vec<Task>, usize, bool)> {
        Self::_find_parent(&self.tasks, &mut 0, to_find, 0, true)
    }

    fn _find_parent<'a>(
        tasks: &'a Vec<Task>,
        current_index: &mut usize,
        to_find: usize,
        offset: usize,
        is_global: bool,
    ) -> Option<(&'a Vec<Task>, usize, bool)> {
        for task_index in 0..tasks.len() {
            if *current_index == to_find {
                return Some((tasks, offset, is_global));
            }

            let offset = *current_index;

            *current_index += 1;

            if tasks[task_index].opened {
                if let Some(task) = Self::_find_parent(
                    &tasks[task_index].sub_tasks,
                    current_index,
                    to_find,
                    offset,
                    false,
                ) {
                    return Some(task);
                }
            }
        }
        None
    }

    // FIXME: This does not actually get the parent properly.
    pub fn find_parent_mut(&mut self, to_find: usize) -> Option<(&mut Vec<Task>, usize, bool)> {
        // FIXME: I have given up on finding a smart way to do this.
        // More inefficient, but should still be O(n)
        let (_, offset, is_global) = Self::find_parent(&self, to_find)?;
        if is_global {
            return Some((&mut self.tasks, offset, is_global ));
        } else {
            return Some((&mut self.task_mut(offset)?.sub_tasks, offset, is_global))
        }
    }

    // fn _find_parent_mut<'a>(
    //     tasks: &'a mut Vec<Task>,
    //     current_index: &mut usize,
    //     to_find: usize,
    //     offset: usize,
    //     is_global: bool,
    // ) -> Option<(&'a mut Vec<Task>, usize, bool)> {
    //     // FIXME: I have given up on finding a smart way to do this.

    //     // for task_index in 0..tasks.len() {
    //     //         let offset = *current_index;

    //     //         *current_index += 1;

    //     //         if tasks[task_index].opened {
    //     //             let sub_tasks = &mut tasks[task_index].sub_tasks;
    //     //             // FIXME: borrow checker bs leads to jank
    //     //             let hi =
    //     //                 Self::_find_parent_mut(sub_tasks, current_index, to_find, offset, false);
    //     //             if hi.is_none() {
    //     //                 continue;
    //     //             }
    //     //             return hi;
    //     //     }
    //     // }
    //     // None

    //     Self::find_parent(&self, to_find)
    // }

    pub fn task_position(&self, to_find: &Task) -> Option<usize> {
        let mut index = 0;
        self.tasks
            .iter()
            .find_map(|tsk| Self::_task_position(&to_find, tsk, &mut index))
    }

    // FIXME: Move to task
    fn _task_position(to_find: &Task, current_task: &Task, index: &mut usize) -> Option<usize> {
        if *to_find == *current_task {
            return Some(*index);
        }
        *index += 1;
        if !current_task.opened {
            return None;
        }
        current_task
            .sub_tasks
            .iter()
            .find_map(|sub_task| Self::_task_position(to_find, sub_task, index))
    }

    pub fn delete_tag(&mut self, tag_id: u32) {
        self.tags.remove(&tag_id);
        for task in &mut self.tasks {
            task.tags.retain(|f| f != &tag_id);
        }
        for completed_task in &mut self.completed_tasks {
            completed_task.task.tags.retain(|f| f != &tag_id);
        }
    }

    pub fn sort(&mut self) {
        self.tasks.sort_by_key(|t| cmp::Reverse(t.priority));
        for task in &mut self.tasks {
            task.sort_subtasks()
        }
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
    pub fn available_help_actions(&self, theme: &Config) -> Vec<HelpAction> {
        match self {
            Mode::CurrentTasks => TaskList::available_actions(theme),
            Mode::CompletedTasks => CompletedList::available_actions(theme),
            Mode::Overlay => vec![],
        }
    }
}
