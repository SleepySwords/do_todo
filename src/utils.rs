use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use tui::layout::{Constraint, Direction, Layout, Rect};

use std::usize;

use crate::app::{App, Mode};
use crate::config::{Config, KeyBindings};
use crate::draw::{Action, PostAction};

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
        Constraint::Length(length) => {
            let var = match rect_bound.checked_sub(length) {
                Some(var) => var / 2,
                _ => 0,
            };
            [
                Constraint::Length(var),
                Constraint::Length(length),
                Constraint::Length(var),
            ]
        }
        _ => [constraint, constraint, constraint],
    }
}

pub fn handle_key_movement(
    theme: &Config,
    key_event: KeyEvent,
    index: &mut usize,
    max_items: usize,
) -> PostAction {
    match KeyBindings::from_event(theme, key_event) {
        KeyBindings::MoveTop => {
            *index = 0;
            PostAction {
                propegate_further: false,
                action: Action::Noop,
            }
        }
        KeyBindings::MoveBottom => {
            *index = max_items - 1;
            PostAction {
                propegate_further: false,
                action: Action::Noop,
            }
        }
        KeyBindings::DownKeys => {
            if max_items == 0 {
                return PostAction {
                    propegate_further: true,
                    action: Action::Noop,
                };
            }
            *index = (*index + 1).rem_euclid(max_items);
            PostAction {
                propegate_further: false,
                action: Action::Noop,
            }
        }
        KeyBindings::UpKeys => {
            if max_items == 0 {
                return PostAction {
                    propegate_further: true,
                    action: Action::Noop,
                };
            }
            match index.checked_sub(1) {
                Some(val) => *index = val,
                None => *index = max_items - 1,
            }
            PostAction {
                propegate_further: false,
                action: Action::Noop,
            }
        }
        _ => PostAction {
            propegate_further: true,
            action: Action::Noop,
        },
    }
}

pub fn handle_mouse_movement(
    app: &mut App,
    area: Rect,
    mode: Mode,
    max_items: usize,
    MouseEvent { row, kind, .. }: crossterm::event::MouseEvent,
) -> PostAction {
    let Some(index) = app.selected_index(mode) else {
        return PostAction {
            propegate_further: false,
            action: Action::Noop,
        };
    };

    if max_items == 0 {
        return PostAction {
            propegate_further: false,
            action: Action::Noop,
        };
    }
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
        app.mode = mode;
        if offset == 0 {
            return PostAction {
                propegate_further: false,
                action: Action::Noop,
            };
        }
        if let Some(index) = app.selected_index(mode) {
            if *index > area.height as usize - 2 {
                let new_index = *index - (area.height as usize - 2) + offset as usize;
                *index = new_index;
            } else {
                if offset as usize > max_items {
                    *index = max_items - 1;
                    return PostAction {
                        propegate_further: false,
                        action: Action::Noop,
                    };
                }
                *index = offset as usize - 1;
            }
        }
    }
    PostAction {
        propegate_further: false,
        action: Action::Noop,
    }
}

pub(crate) mod ui {
    use tui::{
        prelude::Constraint,
        style::Style,
        text::{Line, Span},
        widgets::{Block, Borders, Cell, Row, Table},
    };

    use crate::app::{App, Mode};

    use super::wrap;

    pub fn generate_table<'a>(items: Vec<(Span<'a>, Line<'a>)>, width: usize) -> Table<'a> {
        Table::new(
            items.into_iter().map(|(title, content)| {
                let text = wrap::wrap_text(content, width as u16);

                let height = text.height();
                let cells = vec![Cell::from(title), Cell::from(text)];
                Row::new(cells).height(height as u16).bottom_margin(1)
            }),
            [Constraint::Percentage(20), Constraint::Length(width as u16)],
        )
    }

    /// Generates the default block
    pub fn generate_default_block<'a>(app: &App, title: &'a str, mode: Mode) -> Block<'a> {
        let border_colour = if app.mode == mode {
            app.config.selected_border_colour
        } else {
            app.config.default_border_colour
        };

        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(app.config.border_type)
            .border_style(Style::default().fg(border_colour))
    }
}

mod wrap {
    use tui::text::{Line, Span, Text};
    use unicode_segmentation::UnicodeSegmentation;

    // FIX: This can be replaced when https://github.com/ratatui-org/ratatui/issues/293 is merged
    pub fn wrap_text(line: Line, width: u16) -> Text {
        let mut text = Text::default();
        let mut queue = Vec::new();
        for span in &line.spans {
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
            if l.spans.is_empty() {
                text.lines.pop();
            }
        }
        text
    }

    fn current_width(text: &Text) -> usize {
        text.lines.last().map_or(0usize, |x| {
            x.spans.iter().fold(0usize, |mut acc, e| {
                acc += e.width();
                acc
            })
        })
    }

    fn add_to_current_line<'a>(text: &mut Text<'a>, span: Span<'a>) {
        if let Some(last) = text.lines.last_mut() {
            last.spans.push(span);
        } else {
            text.lines.push(Line::from(span));
        }
    }

    fn new_blank_line(text: &mut Text) {
        text.lines.push(Line::default());
    }
}

#[cfg(test)]
pub mod test {
    use crossterm::event::{KeyCode, KeyModifiers};

    use crate::{app::App, input, task::TaskStore};

    pub fn input_char(character: char, app: &mut App) {
        let _result = input::key_event(
            app,
            crossterm::event::KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
        );
    }

    pub fn input_code(key: KeyCode, app: &mut App) {
        let _result = input::key_event(
            app,
            crossterm::event::KeyEvent::new(key, KeyModifiers::NONE),
        );
    }

    pub fn setup(task_store: TaskStore) -> App {
        App::new(crate::config::Config::default(), task_store)
    }
}
