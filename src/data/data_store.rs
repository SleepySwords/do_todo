use std::collections::HashMap;

use chrono::NaiveDateTime;

use crate::task::{CompletedTask, FindParentResult, Tag, Task};

pub type TaskID = String;

/// Handles how tasks are stored
pub trait DataTaskStore {
    /// Returns this mutable task with this id.
    fn task_mut(&mut self, id: &str) -> Option<&mut Task>;

    /// Returns the task with this id.
    fn task(&self, id: &str) -> Option<&Task>;

    /// Returns this mutable completed task with this id.
    fn completed_task_mut(&mut self, id: &str) -> Option<&mut CompletedTask>;

    /// Returns the completed task with this id.
    fn completed_task(&self, id: &str) -> Option<&CompletedTask>;

    /// Deletes the task with this id.
    fn delete_task(&mut self, id: &str) -> Option<Task>;

    /// Gets the parent of this task with this id.
    fn find_parent(&self, id: &str) -> Option<FindParentResult>;

    /// Returns the subtasks of a task if `id` is some
    /// Otherwise returns the global tasks.
    ///
    /// * `id` - The id to get, if None, will return global tasks
    fn subtasks_mut(&mut self, id: Option<&str>) -> Option<&mut Vec<TaskID>>;

    /// Returns the subtasks of a task if `id` is some
    /// Otherwise returns the global tasks.
    ///
    /// * `id` - The id to get, if None, will return global tasks
    fn subtasks(&self, id: &str) -> Option<&Vec<TaskID>>;

    /// Returns the subtasks of a task if `id` is some
    /// Otherwise returns the global tasks.
    ///
    /// * `id` - The id to get, if None, will return global tasks
    fn root_tasks(&self) -> &Vec<TaskID>;

    /// Returns the subtasks of a task if `id` is some
    /// Otherwise returns the global tasks.
    ///
    /// * `id` - The id to get, if None, will return global tasks
    fn completed_root_tasks(&self) -> &Vec<TaskID>;

    /// Finds the task at the global positon
    // FIXME: Should this be a global interface?
    // This is not really related to the implementation at heart.
    fn global_pos_to_task(&self, pos: usize) -> Option<TaskID>;

    /// Finds the task at the global positon
    fn global_pos_to_completed(&self, pos: usize) -> Option<TaskID>;

    fn task_to_global_pos(&self, id: &str) -> Option<usize>;

    fn delete_tag(&mut self, tag_id: &String);

    /// Sorts all the task based on priority
    fn sort(&mut self);

    /// Adds a task to this data store
    ///
    /// * `task` - The task to be added.
    /// * `parent` - The parent of the task to be added.
    fn add_task(&mut self, task: Task, parent: Option<&str>);

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
    fn move_task(&mut self, id: &str, parent: Option<TaskID>, order: usize, global: Option<()>);
    /// FIXME: global task moving?

    fn find_task_draw_size(&self, task_id: &str) -> usize;

    fn find_tasks_draw_size(&self) -> usize;

    fn complete_task(&mut self, id: &str, time_completed: NaiveDateTime);

    fn restore(&mut self, id: &str);

    fn tags(&self) -> &HashMap<String, Tag>;

    fn tags_mut(&mut self) -> &mut HashMap<String, Tag>;
}
