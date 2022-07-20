#[cfg(test)]
mod input {
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
