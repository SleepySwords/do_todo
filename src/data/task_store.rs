use crate::task::{FindParentResult, Task};

/// Handles how tasks are stored
trait DataTaskStore {
    /// Returns this mutable task with this id.
    fn task_mut(&mut self, id: String) -> Option<&mut dyn Task2>;

    /// Returns the task with this id.
    fn task(&self, id: String) -> Option<&dyn Task2>;

    /// Deletes the task with this id.
    fn delete_task(&mut self, id: String) -> Option<Task>;

    /// Gets the parent of this task with this id.
    fn find_parent(&self, id: String) -> Option<FindParentResult>;

    /// Returns the subtasks of a task if `id` is some
    /// Otherwise returns the global tasks.
    ///
    /// * `id` - The id to get, if None, will return global tasks
    fn subtasks(&mut self, id: Option<String>) -> Option<&mut Vec<Task>>;

    /// Finds the tasks global positon
    fn task_position(&self, id: String) -> Option<usize>;

    fn delete_tag(&mut self, tag_id: usize);

    fn sort(&mut self);

    /// Adds a task to this data store
    ///
    /// * `task` - The task to be added.
    /// * `parent` - The parent of the task to be added.
    fn add_task(&mut self, task: Task, parent: Option<String>);

    /// Fetches data from the data source
    fn refresh(&mut self);

    /// Move a task from one place to another
    ///
    /// * `id` - The id of the task to be moved
    /// * `parent` - If specified, where the task should be moved to
    /// * `order` - What place should the task be placed within the order.
    fn move_task(&mut self, id: String, parent: Option<String>, order: usize);
}

/// Interface for a task.
trait Task2 {
    // Maybe parent?
}
