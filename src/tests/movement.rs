use crate::{
    data::{data_store::DataTaskStore, json_data_store::JsonDataStore},
    task::Task,
    tests::assert_task_eq,
    utils::test::{input_char, setup},
};

#[test]
fn test_rollover() {
    let mut json_data_store = JsonDataStore::default();
    json_data_store.add_task(Task::from_string("meme"), None);
    json_data_store.add_task(Task::from_string("oof"), None);
    let mut screen_manager = setup(json_data_store);

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
    let mut json_data_store = JsonDataStore::default();
    json_data_store.add_task(Task::from_string("meme"), None);
    json_data_store.add_task(Task::from_string("based"), None);
    let mut screen_manager = setup(json_data_store);

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
