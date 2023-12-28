use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{
    app::{App, TaskStore},
    component::task_list::TaskList,
    task::Task,
    tests::{assert_task_cursor_eq, assert_task_eq},
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
            auto_sort: false,
        },
    );
    let index = Rc::new(RefCell::new(0));
    let _task_list = TaskList::new();

    input_char('j', &mut app);
    let current_index = *index.borrow();
    assert_eq!(current_index, 1);

    input_char('j', &mut app);
    let current_index = *index.borrow();
    assert_eq!(current_index, 0);

    input_char('k', &mut app);
    let current_index = *index.borrow();
    assert_eq!(current_index, 1);

    input_char('k', &mut app);
    let current_index = *index.borrow();
    assert_eq!(current_index, 0);
}

#[test]
fn test_shifting_tasks() {
    let mut app = App::new(
        Theme::default(),
        TaskStore {
            tasks: vec![
                Task::from_string(String::from("meme")),
                Task::from_string(String::from("based")),
            ],
            completed_tasks: vec![],
            tags: BTreeMap::new(),
            auto_sort: false,
        },
    );
    let index = Rc::new(RefCell::new(0));
    let _task_list = TaskList::new();

    input_char('J', &mut app);
    assert_task_cursor_eq(&index, 1);
    assert_task_eq(&app, vec!["based", "meme"]);

    input_char('J', &mut app);
    assert_task_cursor_eq(&index, 0);
    assert_task_eq(&app, vec!["meme", "based"]);

    input_char('j', &mut app);

    input_char('K', &mut app);
    assert_task_cursor_eq(&index, 0);
    assert_task_eq(&app, vec!["based", "meme"]);

    input_char('K', &mut app);
    assert_task_cursor_eq(&index, 1);
    assert_task_eq(&app, vec!["meme", "based"]);
}
