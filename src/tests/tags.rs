use std::collections::BTreeMap;

use crossterm::event::KeyCode;
use tui::style::Color;

use crate::{
    app::App,
    task::{Tag, Task, TaskStore},
    utils::test::{input_char, input_code, setup},
};

fn add_tag(app: &mut App, name: &str, colour: &str) {
    input_char('t', app);
    input_code(KeyCode::Up, app);
    input_code(KeyCode::Up, app);
    input_code(KeyCode::Up, app);
    input_code(KeyCode::Up, app);
    input_code(KeyCode::Enter, app);

    name.chars().for_each(|chr| input_char(chr, app));
    input_code(KeyCode::Enter, app);

    colour.chars().for_each(|chr| input_char(chr, app));
    input_code(KeyCode::Enter, app);
}

#[test]
fn test_tag_creation() {
    const TEST_TAG: &str = "WOOO TAGS!!";

    let mut app = setup(TaskStore {
        tasks: vec![
            Task::from_string(String::from("meme")),
            Task::from_string(String::from("oof")),
        ],
        completed_tasks: vec![],
        tags: BTreeMap::new(),
        auto_sort: false,
    });

    let mut tag_count = 0;

    add_tag(&mut app, TEST_TAG, "#aabbcc");
    tag_count += 1;

    assert_eq!(app.task_store.tasks[0].tags.len(), tag_count);
    assert_eq!(
        app.task_store.tasks[0].first_tag(&app).unwrap().name,
        TEST_TAG
    );
    assert_eq!(
        app.task_store.tasks[0].first_tag(&app).unwrap().colour,
        Color::Rgb(170, 187, 204)
    );

    add_tag(&mut app, "Second tag", "Re-D");
    tag_count += 1;

    assert_eq!(app.task_store.tasks[0].tags.len(), tag_count);
    assert_eq!(
        app.task_store
            .tags
            .get(app.task_store.tasks[0].tags.last().unwrap())
            .unwrap()
            .name,
        "Second tag"
    );
    assert_eq!(
        app.task_store
            .tags
            .get(app.task_store.tasks[0].tags.last().unwrap())
            .unwrap()
            .colour,
        Color::Red
    );

    add_tag(&mut app, TEST_TAG, "12");
    tag_count += 1;

    assert_eq!(app.task_store.tasks[0].tags.len(), tag_count);
    assert_eq!(
        app.task_store
            .tags
            .get(app.task_store.tasks[0].tags.last().unwrap())
            .unwrap()
            .name,
        TEST_TAG
    );
    assert_eq!(
        app.task_store
            .tags
            .get(app.task_store.tasks[0].tags.last().unwrap())
            .unwrap()
            .colour,
        Color::Indexed(12)
    );
}

#[test]
fn test_tag_cancel_and_enter() {
    const TEST_TAG: &str = "WOOO TAGS!!";

    let mut app = setup(TaskStore {
        tasks: vec![
            Task::from_string(String::from("meme")),
            Task::from_string(String::from("oof")),
        ],
        completed_tasks: vec![],
        tags: BTreeMap::new(),
        auto_sort: false,
    });
    add_tag(&mut app, TEST_TAG, "ewfnjaweknf");
    input_code(KeyCode::Enter, &mut app);

    assert_eq!(app.task_store.tags.len(), 0);

    "12".chars().for_each(|chr| input_char(chr, &mut app));
    input_code(KeyCode::Enter, &mut app);

    assert_eq!(app.task_store.tags.len(), 1);
}

#[test]
fn test_tag_removal() {
    const TEST_TAG: &str = "WOOO TAGS!!";

    let mut app = setup(TaskStore {
        tasks: vec![
            Task::from_string(String::from("meme")),
            Task::from_string(String::from("oof")),
        ],
        completed_tasks: vec![],
        tags: BTreeMap::new(),
        auto_sort: false,
    });
    add_tag(&mut app, TEST_TAG, "1");
    assert_eq!(app.task_store.tags.len(), 1);

    input_char('t', &mut app);
    input_code(KeyCode::Up, &mut app);
    input_code(KeyCode::Up, &mut app);
    input_code(KeyCode::Enter, &mut app);
    input_code(KeyCode::Enter, &mut app);
    input_code(KeyCode::Enter, &mut app);

    assert_eq!(app.task_store.tags.len(), 0);
}

#[test]
fn test_flip_tag() {
    let mut tags = BTreeMap::new();
    tags.insert(
        0,
        Tag {
            name: String::from("test"),
            colour: Color::Red,
        },
    );
    let mut app = setup(TaskStore {
        tasks: vec![Task::from_string(String::from("oof"))],
        completed_tasks: vec![],
        tags,
        auto_sort: false,
    });

    input_char('t', &mut app);
    input_code(KeyCode::Enter, &mut app);

    assert_eq!(app.task_store.tasks[0].tags.len(), 1);

    input_char('t', &mut app);
    input_code(KeyCode::Enter, &mut app);

    assert_eq!(app.task_store.tasks[0].tags.len(), 0);
}
