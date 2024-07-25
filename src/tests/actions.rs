use std::{cmp, collections::HashMap};

use chrono::Local;
use crossterm::event::KeyCode;
use itertools::Itertools;

use crate::{
    data::json_data_store::JsonDataStore,
    framework::screen_manager::ScreenManager,
    task::{CompletedTask, Priority, Task},
    utils::test::{input_char, input_code, setup},
};

const TEST_TASK_NAME: &str = "yay it works, test letters => abcdefghijklmnopqrstuvwxyz1234567890";

fn get_task<'a>(i: usize, screen_manager: &'a ScreenManager) -> Option<&'a Task> {
    let task_id = screen_manager.app.task_store.global_pos_to_task(i)?;
    return screen_manager.app.task_store.task(&task_id);
}

#[test]
fn test_add_task() {
    let mut main_app = setup(JsonDataStore::default());
    input_char('a', &mut main_app);

    TEST_TASK_NAME.chars().for_each(|chr| {
        input_char(chr, &mut main_app);
    });
    input_code(KeyCode::Enter, &mut main_app);
    assert_eq!(get_task(0, &main_app).unwrap().title, TEST_TASK_NAME)
}

#[test]
fn ok() {}

#[test]
fn test_cancel_add_task() {
    let mut main_app = setup(JsonDataStore::default());
    input_char('a', &mut main_app);

    TEST_TASK_NAME.chars().for_each(|chr| {
        input_char(chr, &mut main_app);
    });
    input_code(KeyCode::Esc, &mut main_app);
    assert_eq!(main_app.app.task_store.root_tasks().len(), 0)
}

#[test]
fn test_edit_task() {
    let mut tasks = HashMap::new();
    tasks.insert(
        "0".to_string(),
        Task::from_string(String::from(TEST_TASK_NAME)),
    );
    let mut main_app = setup(JsonDataStore {
        tasks,
        completed_tasks: HashMap::new(),
        tags: HashMap::new(),
        subtasks: HashMap::new(),
        root: vec!["0".to_string()],
        completed_root: vec![],
        task_count: 1,
    });
    input_char('e', &mut main_app);
    input_char('r', &mut main_app);
    input_char('q', &mut main_app);
    input_code(KeyCode::Enter, &mut main_app);
    assert_eq!(
        get_task(0, &main_app).unwrap().title,
        TEST_TASK_NAME.to_owned() + "rq"
    )
}

#[test]
fn test_edit_delete_task() {
    let mut tasks = HashMap::new();
    tasks.insert(
        "0".to_string(),
        Task::from_string(String::from(TEST_TASK_NAME)),
    );
    let mut main_app = setup(JsonDataStore {
        tasks,
        completed_tasks: HashMap::new(),
        tags: HashMap::new(),
        subtasks: HashMap::new(),
        root: vec!["0".to_string()],
        completed_root: vec![],
        task_count: 0,
    });
    input_char('e', &mut main_app);
    input_code(KeyCode::Backspace, &mut main_app);
    input_code(KeyCode::Backspace, &mut main_app);
    input_code(KeyCode::Enter, &mut main_app);
    assert_eq!(
        main_app.app.task_store.tasks[0].title,
        TEST_TASK_NAME[..TEST_TASK_NAME.len() - 2]
    )
}

#[test]
fn test_cancel_edit_task() {
    let mut main_app = setup(JsonDataStore {
        tasks: vec![Task::from_string(String::from("meme"))],
        completed_tasks: vec![],
        tags: HashMap::new(),
        auto_sort: false,
    });
    input_char('e', &mut main_app);
    input_code(KeyCode::Backspace, &mut main_app);
    input_char('r', &mut main_app);
    input_char('q', &mut main_app);
    input_code(KeyCode::Esc, &mut main_app);
    assert_eq!(main_app.app.task_store.tasks[0].title, "meme")
}

#[test]
fn test_delete_task() {
    let mut main_app = setup(JsonDataStore {
        tasks: vec![Task::from_string(String::from("meme"))],
        completed_tasks: vec![],
        tags: HashMap::new(),
        auto_sort: false,
    });
    input_char('d', &mut main_app);
    input_code(KeyCode::Enter, &mut main_app);
    assert_eq!(main_app.app.task_store.tasks.len(), 0)
}

#[test]
fn test_cancel_delete_task() {
    let mut main_app = setup(JsonDataStore {
        tasks: vec![Task::from_string(String::from("meme"))],
        completed_tasks: vec![],
        tags: HashMap::new(),
        auto_sort: false,
    });
    input_char('d', &mut main_app);
    input_char('j', &mut main_app);
    input_code(KeyCode::Enter, &mut main_app);
    assert_eq!(main_app.app.task_store.tasks.len(), 1)
}

