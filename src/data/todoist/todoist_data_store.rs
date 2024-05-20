use std::{cmp, collections::HashMap};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{
    data::data_store::{DataTaskStore, TaskID}, task::{CompletedTask, FindParentResult, Tag, Task}
};

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct TodoistDataStore {
    pub tasks: HashMap<TaskID, Task>,
    pub completed_tasks: HashMap<TaskID, CompletedTask>,
    pub subtasks: HashMap<TaskID, Vec<String>>,
    pub root: Vec<String>,
    pub completed_root: Vec<TaskID>,
    pub tags: HashMap<String, Tag>,
    pub task_count: usize,
}

impl TodoistDataStore {
    fn _global_pos_to_task(&self, selected: &mut usize, task_id: &str) -> Option<TaskID> {
        if *selected == 0 {
            return Some(task_id.to_string());
        }
        *selected -= 1;

        let task = self.task(task_id)?;
        if !task.opened {
            return None;
        }
        let subtasks = self.subtasks(task_id)?;
        subtasks
            .iter()
            .find_map(|f| self._global_pos_to_task(selected, f))
    }

    fn _task_to_global(
        &self,
        current_index: &mut usize,
        to_find: &str,
        curr: &str,
    ) -> Option<()> {
        if to_find == curr {
            return Some(());
        }
        *current_index += 1;
        let t = self.task(curr)?;
        if !t.opened {
            return None;
        }
        if let Some(subtasks) = self.subtasks(curr) {
            for task in subtasks {
                if task == to_find {
                    return Some(());
                }
                if let Some(()) = self._task_to_global(current_index, to_find, task) {
                    return Some(());
                }
            }
        }
        None
    }
}

impl DataTaskStore for TodoistDataStore {
    fn task_mut(&mut self, id: &str) -> Option<&mut Task> {
        return self.tasks.get_mut(id);
    }

    fn task(&self, id: &str) -> Option<&Task> {
        return self.tasks.get(id);
    }

    fn completed_task_mut(&mut self, id: &str) -> Option<&mut CompletedTask> {
        return self.completed_tasks.get_mut(id);
    }

    fn completed_task(&self, id: &str) -> Option<&CompletedTask> {
        return self.completed_tasks.get(id);
    }

    fn delete_task(&mut self, id: &str) -> Option<Task> {
        self.root.retain(|f| f != id);
        self.subtasks
            .values_mut()
            .for_each(|val| val.retain(|f| f != id));
        self.tasks.remove(id)
    }

    fn find_parent(&self, id: &str) -> Option<FindParentResult> {
        let parent = self
            .subtasks
            .iter()
            .find(|(_, subs)| subs.contains(&id.to_string()))
            .map(|p| p.0);
        let local_index = if let Some(p) = parent {
            self.subtasks.get(p).unwrap().iter().position(|t| t == id)
        } else {
            self.root.iter().position(|t| t == id)
        };
        Some(FindParentResult {
            task_local_offset: local_index?,
            parent_id: parent.cloned(),
        })
    }

    // FIXME: might be able to wrap this in a &mut Vec<Task> perhaps?
    fn subtasks_mut(&mut self, id: Option<&str>) -> Option<&mut Vec<TaskID>> {
        if let Some(id) = id {
            return self.subtasks.get_mut(id);
        } else {
            Some(&mut self.root)
        }
    }

    fn subtasks(&self, id: &str) -> Option<&Vec<TaskID>> {
        return self.subtasks.get(id);
    }

    fn root_tasks(&self) -> &Vec<TaskID> {
        &self.root
    }

    fn completed_root_tasks(&self) -> &Vec<TaskID> {
        &self.completed_root
    }

    fn global_pos_to_task(&self, mut pos: usize) -> Option<TaskID> {
        return self
            .root
            .iter()
            .find_map(|f| self._global_pos_to_task(&mut pos, f));
    }

