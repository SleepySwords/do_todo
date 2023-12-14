use std::collections::BTreeMap;

use crossterm::event::KeyCode;
use tui::style::Color;

use crate::{
    app::{App, TaskStore},
    component::layout::stack_layout::StackLayout,
    task::{Tag, Task},
    utils::test::{input_char, input_code, setup},
};

fn add_tag(app: &mut App, stack_layout: &mut StackLayout, name: &str, colour: &str) {
    input_char('t', app, stack_layout);
    input_code(KeyCode::Up, app, stack_layout);
    input_code(KeyCode::Up, app, stack_layout);
    input_code(KeyCode::Up, app, stack_layout);
    input_code(KeyCode::Up, app, stack_layout);
    input_code(KeyCode::Enter, app, stack_layout);

    name.chars()
        .for_each(|chr| input_char(chr, app, stack_layout));
    input_code(KeyCode::Enter, app, stack_layout);

    colour
        .chars()
        .for_each(|chr| input_char(chr, app, stack_layout));
    input_code(KeyCode::Enter, app, stack_layout);
}

#[test]
fn test_tag_creation() {
    const TEST_TAG: &str = "WOOO TAGS!!";

    let (mut app, mut stack_layout) = setup(TaskStore {
        tasks: vec![
            Task::from_string(String::from("meme")),
            Task::from_string(String::from("oof")),
        ],
        completed_tasks: vec![],
        tags: BTreeMap::new(),
        auto_sort: false,
    });

    let mut tag_count = 0;

    add_tag(&mut app, &mut stack_layout, TEST_TAG, "#aabbcc");
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

    add_tag(&mut app, &mut stack_layout, "Second tag", "Re-D");
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

    add_tag(&mut app, &mut stack_layout, TEST_TAG, "12");
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

    let (mut app, mut stack_layout) = setup(TaskStore {
        tasks: vec![
            Task::from_string(String::from("meme")),
            Task::from_string(String::from("oof")),
        ],
        completed_tasks: vec![],
        tags: BTreeMap::new(),
        auto_sort: false,
    });
    add_tag(&mut app, &mut stack_layout, TEST_TAG, "ewfnjaweknf");
    input_code(KeyCode::Enter, &mut app, &mut stack_layout);

    assert_eq!(app.task_store.tags.len(), 0);

    "12".chars()
        .for_each(|chr| input_char(chr, &mut app, &mut stack_layout));
    input_code(KeyCode::Enter, &mut app, &mut stack_layout);

    assert_eq!(app.task_store.tags.len(), 1);
}

#[test]
fn test_tag_removal() {
    const TEST_TAG: &str = "WOOO TAGS!!";

    let (mut app, mut stack_layout) = setup(TaskStore {
        tasks: vec![
            Task::from_string(String::from("meme")),
            Task::from_string(String::from("oof")),
        ],
        completed_tasks: vec![],
        tags: BTreeMap::new(),
        auto_sort: false,
    });
    add_tag(&mut app, &mut stack_layout, TEST_TAG, "1");
    assert_eq!(app.task_store.tags.len(), 1);

    input_char('t', &mut app, &mut stack_layout);
    input_code(KeyCode::Up, &mut app, &mut stack_layout);
    input_code(KeyCode::Up, &mut app, &mut stack_layout);
    input_code(KeyCode::Enter, &mut app, &mut stack_layout);
    input_code(KeyCode::Enter, &mut app, &mut stack_layout);
    input_code(KeyCode::Enter, &mut app, &mut stack_layout);

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
    let (mut app, mut stack_layout) = setup(TaskStore {
        tasks: vec![Task::from_string(String::from("oof"))],
        completed_tasks: vec![],
        tags,
        auto_sort: false,
    });

    input_char('t', &mut app, &mut stack_layout);
    input_code(KeyCode::Enter, &mut app, &mut stack_layout);

    assert_eq!(app.task_store.tasks[0].tags.len(), 1);

    input_char('t', &mut app, &mut stack_layout);
    input_code(KeyCode::Enter, &mut app, &mut stack_layout);

    assert_eq!(app.task_store.tasks[0].tags.len(), 0);
}
