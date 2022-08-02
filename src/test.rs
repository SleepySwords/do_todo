#[cfg(test)]
mod actions {
    use crossterm::event::KeyCode;

    use crate::{
        app::{App, TaskData},
        input,
        task::Task,
    };

    #[test]
    fn test_add_task() {
        let mut app = App::new(crate::theme::Theme::default(), TaskData::default());
        input::handle_input(KeyCode::Char('a'), &mut app);
        input::handle_input(KeyCode::Char('p'), &mut app);
        input::handle_input(KeyCode::Char('p'), &mut app);
        input::handle_input(KeyCode::Char('y'), &mut app);
        input::handle_input(KeyCode::Char('q'), &mut app);
        input::handle_input(KeyCode::Enter, &mut app);
        assert_eq!(app.task_data.tasks[0].title, "ppyq")
    }

    #[test]
    fn test_edit_task() {
        let mut app = App::new(
            crate::theme::Theme::default(),
            TaskData {
                tasks: vec![Task::from_string(String::from("meme"))],
                completed_tasks: vec![],
            },
        );
        input::handle_input(KeyCode::Char('e'), &mut app);
        input::handle_input(KeyCode::Char('r'), &mut app);
        input::handle_input(KeyCode::Char('q'), &mut app);
        input::handle_input(KeyCode::Enter, &mut app);
        assert_eq!(app.task_data.tasks[0].title, "memerq")
    }

    #[test]
    fn test_delete_task() {
        let mut app = App::new(
            crate::theme::Theme::default(),
            TaskData {
                tasks: vec![Task::from_string(String::from("meme"))],
                completed_tasks: vec![],
            },
        );
        input::handle_input(KeyCode::Char('d'), &mut app);
        input::handle_input(KeyCode::Enter, &mut app);
        assert_eq!(app.task_data.tasks.len(), 0)
    }

    #[test]
    fn test_cancel_delete_task() {
        let mut app = App::new(
            crate::theme::Theme::default(),
            TaskData {
                tasks: vec![Task::from_string(String::from("meme"))],
                completed_tasks: vec![],
            },
        );
        input::handle_input(KeyCode::Char('d'), &mut app);
        input::handle_input(KeyCode::Char('j'), &mut app);
        input::handle_input(KeyCode::Enter, &mut app);
        assert_eq!(app.task_data.tasks.len(), 1)
    }
}

#[cfg(test)]
mod movement {
    use chrono::Local;
    use crossterm::event::KeyCode;

    use crate::{
        app::{App, TaskData},
        input,
        task::{CompletedTask, Task},
    };

    #[test]
    fn test_rollover() {
        let mut app = App::new(
            crate::theme::Theme::default(),
            TaskData {
                tasks: vec![
                    Task::from_string(String::from("meme")),
                    Task::from_string(String::from("based")),
                ],
                completed_tasks: vec![
                    CompletedTask::from_string(String::from("hey"), Local::now().naive_local()),
                    CompletedTask::from_string(String::from("there"), Local::now().naive_local()),
                ],
            },
        );
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_task_index, 1);
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_task_index, 0);
        input::handle_input(KeyCode::Char('k'), &mut app);
        assert_eq!(app.selected_task_index, 1);

        input::handle_input(KeyCode::Char('2'), &mut app);
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_completed_task_index, 1);
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_completed_task_index, 0);
        input::handle_input(KeyCode::Char('k'), &mut app);
        assert_eq!(app.selected_task_index, 1);
    }

    #[test]
    fn test_movement_retention() {
        let mut app = App::new(
            crate::theme::Theme::default(),
            TaskData {
                tasks: vec![
                    Task::from_string(String::from("meme")),
                    Task::from_string(String::from("based")),
                ],
                completed_tasks: vec![
                    CompletedTask::from_string(String::from("hey"), Local::now().naive_local()),
                    CompletedTask::from_string(String::from("there"), Local::now().naive_local()),
                ],
            },
        );
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_task_index, 1);
        assert_eq!(app.selected_completed_task_index, 0);

        input::handle_input(KeyCode::Char('2'), &mut app);
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_task_index, 1);
        assert_eq!(app.selected_completed_task_index, 1);
    }

    #[test]
    fn test_no_data() {
        let mut app = App::new(
            crate::theme::Theme::default(),
            TaskData {
                tasks: vec![
                    Task::from_string(String::from("meme")),
                    Task::from_string(String::from("based")),
                ],
                completed_tasks: vec![
                    CompletedTask::from_string(String::from("hey"), Local::now().naive_local()),
                    CompletedTask::from_string(String::from("there"), Local::now().naive_local()),
                ],
            },
        );
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_task_index, 1);
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_task_index, 0);

        input::handle_input(KeyCode::Char('2'), &mut app);
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_completed_task_index, 1);
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_completed_task_index, 0);
    }

    #[test]
    fn test_one_task() {
        let mut app = App::new(
            crate::theme::Theme::default(),
            TaskData {
                tasks: vec![
                    Task::from_string(String::from("meme")),
                    Task::from_string(String::from("based")),
                ],
                completed_tasks: vec![
                    CompletedTask::from_string(String::from("hey"), Local::now().naive_local()),
                    CompletedTask::from_string(String::from("there"), Local::now().naive_local()),
                ],
            },
        );
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_task_index, 1);
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_task_index, 0);

        input::handle_input(KeyCode::Char('2'), &mut app);
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_completed_task_index, 1);
        input::handle_input(KeyCode::Char('j'), &mut app);
        assert_eq!(app.selected_completed_task_index, 0);
    }
}