#[test]
fn test_priority() {
    let mut main_app = setup(JsonDataStore {
        tasks: vec![
            Task::from_string(String::from("meme")),
            Task::from_string(String::from("oof")),
        ],
        completed_tasks: vec![],
        tags: HashMap::new(),
        auto_sort: false,
    });
    input_char('p', &mut main_app);
    assert_eq!(main_app.app.task_store.tasks[0].priority, Priority::High);
    input_char('p', &mut main_app);
    assert_eq!(main_app.app.task_store.tasks[0].priority, Priority::Normal);
    input_char('p', &mut main_app);
    assert_eq!(main_app.app.task_store.tasks[0].priority, Priority::Low);
    input_char('p', &mut main_app);
    assert_eq!(main_app.app.task_store.tasks[0].priority, Priority::None);

    input_char('j', &mut main_app);
    input_char('p', &mut main_app);
    assert_eq!(main_app.app.task_store.tasks[0].priority, Priority::None);
    assert_eq!(main_app.app.task_store.tasks[1].priority, Priority::High);
    input_char('p', &mut main_app);
    assert_eq!(main_app.app.task_store.tasks[0].priority, Priority::None);
    assert_eq!(main_app.app.task_store.tasks[1].priority, Priority::Normal);
    input_char('p', &mut main_app);
    assert_eq!(main_app.app.task_store.tasks[0].priority, Priority::None);
    assert_eq!(main_app.app.task_store.tasks[1].priority, Priority::Low);
    input_char('p', &mut main_app);
    assert_eq!(main_app.app.task_store.tasks[0].priority, Priority::None);
    assert_eq!(main_app.app.task_store.tasks[1].priority, Priority::None);
}

#[test]
fn test_complete_task() {
    let mut main_app = setup(JsonDataStore {
        tasks: vec![Task::from_string(String::from("meme"))],
        completed_tasks: vec![],
        tags: HashMap::new(),
        auto_sort: false,
    });
    input_char('c', &mut main_app);
    assert_eq!(main_app.app.task_store.tasks.len(), 0);
    assert_eq!(main_app.app.task_store.completed_tasks.len(), 1);
}

#[test]
fn test_restore_task() {
    let mut main_app = setup(JsonDataStore {
        tasks: vec![],
        completed_tasks: vec![CompletedTask::from_string(
            String::from("meme"),
            Local::now().naive_local(),
        )],
        tags: HashMap::new(),
        auto_sort: false,
    });
    input_char('2', &mut main_app);
    input_char('r', &mut main_app);
    assert_eq!(main_app.app.task_store.tasks.len(), 1);
    assert_eq!(main_app.app.task_store.completed_tasks.len(), 0);
}

#[test]
fn sort() {
    let mut main_app = setup(JsonDataStore {
        tasks: vec![
            Task {
                progress: false,
                title: String::from("Toaj"),
                priority: Priority::Low,
                tags: Vec::new(),
                ..Default::default()
            },
            Task {
                progress: false,
                title: String::from("Toajeoifj"),
                priority: Priority::High,
                tags: Vec::new(),
                ..Default::default()
            },
        ],
        completed_tasks: vec![CompletedTask::from_string(
            String::from("meme"),
            Local::now().naive_local(),
        )],
        tags: HashMap::new(),
        auto_sort: false,
    });
    input_char('s', &mut main_app);
    assert!(main_app
        .app
        .task_store
        .tasks
        .iter()
        .sorted_by_key(|t| cmp::Reverse(t.priority))
        .eq(main_app.app.task_store.tasks.iter()));
}

#[test]
fn test_autosort() {
    let mut main_app = setup(JsonDataStore {
        tasks: vec![
            Task {
                progress: false,
                title: String::from("Toaj"),
                priority: Priority::Low,
                tags: Vec::new(),
                ..Default::default()
            },
            Task {
                progress: false,
                title: String::from("Toajeoifj"),
                priority: Priority::High,
                tags: Vec::new(),
                ..Default::default()
            },
        ],
        completed_tasks: vec![CompletedTask::from_string(
            String::from("meme"),
            Local::now().naive_local(),
        )],
        tags: HashMap::new(),
        auto_sort: false,
    });
    input_char('S', &mut main_app);
    input_char('J', &mut main_app);
    assert!(main_app
        .app
        .task_store
        .tasks
        .iter()
        .sorted_by_key(|t| cmp::Reverse(t.priority))
        .eq(main_app.app.task_store.tasks.iter()));
}
