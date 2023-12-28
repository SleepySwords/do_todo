use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{
    app::{App, TaskStore},
    component::{layout::stack_layout::StackLayout, task_list::TaskList},
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
    let task_list = TaskList::new();

    let mut stack_layout = StackLayout {
        children: vec![Box::new(task_list)],
    };
    input_char('j', &mut app, &mut stack_layout);
    let current_index = *index.borrow();
    assert_eq!(current_index, 1);

    input_char('j', &mut app, &mut stack_layout);
    let current_index = *index.borrow();
    assert_eq!(current_index, 0);

    input_char('k', &mut app, &mut stack_layout);
    let current_index = *index.borrow();
    assert_eq!(current_index, 1);

    input_char('k', &mut app, &mut stack_layout);
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
    let task_list = TaskList::new();

    let mut stack_layout = StackLayout {
        children: vec![Box::new(task_list)],
    };
    input_char('J', &mut app, &mut stack_layout);
    assert_task_cursor_eq(&index, 1);
    assert_task_eq(&app, vec!["based", "meme"]);

    input_char('J', &mut app, &mut stack_layout);
    assert_task_cursor_eq(&index, 0);
    assert_task_eq(&app, vec!["meme", "based"]);

    input_char('j', &mut app, &mut stack_layout);

    input_char('K', &mut app, &mut stack_layout);
    assert_task_cursor_eq(&index, 0);
    assert_task_eq(&app, vec!["based", "meme"]);

    input_char('K', &mut app, &mut stack_layout);
    assert_task_cursor_eq(&index, 1);
    assert_task_eq(&app, vec!["meme", "based"]);
}
