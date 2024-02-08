use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use tui::style::Color;

use std::{cmp, collections::BTreeMap, fmt::Display, vec};

use crate::{
    app::App,
    config::{color_parser, Config},
};

#[derive(Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
    #[serde(with = "color_parser")]
    pub colour: Color,
}

#[derive(Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Task {
    pub progress: bool,
    pub title: String,
    pub priority: Priority,
    pub tags: Vec<usize>,
    pub date_to_complete: Option<NaiveDate>,

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
            date_to_complete: None,
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
            .filter_map(|tag_index| return app.task_store.tags.get(tag_index))
    }

    pub fn flip_tag(&mut self, tag: usize) {
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

    pub fn delete_tag(&mut self, tag_id: usize) {
        self.sub_tasks.sort_by_key(|t| cmp::Reverse(t.priority));
        self.tags.retain(|f| f != &tag_id);
        for task in &mut self.sub_tasks {
            task.delete_tag(tag_id)
        }
    }

    /// Also includes the current task in draw size.
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
                date_to_complete: None,
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
    pub tags: BTreeMap<usize, Tag>,
    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<CompletedTask>,
    pub auto_sort: bool,
}

pub struct FindParentResult<'a> {
    pub tasks: &'a Vec<Task>,
    pub parent_index: Option<usize>,
    pub task_local_offset: usize,
}

impl TaskStore {
    pub fn task_mut(&mut self, mut global_index: usize) -> Option<&mut Task> {
        self.tasks
            .iter_mut()
            .find_map(|t| Self::internal_task_mut(t, &mut global_index))
    }

    fn internal_task_mut<'a>(task: &'a mut Task, selected: &mut usize) -> Option<&'a mut Task> {
        if *selected == 0 {
            return Some(task);
        }

        *selected -= 1;

        if !task.opened {
            return None;
        }

        task.sub_tasks
            .iter_mut()
            .find_map(|t| Self::internal_task_mut(t, selected))
    }

    pub fn task(&self, mut global_index: usize) -> Option<&Task> {
        self.tasks
            .iter()
            .find_map(|t| Self::internal_task(t, &mut global_index))
    }

    fn internal_task<'a>(task: &'a Task, selected: &mut usize) -> Option<&'a Task> {
        if *selected == 0 {
            return Some(task);
        }

        *selected -= 1;

        if !task.opened {
            return None;
        }

        task.sub_tasks
            .iter()
            .find_map(|t| Self::internal_task(t, selected))
    }

    pub fn delete_task(&mut self, to_delete: usize) -> Option<Task> {
        Self::internal_delete_task(&mut self.tasks, &mut 0, to_delete)
    }

    fn internal_delete_task(
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

            if let Some(task) = Self::internal_delete_task(
                &mut tasks[task_index].sub_tasks,
                current_index,
                to_delete,
            ) {
                return Some(task);
            }
        }
        None
    }

    pub fn find_tasks_draw_size(&self) -> usize {
        self.tasks
            .iter()
            .map(|t| t.find_task_draw_size())
            .sum::<usize>()
    }

    pub fn local_index_to_global(
        index: usize,
        parent_list: &[Task],
        parent_global_index: Option<usize>,
    ) -> usize {
        if let Some(parent_index) = parent_global_index {
            parent_index
                + parent_list
                .iter()
                .take(index)
                .map(|tsk| tsk.find_task_draw_size())
                .sum::<usize>()
                // Need to add one to focus the element, otherwise it won't
                // this is only for tasks within tasks.
                + 1
        } else {
            parent_list
                .iter()
                .take(index)
                .map(|tsk| tsk.find_task_draw_size())
                .sum::<usize>()
        }
    }

    /// Returns an option tuple
    /// The first is the parent subtasks
    /// Second is the parent_index
    /// Third is the tasks local offset
    /// Fourth is if it is a boolean
    pub fn find_parent(&self, to_find: usize) -> Option<FindParentResult> {
        Self::internal_find_parent(&self.tasks, &mut 0, to_find, 0, true)
    }

    fn internal_find_parent<'a>(
        tasks: &'a Vec<Task>,
        current_index: &mut usize,
        to_find: usize,
        index: usize,
        is_global: bool,
    ) -> Option<FindParentResult<'a>> {
        for task_index in 0..tasks.len() {
            if *current_index == to_find {
                // return Some((tasks, index, task_index, is_global));
                return Some(FindParentResult {
                    tasks,
                    parent_index: if is_global { None } else { Some(index) },
                    task_local_offset: task_index,
                });
            }

            let index = *current_index;

            *current_index += 1;

            if tasks[task_index].opened {
                if let Some(task) = Self::internal_find_parent(
                    &tasks[task_index].sub_tasks,
                    current_index,
                    to_find,
                    index,
                    false,
                ) {
                    return Some(task);
                }
            }
        }
        None
    }

    /// Returns the subtasks of a task if `is_global` is true
    /// Otherwise returns the global tasks.
    pub fn subtasks(&mut self, index: Option<usize>) -> Option<&mut Vec<Task>> {
        if let Some(index) = index {
            Some(&mut self.task_mut(index)?.sub_tasks)
        } else {
            Some(&mut self.tasks)
        }
    }

    pub fn task_position(&self, to_find: &Task) -> Option<usize> {
        let mut index = 0;
        self.tasks
            .iter()
            .find_map(|tsk| Self::internal_task_pos(to_find, tsk, &mut index))
    }

    fn internal_task_pos(to_find: &Task, current_task: &Task, index: &mut usize) -> Option<usize> {
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
            .find_map(|sub_task| Self::internal_task_pos(to_find, sub_task, index))
    }

    pub fn delete_tag(&mut self, tag_id: usize) {
        self.tags.remove(&tag_id);
        for task in &mut self.tasks {
            task.delete_tag(tag_id);
        }
        for completed_task in &mut self.completed_tasks {
            completed_task.task.delete_tag(tag_id);
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
