use std::collections::HashMap;

use crossterm::event::KeyCode;
use tui::style::Color;

use crate::{
    data::{data_store::DataTaskStore, json_data_store::JsonDataStore},
    framework::screen_manager::ScreenManager,
    task::{Tag, Task},
    utils::test::{get_task_from_pos, input_char, input_code, setup},
};

fn add_tag(screen_manager: &mut ScreenManager, name: &str, colour: &str) {
    input_char('t', screen_manager);
    input_code(KeyCode::Up, screen_manager);
    input_code(KeyCode::Up, screen_manager);
    input_code(KeyCode::Up, screen_manager);
    input_code(KeyCode::Up, screen_manager);
    input_code(KeyCode::Up, screen_manager);
    input_code(KeyCode::Enter, screen_manager);

    name.chars().for_each(|chr| input_char(chr, screen_manager));
    input_code(KeyCode::Enter, screen_manager);

    colour
        .chars()
        .for_each(|chr| input_char(chr, screen_manager));
    input_code(KeyCode::Enter, screen_manager);
}

#[test]
fn test_tag_creation() {
    const TEST_TAG: &str = "WOOO TAGS!!";

    let mut json_data_store = JsonDataStore::default();
    json_data_store.add_task(Task::from_string("meme"), None);
    json_data_store.add_task(Task::from_string("oof"), None);
    let mut screen_manager = setup(json_data_store);

    let mut tag_count = 0;

    add_tag(&mut screen_manager, TEST_TAG, "#aabbcc");
    tag_count += 1;

    assert_eq!(
        get_task_from_pos(&*screen_manager.app.task_store, 0)
            .tags
            .len(),
        tag_count
    );
    assert_eq!(
        get_task_from_pos(&*screen_manager.app.task_store, 0)
            .first_tag(&screen_manager.app)
            .unwrap()
            .name,
        TEST_TAG
    );
    assert_eq!(
        get_task_from_pos(&*screen_manager.app.task_store, 0)
            .first_tag(&screen_manager.app)
            .unwrap()
            .colour,
        Color::Rgb(170, 187, 204)
    );

    add_tag(&mut screen_manager, "Second tag", "Re-D");
    tag_count += 1;

    assert_eq!(
        get_task_from_pos(&*screen_manager.app.task_store, 0)
            .tags
            .len(),
        tag_count
    );
    assert_eq!(
        screen_manager
            .app
            .task_store
            .tags()
            .get(
                get_task_from_pos(&*screen_manager.app.task_store, 0)
                    .tags
                    .last()
                    .unwrap()
            )
            .unwrap()
            .name,
        "Second tag"
    );
    assert_eq!(
        screen_manager
            .app
            .task_store
            .tags()
            .get(
                get_task_from_pos(&*screen_manager.app.task_store, 0)
                    .tags
                    .last()
                    .unwrap()
            )
            .unwrap()
            .colour,
        Color::Red
    );

    add_tag(&mut screen_manager, TEST_TAG, "12");
    tag_count += 1;

    assert_eq!(
        get_task_from_pos(&*screen_manager.app.task_store, 0)
            .tags
            .len(),
        tag_count
    );
    assert_eq!(
        screen_manager
            .app
            .task_store
            .tags()
            .get(
                get_task_from_pos(&*screen_manager.app.task_store, 0)
                    .tags
                    .last()
                    .unwrap()
            )
            .unwrap()
            .name,
        TEST_TAG
    );
    assert_eq!(
        screen_manager
            .app
            .task_store
            .tags()
            .get(
                get_task_from_pos(&*screen_manager.app.task_store, 0)
                    .tags
                    .last()
                    .unwrap()
            )
            .unwrap()
            .colour,
        Color::Indexed(12)
    );
}

#[test]
fn test_tag_cancel_and_enter() {
    const TEST_TAG: &str = "WOOO TAGS!!";

    let mut json_data_store = JsonDataStore::default();
    json_data_store.add_task(Task::from_string("meme"), None);
    json_data_store.add_task(Task::from_string("oof"), None);
    let mut screen_manager = setup(json_data_store);
    add_tag(&mut screen_manager, TEST_TAG, "ewfnjaweknf");

    input_code(KeyCode::Enter, &mut screen_manager);

    assert_eq!(screen_manager.app.task_store.tags().len(), 0);

    "12".chars()
        .for_each(|chr| input_char(chr, &mut screen_manager));
    input_code(KeyCode::Enter, &mut screen_manager);

    assert_eq!(screen_manager.app.task_store.tags().len(), 1);
}

#[test]
fn test_tag_removal() {
    const TEST_TAG: &str = "WOOO TAGS!!";

    let mut json_data_store = JsonDataStore::default();
    json_data_store.add_task(Task::from_string("meme"), None);
    json_data_store.add_task(Task::from_string("oof"), None);
    let mut screen_manager = setup(json_data_store);
    add_tag(&mut screen_manager, TEST_TAG, "1");
    assert_eq!(screen_manager.app.task_store.tags().len(), 1);

    input_char('t', &mut screen_manager);
    input_code(KeyCode::Up, &mut screen_manager);
    input_code(KeyCode::Up, &mut screen_manager);
    input_code(KeyCode::Enter, &mut screen_manager);
    input_code(KeyCode::Enter, &mut screen_manager);
    input_code(KeyCode::Enter, &mut screen_manager);

    assert_eq!(screen_manager.app.task_store.tags().len(), 0);
}

#[test]
fn test_flip_tag() {
    let mut tags = HashMap::new();
    tags.insert(
        "0".to_string(),
        Tag {
            name: String::from("test"),
            colour: Color::Red,
        },
    );
    let mut json_data_store = JsonDataStore {
        tags,
        ..Default::default()
    };
    json_data_store.add_task(Task::from_string("meme"), None);
    json_data_store.add_task(Task::from_string("oof"), None);
    let mut screen_manager = setup(json_data_store);

    input_char('t', &mut screen_manager);
    input_code(KeyCode::Enter, &mut screen_manager);

    assert_eq!(
        get_task_from_pos(&*screen_manager.app.task_store, 0)
            .tags
            .len(),
        1
    );

    input_char('t', &mut screen_manager);
    input_code(KeyCode::Enter, &mut screen_manager);

    assert_eq!(
        get_task_from_pos(&*screen_manager.app.task_store, 0)
            .tags
            .len(),
        0
    );
}
