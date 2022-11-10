#[cfg(test)]
mod actions {
    use std::collections::BTreeMap;

    use crossterm::event::KeyCode;

    use crate::{
        app::TaskStore,
        task::Task,
        utils::test::{input_char, input_code, setup},
    };

    #[test]
    fn test_add_task() {
        let (mut app, mut stack_layout) = setup(TaskStore::default());
        input_char('a', &mut app, &mut stack_layout);
        input_char('p', &mut app, &mut stack_layout);
        input_char('p', &mut app, &mut stack_layout);
        input_char('y', &mut app, &mut stack_layout);
        input_char('q', &mut app, &mut stack_layout);
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);
        println!("{}", app.task_store.tasks[0].title);
        assert_eq!(app.task_store.tasks[0].title, "ppyq")
    }

    #[test]
    fn test_edit_task() {
        let (mut app, mut stack_layout) = setup(TaskStore {
            tasks: vec![Task::from_string(String::from("meme"))],
            completed_tasks: vec![],
            tags: BTreeMap::new(),
        });
        input_char('e', &mut app, &mut stack_layout);
        input_char('r', &mut app, &mut stack_layout);
        input_char('q', &mut app, &mut stack_layout);
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);
        assert_eq!(app.task_store.tasks[0].title, "memerq")
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
