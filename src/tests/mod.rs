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
#[cfg(test)]
mod tags;

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
