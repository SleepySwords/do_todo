#[cfg(test)]
mod actions {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::{
        app::{App, TaskData},
        input,
        task::Task,
        utils::test::input_char,
    };

    fn generate_event(key_code: KeyCode) -> KeyEvent {
        KeyEvent {
            code: key_code,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn test_add_task() {
        let mut app = App::new(crate::theme::Theme::default(), TaskData::default());
        input_char('a', &mut app);
        input_char('p', &mut app);
        input_char('p', &mut app);
        input_char('y', &mut app);
        input_char('q', &mut app);
        input::handle_key(generate_event(KeyCode::Enter), &mut app);
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
        input_char('e', &mut app);
        input_char('r', &mut app);
        input_char('q', &mut app);
        input::handle_key(generate_event(KeyCode::Enter), &mut app);
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
        input_char('d', &mut app);
        input::handle_key(generate_event(KeyCode::Enter), &mut app);
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
        input_char('d', &mut app);
        input_char('j', &mut app);
        input::handle_key(generate_event(KeyCode::Enter), &mut app);
        assert_eq!(app.task_data.tasks.len(), 1)
    }
}

#[cfg(test)]
mod movement {
    use crate::{
        app::{App, TaskData},
        task::Task,
        utils::test::input_char,
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
                completed_tasks: vec![],
            },
        );
        input_char('j', &mut app);
        input_char('j', &mut app);
        assert_eq!(app.selected_task_index, 0)
    }
}
