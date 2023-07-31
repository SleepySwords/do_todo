use std::collections::BTreeMap;

use crossterm::event::KeyCode;

use crate::{
    app::TaskStore,
    task::{Priority, Task},
    utils::test::{input_char, input_code, setup},
};

const TEST_TASK_NAME: &str = "yay it works, test letters => abcdefghijklmnopqrstuvwxyz1234567890";

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
        auto_sort: false,
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
        auto_sort: false,
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
        auto_sort: false,
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
        auto_sort: false,
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
        auto_sort: false,
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
        auto_sort: false,
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
