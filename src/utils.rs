use crossterm::event::{KeyCode, MouseEvent, MouseEventKind};
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Row, Table},
};
use unicode_segmentation::UnicodeSegmentation;

use std::{char, usize};

use crate::{
    app::{App, Mode},
    draw::EventResult,
};

// Only available for percentages, ratios and length
pub fn centre_rect(constraint_x: Constraint, constraint_y: Constraint, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(centre_constraints(constraint_y, r.height).as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(centre_constraints(constraint_x, r.width).as_ref())
        .split(popup_layout[1])[1]
}

pub fn inside_rect((row, column): (u16, u16), rect: Rect) -> bool {
    rect.x <= column
        && column < (rect.x + rect.width)
        && rect.y <= row
        && row < (rect.y + rect.height)
}

fn centre_constraints(constraint: Constraint, rect_bound: u16) -> [Constraint; 3] {
    match constraint {
        Constraint::Percentage(percent) => [
            Constraint::Percentage((100 - percent) / 2),
            Constraint::Percentage(percent),
            Constraint::Percentage((100 - percent) / 2),
        ],
        Constraint::Ratio(num, den) => [
            Constraint::Ratio((den - num) / 2, den),
            Constraint::Ratio(num, den),
            Constraint::Ratio((den - num) / 2, den),
        ],
        Constraint::Length(length) => [
            Constraint::Length((rect_bound - length) / 2),
            Constraint::Length(length),
            Constraint::Length((rect_bound - length) / 2),
        ],
        _ => [constraint, constraint, constraint],
    }
}

pub fn handle_key_movement(key_code: KeyCode, index: &mut usize, max_items: usize) -> EventResult {
    match key_code {
        KeyCode::Char('g') => {
            *index = 0;
            EventResult::Consumed
        }
        KeyCode::Char('G') => {
            *index = max_items - 1;
            EventResult::Consumed
        }
        KeyCode::Char('j') => {
            if max_items == 0 {
                return EventResult::Ignored;
            }
            if *index == max_items - 1 {
                *index = 0;
            } else {
                *index += 1;
            }
            EventResult::Consumed
        }
        KeyCode::Down => {
            if max_items == 0 {
                return EventResult::Ignored;
            }
            if *index == max_items - 1 {
                *index = 0;
            } else {
                *index += 1;
            }
            EventResult::Consumed
        }
        KeyCode::Char('k') => {
            if max_items == 0 {
                return EventResult::Ignored;
            }
            if *index == 0 {
                *index = max_items - 1;
            } else {
                *index -= 1;
            }
            EventResult::Consumed
        }
        KeyCode::Up => {
            if max_items == 0 {
                return EventResult::Ignored;
            }
            if *index == 0 {
                *index = max_items - 1;
            } else {
                *index -= 1;
            }
            EventResult::Consumed
        }
        _ => EventResult::Ignored,
    }
}

pub fn handle_mouse_movement(
    app: &mut App,
    area: Rect,
    mode_type: Option<Mode>,
    max_items: usize,
    index: &mut usize,
    MouseEvent { row, kind, .. }: crossterm::event::MouseEvent,
) -> EventResult {
    let offset = row - area.y;
    if let MouseEventKind::ScrollUp = kind {
        if *index != 0 {
            *index -= 1;
        }
    }

    if let MouseEventKind::ScrollDown = kind {
        if *index < max_items - 1 {
            *index += 1;
        }
    }

    if let MouseEventKind::Down(_) = kind {
        if let Some(mode) = mode_type {
            app.mode = mode;
        }
        if offset == 0 {
            return EventResult::Consumed;
        }
        if *index > area.height as usize - 2 {
            let new_index = *index - (area.height as usize - 2) + offset as usize;
            *index = new_index;
        } else {
            if offset as usize > max_items {
                *index = max_items - 1;
                return EventResult::Consumed;
            }
            *index = offset as usize - 1;
        }
    }
    EventResult::Consumed
}

pub fn generate_table<'a>(items: Vec<(Span<'a>, Spans<'a>)>, width: usize) -> Table<'a> {
    Table::new(items.into_iter().map(|(title, content)| {
        let text = wrap_text(content, width as u16);

        let height = text.height();
        let cells = vec![Cell::from(title), Cell::from(text)];
        Row::new(cells).height(height as u16).bottom_margin(1)
    }))
}

// FIX: This can be replaced when https://github.com/fdehau/tui-rs/pull/413 is merged
pub fn wrap_text(line: Spans, width: u16) -> Text {
    let mut text = Text::default();
    let mut queue = Vec::new();
    for span in &line.0 {
        let mut content = String::new();
        let style = span.style;
        for grapheme in UnicodeSegmentation::graphemes(span.content.as_ref(), true) {
            let is_newline = grapheme.chars().any(|chr| chr == '\n');
            if is_newline {
                queue
                    .into_iter()
                    .for_each(|x| add_to_current_line(&mut text, x));
                add_to_current_line(&mut text, Span::styled(content, style));
                queue = Vec::new();
                content = String::new();
                new_blank_line(&mut text);
            }

            // Insert when encountering a space.
            let is_whitespace = grapheme.chars().all(&char::is_whitespace);
            if is_whitespace {
                if current_width(&text) as u16 + content.len() as u16 != width {
                    content.push_str(grapheme);
                }
                queue
                    .into_iter()
                    .for_each(|x| add_to_current_line(&mut text, x));
                add_to_current_line(&mut text, Span::styled(content, style));
                queue = Vec::new();
                content = String::new();
                continue;
            }
            content.push_str(grapheme);

            // If the content exceeds the current length, break the content up
            if content.len() as u16 == width {
                queue
                    .into_iter()
                    .for_each(|x| add_to_current_line(&mut text, x));
                add_to_current_line(&mut text, Span::styled(content, style));
                queue = Vec::new();
                content = String::new();
                new_blank_line(&mut text);
            }

            // If the content + current width exceeds the width make a new line to break it up.
            if current_width(&text) as u16 + content.len() as u16 > width {
                new_blank_line(&mut text);
            }
        }
        if !content.is_empty() {
            queue.push(Span::styled(content, style));
        }
    }
    queue
        .into_iter()
        .for_each(|x| add_to_current_line(&mut text, x));
    if let Some(l) = text.lines.last() {
        if l.0.is_empty() {
            text.lines.pop();
        }
    }
    text
}

fn current_width(text: &Text) -> usize {
    text.lines.last().map_or(0usize, |x| {
        x.0.iter().fold(0usize, |mut acc, e| {
            acc += e.width();
            acc
        })
    })
}

fn add_to_current_line<'a>(text: &mut Text<'a>, span: Span<'a>) {
    if let Some(last) = text.lines.last_mut() {
        last.0.push(span);
    } else {
        text.lines.push(Spans::from(span));
    }
}

