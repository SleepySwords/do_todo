#[cfg(test)]
mod actions {
    use std::collections::BTreeMap;

    use crossterm::event::KeyCode;

    use crate::{
        app::TaskStore,
        task::{Priority, Task},
        utils::test::{input_char, input_code, setup},
    };

    const TEST_TASK_NAME: &str =
        "yay it works, test letters => abcdefghijklmnopqrstuvwxyz1234567890";

    #[test]
    fn test_add_task() {
        let (mut app, mut stack_layout) = setup(TaskStore::default());
        input_char('a', &mut app, &mut stack_layout);

        TEST_TASK_NAME.chars().for_each(|chr| {
            input_char(chr, &mut app, &mut stack_layout);
        });
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].title, TEST_TASK_NAME)
    }

    #[test]
    fn test_cancel_add_task() {
        let (mut app, mut stack_layout) = setup(TaskStore::default());
        input_char('a', &mut app, &mut stack_layout);

        TEST_TASK_NAME.chars().for_each(|chr| {
            input_char(chr, &mut app, &mut stack_layout);
        });
        input_code(KeyCode::Esc, &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks.len(), 0)
    }

    #[test]
    fn test_edit_task() {
        let (mut app, mut stack_layout) = setup(TaskStore {
            tasks: vec![Task::from_string(String::from(TEST_TASK_NAME))],
            completed_tasks: vec![],
            tags: BTreeMap::new(),
        });
        input_char('e', &mut app, &mut stack_layout);
        input_char('r', &mut app, &mut stack_layout);
        input_char('q', &mut app, &mut stack_layout);
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);
        assert_eq!(
            app.task_store.tasks[0].title,
            TEST_TASK_NAME.to_owned() + "rq"
        )
    }

    #[test]
    fn test_edit_delete_task() {
        let (mut app, mut stack_layout) = setup(TaskStore {
            tasks: vec![Task::from_string(String::from(TEST_TASK_NAME))],
            completed_tasks: vec![],
            tags: BTreeMap::new(),
        });
        input_char('e', &mut app, &mut stack_layout);
        input_code(KeyCode::Backspace, &mut app, &mut stack_layout);
        input_code(KeyCode::Backspace, &mut app, &mut stack_layout);
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);
        assert_eq!(
            app.task_store.tasks[0].title,
            TEST_TASK_NAME[..TEST_TASK_NAME.len() - 2]
        )
    }

    #[test]
    fn test_cancel_edit_task() {
        let (mut app, mut stack_layout) = setup(TaskStore {
            tasks: vec![Task::from_string(String::from("meme"))],
            completed_tasks: vec![],
            tags: BTreeMap::new(),
        });
        input_char('e', &mut app, &mut stack_layout);
        input_code(KeyCode::Backspace, &mut app, &mut stack_layout);
        input_char('r', &mut app, &mut stack_layout);
        input_char('q', &mut app, &mut stack_layout);
        input_code(KeyCode::Esc, &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].title, "meme")
    }

    #[test]
    fn test_delete_task() {
        let (mut app, mut stack_layout) = setup(TaskStore {
            tasks: vec![Task::from_string(String::from("meme"))],
            completed_tasks: vec![],
            tags: BTreeMap::new(),
        });
        input_char('d', &mut app, &mut stack_layout);
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks.len(), 0)
    }

    #[test]
    fn test_cancel_delete_task() {
        let (mut app, mut stack_layout) = setup(TaskStore {
            tasks: vec![Task::from_string(String::from("meme"))],
            completed_tasks: vec![],
            tags: BTreeMap::new(),
        });
        input_char('d', &mut app, &mut stack_layout);
        input_char('j', &mut app, &mut stack_layout);
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks.len(), 1)
    }

    #[test]
    fn test_priority() {
        let (mut app, mut stack_layout) = setup(TaskStore {
            tasks: vec![
                Task::from_string(String::from("meme")),
                Task::from_string(String::from("oof")),
            ],
            completed_tasks: vec![],
            tags: BTreeMap::new(),
        });
        input_char('h', &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].priority, Priority::High);
        input_char('h', &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].priority, Priority::Normal);
        input_char('h', &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].priority, Priority::Low);
        input_char('h', &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].priority, Priority::None);

        input_char('j', &mut app, &mut stack_layout);
        input_char('h', &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].priority, Priority::None);
        assert_eq!(app.task_store.tasks[1].priority, Priority::High);
        input_char('h', &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].priority, Priority::None);
        assert_eq!(app.task_store.tasks[1].priority, Priority::Normal);
        input_char('h', &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].priority, Priority::None);
        assert_eq!(app.task_store.tasks[1].priority, Priority::Low);
        input_char('h', &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].priority, Priority::None);
        assert_eq!(app.task_store.tasks[1].priority, Priority::None);
    }
}

#[cfg(test)]
mod tags {
    use std::collections::BTreeMap;

    use crossterm::event::KeyCode;
    use tui::style::Color;

    use crate::{
        app::TaskStore,
        task::Task,
        utils::test::{input_char, input_code, setup},
    };

    #[test]
    fn test_tag_creation() {
        const TEST_TAG: &str = "WOOO TAGS!!";

        let (mut app, mut stack_layout) = setup(TaskStore {
            tasks: vec![
                Task::from_string(String::from("meme")),
                Task::from_string(String::from("oof")),
            ],
            completed_tasks: vec![],
            tags: BTreeMap::new(),
        });

        input_char('t', &mut app, &mut stack_layout);
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);

        TEST_TAG
            .chars()
            .for_each(|chr| input_char(chr, &mut app, &mut stack_layout));
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);

        "aabbcc"
            .chars()
            .for_each(|chr| input_char(chr, &mut app, &mut stack_layout));
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);

        assert_eq!(app.task_store.tasks[0].tags.len(), 1);
        assert_eq!(
            app.task_store.tasks[0].first_tag(&app).unwrap().name,
            TEST_TAG
        );
        assert_eq!(
            app.task_store.tasks[0].first_tag(&app).unwrap().colour,
            Color::Rgb(170, 187, 204)
        );
    }
}

#[cfg(test)]
mod movement {
    use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

    use crate::{
        app::{App, TaskStore},
        component::{layout::stack_layout::StackLayout, task_list::TaskList},
        task::Task,
        theme::Theme,
        utils::test::input_char,
    };

    #[test]
    fn test_rollover() {
        let mut app = App::new(
            Theme::default(),
            TaskStore {
                tasks: vec![
                    Task::from_string(String::from("meme")),
                    Task::from_string(String::from("based")),
                ],
                completed_tasks: vec![],
                tags: BTreeMap::new(),
            },
        );
        let index = Rc::new(RefCell::new(0));
        let task_list = TaskList::new(index.clone());

        let mut stack_layout = StackLayout {
            children: vec![Box::new(task_list)],
        };
        input_char('j', &mut app, &mut stack_layout);
        input_char('j', &mut app, &mut stack_layout);
        let current_index = *index.borrow();
        assert_eq!(current_index, 0)
    }
}
