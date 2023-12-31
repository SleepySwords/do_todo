use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tui::style::Color;

use std::{cmp, collections::BTreeMap, fmt::Display, vec};

use crate::{app::App, config::Config};

#[derive(Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
    pub colour: Color,
}

#[derive(Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Task {
    pub progress: bool,
    pub title: String,
    pub priority: Priority,
    pub tags: Vec<u32>,

    // Ignored if sub_tasks is empty
    pub opened: bool,
    pub sub_tasks: Vec<Task>,
}

impl Task {
    pub fn from_string(content: String) -> Self {
        Task {
            progress: false,
            title: content,
            priority: Priority::None,
            tags: Vec::new(),
            opened: true,
            sub_tasks: vec![],
        }
    }

    pub fn first_tag<'a>(&self, app: &'a App) -> Option<&'a Tag> {
        app.task_store.tags.get(self.tags.first().unwrap())
    }

    pub fn iter_tags<'a>(&'a self, app: &'a App) -> impl Iterator<Item = &'a Tag> + '_ {
        self.tags
            .iter()
            // FIXME: Remove tags from submenus, this is a hack for now, as new tags can share old
            // indicies
            .filter_map(|tag_index| return app.task_store.tags.get(tag_index))
    }

    pub fn flip_tag(&mut self, tag: u32) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag)
        } else {
            self.tags.retain(|x| x != &tag);
        }
    }

    pub fn from_completed_task(completed_task: CompletedTask) -> Self {
        completed_task.task
    }

    pub fn sort_subtasks(&mut self) {
        self.sub_tasks.sort_by_key(|t| cmp::Reverse(t.priority));
        for task in &mut self.sub_tasks {
            task.sort_subtasks()
        }
    }

    pub fn _find_selected_mut<'a>(&'a mut self, selected: &mut usize) -> Option<&'a mut Task> {
        if *selected == 0 {
            return Some(self);
        }

        *selected -= 1;

        if !self.opened {
            return None;
        }

        self.sub_tasks
            .iter_mut()
            .find_map(|t| t._find_selected_mut(selected))
    }

    pub fn _find_selected<'a>(&'a self, selected: &mut usize) -> Option<&'a Task> {
        if *selected == 0 {
            return Some(self);
        }

        *selected -= 1;

        if !self.opened {
            return None;
        }

        self.sub_tasks
            .iter()
            .find_map(|t| t._find_selected(selected))
    }

    // Includes this current task
    pub fn find_task_draw_size(&self) -> usize {
        (if self.opened {
            self.sub_tasks
                .iter()
                .map(|t| t.find_task_draw_size())
                .sum::<usize>()
        } else {
            0
        }) + 1
    }
}

#[derive(Deserialize, Serialize)]
pub struct CompletedTask {
    pub task: Task,
    pub time_completed: NaiveDateTime,
}

impl CompletedTask {
    pub fn from_task(task: Task, time_completed: NaiveDateTime) -> Self {
        CompletedTask {
            task,
            time_completed,
        }
    }

    pub fn from_string(content: String, time_completed: NaiveDateTime) -> Self {
        CompletedTask {
            task: Task {
                progress: false,
                title: content,
                priority: Priority::None,
                tags: Vec::new(),
                opened: true,
                sub_tasks: vec![],
            },
            time_completed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Default)]
pub enum Priority {
    #[default]
    None,
    Low,
    Normal,
    High,
}

impl Priority {
    pub fn display_string(&self) -> &str {
        match *self {
            Priority::None => "None",
            Priority::High => "High",
            Priority::Normal => "Normal",
            Priority::Low => "Low",
        }
    }

    pub fn short_hand(&self) -> &str {
        match *self {
            Priority::None => "    ",
            Priority::High => "!!! ",
            Priority::Normal => "!!  ",
            Priority::Low => "!   ",
        }
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Priority::None => write!(f, "None"),
            Priority::High => write!(f, "High"),
            Priority::Normal => write!(f, "Normal"),
            Priority::Low => write!(f, "Low"),
        }
    }
}

impl Priority {
    pub fn colour(&self, theme: &Config) -> Color {
        match self {
            Priority::None => Color::White,
            Priority::High => theme.high_priority_colour,
            Priority::Normal => theme.normal_priority_colour,
            Priority::Low => theme.low_priority_colour,
        }
    }

    pub fn next_priority(&self) -> Priority {
        match self {
            Priority::None => Priority::High,
            Priority::High => Priority::Normal,
            Priority::Normal => Priority::Low,
            Priority::Low => Priority::None,
        }
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

    fn _delete_task(
        tasks: &mut Vec<Task>,
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

    pub fn find_task_draw_size(&self) -> usize {
        self.tasks
            .iter()
            .map(|t| t.find_task_draw_size())
            .sum::<usize>()
    }

    pub fn local_index_to_global(
        index: usize,
        parent_list: &Vec<Task>,
        parent_global_offset: usize,
        is_global: bool,
    ) -> usize {
        parent_global_offset
                + parent_list
                    .iter()
                    .take(index)
                    .map(|tsk| tsk.find_task_draw_size())
                    .sum::<usize>()
                    // Need to add one to focus the element, otherwise it won't
                    // this is only for tasks within tasks.
                + if is_global { 0 } else { 1 }
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
        let (_, offset, is_global) = Self::find_parent(self, to_find)?;
        if is_global {
            Some((&mut self.tasks, offset, is_global))
        } else {
            Some((&mut self.task_mut(offset)?.sub_tasks, offset, is_global))
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
            .find_map(|tsk| Self::_task_position(to_find, tsk, &mut index))
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