fn new_blank_line(text: &mut Text) {
    text.lines.push(Spans::default());
}

/// Generates the default block
pub fn generate_default_block<'a>(title: &'a str, mode: Mode, app: &App) -> Block<'a> {
    let border_colour = if app.mode == mode {
        app.theme.selected_border_colour
    } else {
        app.theme.default_border_colour
    };

    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(app.theme.border_style.border_type)
        .border_style(Style::default().fg(border_colour))
}

#[cfg(test)]
pub mod test {
    use crossterm::event::{KeyCode, KeyModifiers};

    use crate::{
        app::{App, TaskStore},
        component::layout::stack_layout::StackLayout,
        screens::main_screen::MainScreenLayer,
    };

    pub fn input_char(character: char, app: &mut App, stack_layout: &mut StackLayout) {
        app.execute_event(crossterm::event::KeyEvent::new(
            KeyCode::Char(character),
            KeyModifiers::NONE,
        ));
        execute_callbacks(app, stack_layout);
    }

    pub fn input_code(key: KeyCode, app: &mut App, stack_layout: &mut StackLayout) {
        app.execute_event(crossterm::event::KeyEvent::new(key, KeyModifiers::NONE));
        execute_callbacks(app, stack_layout);
    }

    pub fn setup(task_store: TaskStore) -> (App, StackLayout) {
        let app = App::new(crate::theme::Theme::default(), task_store);
        let stack_layout = StackLayout {
            children: vec![Box::new(MainScreenLayer::new())],
        };

        (app, stack_layout)
    }

    pub fn execute_callbacks(app: &mut App, stack_layout: &mut StackLayout) {
        while let Some(callback) = app.callbacks.pop_front() {
            callback(app, stack_layout);
        }
    }
}
