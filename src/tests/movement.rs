use std::collections::HashMap;

use crate::{
    app::App,
    config::Config,
    framework::screen_manager::ScreenManager,
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
            tags: HashMap::new(),
            auto_sort: false,
        },
    );
    let mut screen_manager = ScreenManager {
        app,
        overlays: vec![],
    };

    input_char('j', &mut screen_manager);
    let current_index = screen_manager.app.task_list.selected_index;
    assert_eq!(current_index, 1);

    input_char('j', &mut screen_manager);
    let current_index = screen_manager.app.task_list.selected_index;
    assert_eq!(current_index, 0);

    input_char('k', &mut screen_manager);
    let current_index = screen_manager.app.task_list.selected_index;
    assert_eq!(current_index, 1);

    input_char('k', &mut screen_manager);
    let current_index = screen_manager.app.task_list.selected_index;
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
            tags: HashMap::new(),
            auto_sort: false,
        },
    );
    let mut screen_manager = ScreenManager {
        app,
        overlays: vec![],
    };

    input_char('J', &mut screen_manager);
    assert_eq!(screen_manager.app.task_list.selected_index, 1);
    assert_task_eq(&screen_manager.app, vec!["based", "meme"]);

    input_char('J', &mut screen_manager);
    assert_eq!(screen_manager.app.task_list.selected_index, 0);
    assert_task_eq(&screen_manager.app, vec!["meme", "based"]);

    input_char('j', &mut screen_manager);

    input_char('K', &mut screen_manager);
    assert_eq!(screen_manager.app.task_list.selected_index, 0);
    assert_task_eq(&screen_manager.app, vec!["based", "meme"]);

    input_char('K', &mut screen_manager);
    assert_eq!(screen_manager.app.task_list.selected_index, 1);
    assert_task_eq(&screen_manager.app, vec!["meme", "based"]);
}
