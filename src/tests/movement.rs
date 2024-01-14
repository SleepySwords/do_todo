use std::collections::BTreeMap;

use crate::{
    app::{App, MainApp},
    config::Config,
    task::{Task, TaskStore},
    tests::assert_task_eq,
    utils::test::input_char,
};

#[test]
fn test_rollover() {
    let app = App::new(
        Config::default(),
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
    let mut main_app = MainApp {
        app,
        overlays: vec![]
    };

    input_char('j', &mut main_app);
    let current_index = main_app.app.task_list.selected_index;
    assert_eq!(current_index, 1);

    input_char('j', &mut main_app);
    let current_index = main_app.app.task_list.selected_index;
    assert_eq!(current_index, 0);

    input_char('k', &mut main_app);
    let current_index = main_app.app.task_list.selected_index;
    assert_eq!(current_index, 1);

    input_char('k', &mut main_app);
    let current_index = main_app.app.task_list.selected_index;
    assert_eq!(current_index, 0);
}

#[test]
fn test_shifting_tasks() {
    let app = App::new(
        Config::default(),
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
    let mut main_app = MainApp {
        app: app,
        overlays: vec![]
    };

    input_char('J', &mut main_app);
    assert_eq!(main_app.app.task_list.selected_index, 1);
    assert_task_eq(&main_app.app, vec!["based", "meme"]);

    input_char('J', &mut main_app);
    assert_eq!(main_app.app.task_list.selected_index, 0);
    assert_task_eq(&main_app.app, vec!["meme", "based"]);

    input_char('j', &mut main_app);

    input_char('K', &mut main_app);
    assert_eq!(main_app.app.task_list.selected_index, 0);
    assert_task_eq(&main_app.app, vec!["based", "meme"]);

    input_char('K', &mut main_app);
    assert_eq!(main_app.app.task_list.selected_index, 1);
    assert_task_eq(&main_app.app, vec!["meme", "based"]);
}