    fn global_pos_to_completed(&self, mut pos: usize) -> Option<TaskID> {
        return self
            .completed_root_tasks()
            .iter()
            .find_map(|f| self._global_pos_to_task(&mut pos, f));
    }

    fn delete_tag(&mut self, tag_id: &String) {
        self.tags.remove(tag_id);
        for task in &mut self.tasks.values_mut() {
            task.tags.retain(|f| f != tag_id);
        }
        for completed_task in &mut self.completed_tasks.values_mut() {
            completed_task.task.tags.retain(|f| f != tag_id);
        }
    }

    fn sort(&mut self) {
        self.root.sort_by_key(|f| cmp::Reverse(self.tasks[f].priority));
        for subtasks in self.subtasks.values_mut() {
            subtasks.sort_by_key(|f| cmp::Reverse(self.tasks[f].priority));
        }
    }

    fn add_task(&mut self, task: Task, parent: Option<&str>) {
        let parents = if let Some(parent_id) = parent {
            self.subtasks.get_mut(parent_id).unwrap()
        } else {
            &mut self.root
        };
        let key = (self.task_count + 1).to_string();
        self.task_count += 1;
        self.tasks.insert(key.clone(), task);
        parents.push(key);
    }

    fn refresh(&mut self) {
        todo!()
    }

    fn save(&self) {
        // data_io::save_task_json(self);
    }

    fn move_task(&mut self, id: &str, parent: Option<TaskID>, order: usize, global: Option<()>) {
        let hash_map = &mut self.subtasks;
        let subtasks = if let Some((_, subtasks)) = hash_map
            .iter_mut()
            .find(|(_, subtasks)| subtasks.contains(&id.to_string()))
        {
            subtasks
        } else {
            &mut self.root
        };
        if let Some(p) = parent {
            subtasks.retain(|f| f != id);
            if let Some(subtasks) = self.subtasks_mut(Some(&p)) {
                subtasks.insert(order, id.to_string());
            } else {
                self.subtasks.insert(p, vec![id.to_string()]);
            }
        } else if global.is_some() {
            subtasks.retain(|f| f != id);
            self.root.insert(order, id.to_string());
        } else {
            subtasks.retain(|f| f != id);
            subtasks.insert(order, id.to_string());
        }
    }

    fn find_task_draw_size(&self, task_id: &str) -> usize {
        if let Some(task) = self.task(task_id) {
            return if !task.opened {
                0
            } else {
                self.subtasks(task_id)
                    .map_or(0, |f| f.iter().map(|k| self.find_task_draw_size(k)).sum())
            } + 1;
        }
        0
    }

    fn find_tasks_draw_size(&self) -> usize {
        self.root_tasks()
            .iter()
            .map(|t| self.find_task_draw_size(t))
            .sum()
    }

    fn complete_task(&mut self, id: &str, time_completed: NaiveDateTime) {
        self.root.retain(|f| f != id);
        self.subtasks
            .values_mut()
            .for_each(|subtasks| subtasks.retain(|f| f != id));
        if let Some(task) = self.tasks.remove(id) {
            self.completed_tasks.insert(
                id.to_string(),
                CompletedTask::from_task(task, time_completed),
            );
            self.completed_root.push(id.to_string());
        }
    }

    fn restore(&mut self, id: &str) {
        self.completed_root.retain(|f| f != id);
        self.subtasks
            .values_mut()
            .for_each(|subtasks| subtasks.retain(|f| f != id));
        if let Some(task) = self.completed_tasks.remove(id) {
            self.tasks.insert(id.to_string(), task.task);
            self.root.push(id.to_string());
        }
    }

    fn tags(&self) -> &HashMap<String, Tag> {
        &self.tags
    }

    fn tags_mut(&mut self) -> &mut HashMap<String, Tag> {
        &mut self.tags
    }

    fn task_to_global_pos(&self, id: &str) -> Option<usize> {
        let mut current_index = 0;
        for curr in &self.root {
            if let Some(()) = self._task_to_global(&mut current_index, id, curr) {
                return Some(current_index);
            }
        }
        None
    }
}
