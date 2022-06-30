#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;

    use crate::{
        app::{App, TaskData},
        input,
        task::Task,
    };

    // #[test]
    // fn test_start_app() -> io::Result<()> {
    //     // let mut stdout = stdout();
    //     // execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    //     // let backend = CrosstermBackend::new(stdout);
    //     // let mut terminal = Terminal::new(backend)?;
    //     // let mut app = App::new(
    //     //     config::get_config().unwrap().0,
    //     //     config::get_config().unwrap().1,
    //     // );
    //     // start_app(&mut app, &mut terminal)
    // }

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
        input::handle_input(KeyCode::Enter, &mut app);
        assert_eq!(app.task_data.tasks[0].title, "memer")
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
