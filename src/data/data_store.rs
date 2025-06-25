use std::collections::HashMap;

use chrono::NaiveDateTime;
use enum_dispatch::enum_dispatch;

use crate::task::{CompletedTask, FindParentResult, Tag, Task};

use super::json_data_store::JsonDataStore;
use super::todoist::todoist_data_store::TodoistDataStore;

pub type TaskID = String;
pub type TaskIDRef<'a> = &'a str;

#[enum_dispatch]
pub enum DataTaskStoreKind {
    Json(JsonDataStore),
    Todoist(TodoistDataStore),
}

/// Handles how tasks are stored
#[enum_dispatch(DataTaskStoreKind)]
pub trait DataTaskStore {
    /// Returns this mutable task with this id.
    fn modify_task<F, T: FnOnce(&mut Task) -> F>(&mut self, id: TaskIDRef, closure: T)
        -> Option<F>;

    /// Notify the server that this task has been modified.
    fn update_task(&mut self, id: TaskIDRef);

    /// Returns the task with this id.
    fn task(&self, id: TaskIDRef) -> Option<&Task>;

    /// Returns this mutable completed task with this id.
    fn completed_task_mut(&mut self, id: TaskIDRef) -> Option<&mut CompletedTask>;

    /// Returns the completed task with this id.
    fn completed_task(&self, id: TaskIDRef) -> Option<&CompletedTask>;

    /// Deletes the task with this id.
    fn delete_task(&mut self, id: TaskIDRef) -> Option<Task>;

    /// Gets the parent of this task with this id.
    fn find_parent(&self, id: TaskIDRef) -> Option<FindParentResult>;

    /// Returns the subtasks of a task if `id` is some
    /// Otherwise returns the global tasks.
    ///
    /// * `id` - The id to get, if None, will return root tasks
    fn subtasks_mut(&mut self, id: Option<TaskIDRef>) -> Option<&mut Vec<TaskID>>;

    /// Returns the subtasks of a task if `id` is some
    /// Otherwise returns the global tasks.
    ///
    /// * `id` - The id to get, if None, will return root tasks
    fn subtasks(&self, id: TaskIDRef) -> Option<&Vec<TaskID>>;

    /// Returns the subtasks of a task if `id` is some
    /// Otherwise returns the global tasks.
    ///
    /// * `id` - The id to get, if None, will return root tasks
    fn root_tasks(&self) -> &Vec<TaskID>;

    /// Returns the subtasks of a task if `id` is some
    /// Otherwise returns the global tasks.
    ///
    /// * `id` - The id to get, if None, will return root tasks
    fn completed_root_tasks(&self) -> &Vec<TaskID>;

    fn delete_tag(&mut self, tag_id: TaskIDRef);

    /// Sorts all the task based on priority
    fn sort(&mut self);

    /// Adds a task to this data store
    ///
    /// * `task` - The task to be added.
    /// * `parent` - The parent of the task to be added.
    fn add_task(&mut self, task: Task, parent: Option<TaskIDRef>);

    /// Fetches data from the data source
    fn refresh(&mut self);

    /// Saves the data to the data source
    fn save(&self);

    /// Move a task from one place to another
    ///
    /// * `id` - The id of the task to be moved
    /// * `parent` - If specified, where the task should be moved to
    /// * `order` - What place should the task be placed within the order.
    ///
    fn move_task(
        &mut self,
        id: TaskIDRef,
        parent: Option<TaskID>,
        order: usize,
        global: Option<()>,
    );
    /// FIXME: global task moving?

    fn find_task_draw_size(&self, id: TaskIDRef) -> usize;

    fn find_tasks_draw_size(&self) -> usize;

    fn complete_task(&mut self, id: TaskIDRef, time_completed: NaiveDateTime);

    fn restore(&mut self, id: TaskIDRef);

    fn tags(&self) -> &HashMap<String, Tag>;

    fn tags_mut(&mut self) -> &mut HashMap<String, Tag>;

    fn is_syncing(&self) -> bool;
}
