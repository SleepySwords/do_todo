use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tui::style::Color;

use std::{fmt::Display, vec, cmp};

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
