use std::{cell::RefCell, rc::Rc};

use itertools::Itertools;
use tui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::{Constraint, Layout},
    style::Style,
    Terminal,
};

use crate::{
    app::App,
    component::layout::stack_layout::StackLayout,
    draw::{DrawFrame, DrawableComponent, Drawer},
};

#[cfg(test)]
mod actions;
#[cfg(test)]
mod movement;
#[cfg(test)]
mod visual;

fn assert_task_eq(app: &App, task_names: Vec<&str>) {
    assert_eq!(
        app.task_store
            .tasks
            .iter()
            .map(|f| f.title.clone())
            .collect_vec(),
        task_names
    );
}

fn assert_task_cursor_eq(current: &Rc<RefCell<usize>>, selected_index: usize) {
    let current_index = *current.borrow();
    assert_eq!(current_index, selected_index);
}

fn generate_buffer(buffer: &Buffer, format_styles: Vec<Style>) -> String {
    let mut current_style = Style::default();
    let mut drawn_screen = String::new();

    for cells in buffer.content().chunks(buffer.area.width as usize) {
        for (x, cell) in cells.iter().enumerate() {
            let style_index = format_styles
                .iter()
                .position(|f| cell.style().eq(f))
                .map(|f| f.to_string())
                .unwrap_or(format!("unexpected: {:?}", cell.style()));
            if x == 0 {
                drawn_screen.push_str(&format!("{{{}:{}", style_index, cell.symbol));
            } else if current_style != cell.style() {
                drawn_screen.push_str(&format!("}}{{{}:{}", style_index, cell.symbol));
            } else {
                drawn_screen.push_str(&cell.symbol);
            }
            current_style = cell.style();
        }
        drawn_screen.push_str("}}");
        drawn_screen.push('\n');
    }
    drawn_screen
}

fn assert_screen(
    app: &mut App,
    stack_layout: &mut StackLayout,
    format_styles: Vec<Style>,
    expect: &str,
) {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let expect_result = expect.trim().split('\n').collect_vec();

    terminal
        .draw(|f| {
            let draw_size = f.size();

            let mut draw_frame = DrawFrame::TestFrame(f);
            let mut drawer = Drawer::new(&mut draw_frame);

            let chunk = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(draw_size);

            stack_layout.update_layout(chunk[0]);
            stack_layout.draw(app, &mut drawer);

            app.status_line.update_layout(chunk[1]);
            app.status_line.draw(app, &mut drawer);
        })
        .unwrap();

    let mut failed = false;
    let buffer = generate_buffer(terminal.backend().buffer(), format_styles);
    let buffered = buffer.trim().split('\n').collect_vec();
    let mut expected = String::new();

    for y in 0..expect_result.len() {
        if *buffered.get(y).unwrap_or(&"") != expect_result[y] {
            failed = true;
            expected.push_str(&format!("*{}\n", expect_result[y]));
        } else {
            expected.push_str(&format!("{}\n", expect_result[y]));
        }
    }

    if failed {
        let mut debug_info = format!(
            "Screen does not match. {}\n\n",
            if buffered.len() != expect_result.len() {
                format!(
                    "Screen size does not match, expected {}, actual {}.",
                    expect_result.len(),
                    buffered.len()
                )
            } else {
                "".to_string()
            }
        );
        debug_info.push_str("Expected:\n");
        debug_info.push_str(&expected);
        debug_info.push_str("\n\nFound:\n");
        debug_info.push_str(&buffer);
        panic!("{}", debug_info);
    }
}

#[cfg(test)]
mod tags {
    use std::collections::BTreeMap;

    use crossterm::event::KeyCode;
    use tui::style::Color;

    use crate::{
        app::{App, TaskStore},
        component::layout::stack_layout::StackLayout,
        task::Task,
        utils::test::{input_char, input_code, setup},
    };

    fn add_tag(app: &mut App, stack_layout: &mut StackLayout, name: &str, colour: &str) {
        input_char('t', app, stack_layout);
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
        input_code(KeyCode::Down, &mut app, &mut stack_layout);
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);
        input_code(KeyCode::Enter, &mut app, &mut stack_layout);

        assert_eq!(app.task_store.tags.len(), 0);
    }
}
