use std::collections::BTreeMap;

use crate::{
    app::{App, TaskStore},
    component::task_list::TaskList,
    task::Task,
    tests::assert_task_eq,
    theme::Config,
    utils::test::input_char,
};

#[test]
fn test_rollover() {
    let mut app = App::new(
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
    let _task_list = TaskList::new();

    input_char('j', &mut app);
    let current_index = app.task_list.selected_index;
    assert_eq!(current_index, 1);

    input_char('j', &mut app);
    let current_index = app.task_list.selected_index;
    assert_eq!(current_index, 0);

    input_char('k', &mut app);
    let current_index = app.task_list.selected_index;
    assert_eq!(current_index, 1);

    input_char('k', &mut app);
    let current_index = app.task_list.selected_index;
    assert_eq!(current_index, 0);
}

#[test]
fn test_shifting_tasks() {
    let mut app = App::new(
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
    let _task_list = TaskList::new();

    input_char('J', &mut app);
    assert_eq!(app.task_list.selected_index, 1);
    assert_task_eq(&app, vec!["based", "meme"]);

    input_char('J', &mut app);
    assert_eq!(app.task_list.selected_index, 0);
    assert_task_eq(&app, vec!["meme", "based"]);

    input_char('j', &mut app);

    input_char('K', &mut app);
    assert_eq!(app.task_list.selected_index, 0);
    assert_task_eq(&app, vec!["based", "meme"]);

    input_char('K', &mut app);
    assert_eq!(app.task_list.selected_index, 1);
    assert_task_eq(&app, vec!["meme", "based"]);
}
