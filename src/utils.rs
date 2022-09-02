use crossterm::event::KeyCode;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Text},
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

pub fn generate_table<'a>(items: Vec<(Span<'a>, &str, Style)>, width: usize) -> Table<'a> {
    Table::new(items.iter().map(|item| {
        let text = textwrap::fill(item.1, width);
        let height = text.chars().filter(|c| *c == '\n').count() + 1;
        // Clone (actually crying tho)
        let cells = vec![
            Cell::from(item.0.to_owned()),
            Cell::from(Text::styled(text, item.2)),
        ];
        Row::new(cells).height(height as u16).bottom_margin(1)
    }))
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
