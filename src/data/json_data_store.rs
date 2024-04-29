use std::collections::HashMap;

use crate::task::{FindParentResult, Task};

use super::task_store::{DataTaskStore, TaskID};

struct JsonDataStore {
    // FIXME: can replace with a tree later.
    tasks: HashMap<TaskID, Task>,
    subtasks: HashMap<TaskID, Vec<String>>,
    root: Vec<TaskID>,
}

impl DataTaskStore for JsonDataStore {
    fn task_mut(&mut self, id: TaskID) -> Option<&mut Task> {
        return self.tasks.get_mut(&id);
    }

    fn task(&self, id: TaskID) -> Option<&Task> {
        return self.tasks.get(&id);
    }

    fn delete_task(&mut self, id: TaskID) -> Option<Task> {
        return self.tasks.remove(&id);
    }

    // FIXME: this is redunded with the move operator
    fn find_parent(&self, id: TaskID) -> Option<FindParentResult> {
        todo!()
    }

    // FIXME: might be able to wrap this in a &mut Vec<Task> perhaps?
    fn subtasks(&mut self, id: Option<TaskID>) -> Option<&mut Vec<TaskID>> {
        if let Some(id) = id {
            return self.subtasks.get_mut(&id);
        } else {
            return Some(&mut self.root);
        }
    }

    fn task_position(&self, id: TaskID) -> Option<usize> {
        todo!()
    }

    fn delete_tag(&mut self, tag_id: usize) {
        todo!()
    }

    fn sort(&mut self) {
        todo!("Sort all the subtasks vecs")
    }

    fn add_task(&mut self, task: crate::task::Task, parent: Option<TaskID>) {
        todo!()
    }

    fn refresh(&mut self) {
        todo!()
    }

    fn move_task(&mut self, id: TaskID, parent: Option<TaskID>, order: usize) {
        todo!()
    }
}
