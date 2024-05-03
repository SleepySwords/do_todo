use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    data_io,
    task::{CompletedTask, FindParentResult, Tag, Task},
};

use super::task_store::{DataTaskStore, TaskID};

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct JsonDataStore {
    pub tasks: HashMap<TaskID, Task>,
    pub completed_tasks: HashMap<TaskID, CompletedTask>,
    pub subtasks: HashMap<TaskID, Vec<String>>,
    pub root: Vec<TaskID>,
    pub completed_root: Vec<TaskID>,
    pub tags: HashMap<String, Tag>,
    pub task_count: usize,
}

impl JsonDataStore {
    fn _global_pos_to_task(&self, selected: &mut usize, task_id: &TaskID) -> Option<TaskID> {
        if *selected == 0 {
            return Some(task_id.clone());
        }
        *selected -= 1;

        let Some(task) = self.task(task_id) else {
            return None;
        };
        if !task.opened {
            return None;
        }
        let Some(subtasks) = self.subtasks.get(task_id) else {
            return None;
        };
        subtasks
            .iter()
            .find_map(|f| self._global_pos_to_task(selected, f))
    }

    fn _find_tasks_draw_size(&self, task_id: &TaskID) -> usize {
        if let Some(task) = self.task(task_id) {
            return if !task.opened {
                1
            } else {
                self.subtasks(task_id).map_or(1, |f| {
                    f.iter().map(|k| 1 + self._find_tasks_draw_size(k)).sum()
                })
            };
        }
        return 0;
    }
}

impl DataTaskStore for JsonDataStore {
    fn task_mut(&mut self, id: &TaskID) -> Option<&mut Task> {
        return self.tasks.get_mut(id);
    }

    fn task(&self, id: &TaskID) -> Option<&Task> {
        return self.tasks.get(id);
    }

    fn completed_task_mut(&mut self, id: &TaskID) -> Option<&mut CompletedTask> {
        return self.completed_tasks.get_mut(id);
    }

    fn completed_task(&self, id: &TaskID) -> Option<&CompletedTask> {
        return self.completed_tasks.get(id);
    }

    fn delete_task(&mut self, id: &TaskID) -> Option<Task> {
        self.root.retain(|f| f != id);
        return self.tasks.remove(id);
    }

    // FIXME: this is redunded with the move operator
    fn find_parent(&self, id: TaskID) -> Option<FindParentResult> {
        todo!()
    }

    // FIXME: might be able to wrap this in a &mut Vec<Task> perhaps?
    fn subtasks_mut(&mut self, id: Option<&TaskID>) -> Option<&mut Vec<TaskID>> {
        if let Some(id) = id {
            return self.subtasks.get_mut(id);
        } else {
            return Some(&mut self.root);
        }
    }

    fn subtasks(&self, id: &TaskID) -> Option<&Vec<TaskID>> {
        return self.subtasks.get(id);
    }

    fn root_tasks(&self) -> &Vec<TaskID> {
        return &self.root;
    }

    fn completed_root_tasks(&self) -> &Vec<TaskID> {
        return &self.completed_root;
    }

    fn global_pos_to_task(&self, mut pos: usize) -> Option<TaskID> {
        return self
            .root
            .iter()
            .find_map(|f| self._global_pos_to_task(&mut pos, f));
    }

    fn global_pos_to_completed(&self, pos: usize) -> Option<TaskID> {
        todo!()
    }

    fn delete_tag(&mut self, tag_id: &String) {
        self.tags.remove(tag_id);
        for (_, task) in &mut self.tasks {
            task.tags.retain(|f| f == tag_id);
        }
        for (_, completed_task) in &mut self.completed_tasks {
            completed_task.task.tags.retain(|f| f == tag_id);
        }
    }

    fn sort(&mut self) {
        todo!("Sort all the subtasks vecs")
    }

    fn add_task(&mut self, task: Task, parent: Option<TaskID>) {
        let parents = if let Some(parent_id) = parent {
            self.subtasks.get_mut(&parent_id).unwrap()
        } else {
            &mut self.root
        };
        let key = (self.tasks.len() + self.completed_tasks.len() + 1).to_string();
        self.tasks.insert(key.clone(), task);
        parents.push(key);
    }

    fn refresh(&mut self) {
        todo!()
    }

    fn move_task(&mut self, id: TaskID, parent: Option<TaskID>, order: usize) {
        todo!()
    }

    fn find_tasks_draw_size(&self) -> usize {
        self.root_tasks()
            .iter()
            .map(|t| self._find_tasks_draw_size(t))
            .sum()
    }

    fn complete_task(&self, id: TaskID, data: chrono::prelude::NaiveDateTime) -> usize {
        todo!()
    }

    fn tags(&self) -> &HashMap<String, Tag> {
        return &self.tags;
    }

    fn tags_mut(&mut self) -> &mut HashMap<String, Tag> {
        return &mut self.tags;
    }

    fn save(&self) {
        data_io::save_task_json(self);
    }
}
