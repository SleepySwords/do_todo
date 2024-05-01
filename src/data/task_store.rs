use crate::task::{CompletedTask, FindParentResult, Task};

pub type TaskID = String;

/// Handles how tasks are stored
pub trait DataTaskStore {
    /// Returns this mutable task with this id.
    fn task_mut(&mut self, id: &TaskID) -> Option<&mut Task>;

    /// Returns the task with this id.
    fn task(&self, id: &TaskID) -> Option<&Task>;

    /// Returns this mutable completed task with this id.
    fn completed_task_mut(&mut self, id: &TaskID) -> Option<&mut CompletedTask>;

    /// Returns the completed task with this id.
    fn completed_task(&self, id: &TaskID) -> Option<&CompletedTask>;

    /// Deletes the task with this id.
    fn delete_task(&mut self, id: String) -> Option<Task>;

    /// Gets the parent of this task with this id.
    fn find_parent(&self, id: String) -> Option<FindParentResult>;

    /// Returns the subtasks of a task if `id` is some
    /// Otherwise returns the global tasks.
    ///
    /// * `id` - The id to get, if None, will return global tasks
    fn subtasks(&mut self, id: Option<&TaskID>) -> Option<&mut Vec<TaskID>>;

    /// Finds the task at the global positon
    fn global_pos_to_task(&self, pos: usize) -> Option<TaskID>;

    fn delete_tag(&mut self, tag_id: String);

    /// Sorts all the task based on priority
    fn sort(&mut self);

    /// Adds a task to this data store
    ///
    /// * `task` - The task to be added.
    /// * `parent` - The parent of the task to be added.
    fn add_task(&mut self, task: Task, parent: Option<TaskID>);

    /// Fetches data from the data source
    fn refresh(&mut self);

    /// Move a task from one place to another
    ///
    /// * `id` - The id of the task to be moved
    /// * `parent` - If specified, where the task should be moved to
    /// * `order` - What place should the task be placed within the order.
    fn move_task(&mut self, id: TaskID, parent: Option<TaskID>, order: usize);
}
