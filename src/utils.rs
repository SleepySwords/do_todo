use crossterm::event::KeyCode;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::app::{App, SelectedComponent};

// Only available for percentages, ratios and length
pub fn centered_rect(constraint_x: Constraint, constraint_y: Constraint, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(generate_constraints(constraint_y, r.height).as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(generate_constraints(constraint_x, r.width).as_ref())
        .split(popup_layout[1])[1]
}

fn generate_constraints(constraint: Constraint, rect_bound: u16) -> [Constraint; 3] {
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

// Should return if consumed input
pub fn handle_movement(key_code: KeyCode, index: &mut usize, max_items: usize) {
    match key_code {
        KeyCode::Char('g') => {
            *index = 0;
        }
        KeyCode::Char('G') => {
            *index = max_items - 1;
        }
        KeyCode::Char('j') => {
            if max_items == 0 {
                return;
            }
            if *index == max_items - 1 {
                *index = 0;
            } else {
                *index += 1;
            }
        }
        KeyCode::Down => {
            if max_items == 0 {
                return;
            }
            if *index == max_items - 1 {
                *index = 0;
            } else {
                *index += 1;
            }
        }
        KeyCode::Char('k') => {
            if max_items == 0 {
                return;
            }
            if *index == 0 {
                *index = max_items - 1;
            } else {
                *index -= 1;
            }
        }
        KeyCode::Up => {
            if max_items == 0 {
                return;
            }
            if *index == 0 {
                *index = max_items - 1;
            } else {
                *index -= 1;
            }
        }
        _ => {}
    }
}

pub fn generate_table<'a>(items: Vec<(Span<'a>, Spans<'a>)>, width: usize) -> Table<'a> {
    Table::new(items.into_iter().map(|(title, content)| {
        // FIX: Spans are broken up even if they don't have a space
        // FIX: This is because we would split based on spans not spaces.
        // This can be replaced when https://github.com/fdehau/tui-rs/pull/413 is merged
        // HACK: Factorise this
        let acc = content.0.into_iter().fold((0, Text::raw("")), |acc, span| {
            let mut current_width = acc.0;
            let mut text = acc.1;
            if current_width + span.width() < width {
                current_width = (current_width + span.width()) % width;
                add_to_text(&mut text, span);
                (current_width, text)
            } else {
                let mut iter = span.content.split(' ').peekable();
                while let Some(str_content) = iter.next() {
                    let next_element = iter.peek().is_some();
                    if str_content.len() + current_width + if next_element { 1 } else { 0 } < width
                    {
                        // To string?!?
                        let mut stx = str_content.to_string();
                        if next_element {
                            stx.push(' ');
                        }
                        current_width = (current_width + stx.len()) % width;
                        add_to_text(&mut text, Span::styled(stx, span.style));
                    } else {
                        let mut stx = str_content.to_string();
                        if next_element {
                            stx.push(' ');
                        }
                        current_width = (current_width + stx.len()) % width;
                        text.lines.push(Spans::from(Span::styled(stx, span.style)));
                    }
                }
                (current_width, text)
            }
        });

        let height = acc.1.height();
        let cells = vec![Cell::from(title), Cell::from(acc.1)];
        Row::new(cells).height(height as u16).bottom_margin(1)
    }))
}

pub fn add_to_text<'a>(text: &mut Text<'a>, span: Span<'a>) {
    if let Some(last) = text.lines.last_mut() {
        last.0.push(span);
    } else {
        text.lines.push(Spans::from(span));
    }
}

pub fn generate_block<'a>(
    title: &'a str,
    selected_component: SelectedComponent,
    app: &App,
) -> Block<'a> {
    let border_colour = if app.selected_component == selected_component {
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
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::{app::App, input};

    pub fn input_char(character: char, app: &mut App) {
        input::handle_key(
            KeyEvent {
                code: KeyCode::Char(character),
                modifiers: KeyModifiers::NONE,
            },
            app,
        )
    }
}
