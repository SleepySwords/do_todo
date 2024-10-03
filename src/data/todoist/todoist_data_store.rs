use std::{
    cmp,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::NaiveDateTime;
use tokio::sync::mpsc::Sender;

use crate::{
    data::data_store::{DataTaskStore, TaskID, TaskIDRef},
    task::{CompletedTask, FindParentResult, Tag, Task},
    utils,
};

use super::todoist_command::{
    task_to_todoist, TodoistCommand, TodoistItemAddCommand, TodoistItemDeleteCommand,
    TodoistItemReorder, TodoistItemReorderCommand, TodoistUpdateItem,
};

pub struct TodoistDataStore {
    pub tasks: HashMap<TaskID, Task>,
    pub completed_tasks: HashMap<TaskID, CompletedTask>,
    pub subtasks: HashMap<TaskID, Vec<TaskID>>,
    pub root: Vec<TaskID>,
    pub completed_root: Vec<TaskID>,
    pub tags: HashMap<String, Tag>,
    pub task_count: usize,

    pub currently_syncing: Arc<Mutex<bool>>,
    pub command_sender: Sender<TodoistCommand>,
}

impl DataTaskStore for TodoistDataStore {
    // FIXME: replace task_mut with an edit_task
    // If we use a shared string library tasks should be cheap to make.
    //
    //
    // Actually, just assume we are always modifying the task.
    fn task_mut(&mut self, id: TaskIDRef) -> Option<&mut Task> {
        return self.tasks.get_mut(id);
    }

    fn update_task(&mut self, id: TaskIDRef) {
        let command = TodoistCommand::ItemUpdate {
            uuid: uuid::Uuid::new_v4().to_string(),
            args: task_to_todoist(id.to_string(), self.task(id).unwrap()),
        };

        let sender = self.command_sender.clone();
        tokio::spawn(async move {
            sender.send(command).await.unwrap();
        });
    }

    fn task(&self, id: TaskIDRef) -> Option<&Task> {
        return self.tasks.get(id);
    }

    fn completed_task_mut(&mut self, id: TaskIDRef) -> Option<&mut CompletedTask> {
        return self.completed_tasks.get_mut(id);
    }

    fn completed_task(&self, id: TaskIDRef) -> Option<&CompletedTask> {
        return self.completed_tasks.get(id);
    }

    fn delete_task(&mut self, id: TaskIDRef) -> Option<Task> {
        self.root.retain(|f| f != id);
        self.subtasks
            .values_mut()
            .for_each(|val| val.retain(|f| f != id));

        let command = TodoistCommand::ItemDelete {
            uuid: uuid::Uuid::new_v4().to_string(),
            args: TodoistItemDeleteCommand { id: id.to_string() },
        };

        let sender = self.command_sender.clone();
        tokio::spawn(async move {
            sender.send(command).await.unwrap();
        });

        self.tasks.remove(id)
    }

    fn find_parent(&self, id: TaskIDRef) -> Option<FindParentResult> {
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
    // Similar to the comment above, if we make strings cheap, it will
    // be relatively cheap to clone this and have this function replace the
    // inner version.
    fn subtasks_mut(&mut self, id: Option<TaskIDRef>) -> Option<&mut Vec<TaskID>> {
        if let Some(id) = id {
            return self.subtasks.get_mut(id);
        } else {
            Some(&mut self.root)
        }
    }

    fn subtasks(&self, id: TaskIDRef) -> Option<&Vec<TaskID>> {
        return self.subtasks.get(id);
    }

    fn root_tasks(&self) -> &Vec<TaskID> {
        &self.root
    }

    fn completed_root_tasks(&self) -> &Vec<TaskID> {
        &self.completed_root
    }

    fn cursor_to_task(&self, pos: usize) -> Option<TaskID> {
        utils::task_position::cursor_to_task(self, pos)
    }

    fn cursor_to_completed_task(&self, pos: usize) -> Option<TaskID> {
        utils::task_position::cursor_to_completed_task(self, pos)
    }

    fn task_to_cursor(&self, id: TaskIDRef) -> Option<usize> {
        utils::task_position::task_to_cursor(self, id)
    }

    fn delete_tag(&mut self, tag_id: TaskIDRef) {
        self.tags.remove(tag_id);
        for task in &mut self.tasks.values_mut() {
            task.tags.retain(|f| f != tag_id);
        }
        for completed_task in &mut self.completed_tasks.values_mut() {
            completed_task.task.tags.retain(|f| f != tag_id);
        }
    }

    fn sort(&mut self) {
        self.root
            .sort_by_key(|f| cmp::Reverse(self.tasks[f].priority));
        for subtasks in self.subtasks.values_mut() {
            subtasks.sort_by_key(|f| cmp::Reverse(self.tasks[f].priority));
        }
    }

    fn add_task(&mut self, task: Task, parent: Option<TaskIDRef>) {
        let parents = if let Some(parent_id) = parent {
            // FIXME: consider writing ugly version that avoids a clone.
            self.subtasks.entry(parent_id.to_string()).or_default()
        } else {
            &mut self.root
        };
        let key = uuid::Uuid::new_v4().to_string();
        self.task_count += 1;
        self.tasks.insert(key.clone(), task.clone());
        parents.push(key.clone());

        let command = TodoistCommand::ItemAdd {
            uuid: uuid::Uuid::new_v4().to_string(),
            temp_id: key,
            args: TodoistItemAddCommand {
                content: task.title.to_string(),
                parent_id: parent.map(|f| f.to_string()),
            },
        };

        let sender = self.command_sender.clone();
        tokio::spawn(async move {
            sender.send(command).await.unwrap();
            // println!("{:?}", sync);
        });
    }

    fn refresh(&mut self) {
        todo!()
    }

    fn save(&self) {
        // data_io::save_task_json(self);
    }

    fn move_task(
        &mut self,
        id: TaskIDRef,
        parent: Option<TaskID>,
        order: usize,
        global: Option<()>,
    ) {
        let hash_map = &mut self.subtasks;
        let subtasks = if let Some((_, subtasks)) = hash_map
            .iter_mut()
            .find(|(_, subtasks)| subtasks.contains(&id.to_string()))
        {
            subtasks
        } else {
            &mut self.root
        };
        let previous_id = subtasks.get(order).map(|f| f.to_string());
        let previous_position = subtasks.iter().position(|f| f == id);
        subtasks.retain(|f| f != id);
        let mutable_subtasks = if let Some(p) = parent {
            self.subtasks.entry(p).or_insert(vec![])
        } else if global.is_some() {
            &mut self.root
        } else {
            subtasks
        };

        mutable_subtasks.insert(order, id.to_string());
        let mut items = vec![TodoistItemReorder {
            id: id.to_string(),
            child_order: order,
        }];

        if let (Some(previous_id), Some(previous_position)) = (previous_id, previous_position) {
            items.push(TodoistItemReorder {
                id: previous_id,
                child_order: previous_position,
            })
        }

        let command = TodoistCommand::ItemReorder {
            uuid: uuid::Uuid::new_v4().to_string(),
            args: TodoistItemReorderCommand { items },
        };
        // FIXME: this does not work for some reason.
        let sender = self.command_sender.clone();
        tokio::spawn(async move {
            sender.send(command).await.unwrap();
            // println!("{:?}", sync);
        });
    }

    fn find_task_draw_size(&self, task_id: TaskIDRef) -> usize {
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

    fn complete_task(&mut self, id: TaskIDRef, time_completed: NaiveDateTime) {
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

    fn restore(&mut self, id: TaskIDRef) {
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

    fn is_syncing(&self) -> bool {
        self.currently_syncing.lock().map_or(false, |f| *f)
    }
}
