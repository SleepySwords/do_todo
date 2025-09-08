use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Color;

use crate::app::{App, Mode};
use crate::config::Config;
use crate::error::AppError;
use crate::framework::event::PostEvent;

pub const IS_DEBUG: bool = cfg!(debug_assertions);

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
) -> PostEvent {
    if theme.move_top.is_pressed(key_event) {
        *index = 0;
        return PostEvent::noop(false);
    }
    if theme.move_bottom.is_pressed(key_event) {
        *index = max_items - 1;
        return PostEvent::noop(false);
    }
    if theme.down_keys.iter().any(|f| f.is_pressed(key_event)) {
        if max_items == 0 {
            return PostEvent::noop(true);
        }
        *index = (*index + 1).rem_euclid(max_items);
        return PostEvent::noop(false);
    }
    if theme.up_keys.iter().any(|f| f.is_pressed(key_event)) {
        if max_items == 0 {
            return PostEvent::noop(true);
        }
        match index.checked_sub(1) {
            Some(val) => *index = val,
            None => *index = max_items - 1,
        }
        return PostEvent::noop(false);
    }
    PostEvent::noop(true)
}

pub fn handle_mouse_movement_app(
    app: &mut App,
    area: Rect,
    mode: Mode,
    max_items: usize,
    MouseEvent { row, kind, .. }: crossterm::event::MouseEvent,
) -> PostEvent {
    if let Some(index) = app.selected_index(mode) {
        if max_items == 0 {
            return PostEvent::noop(false);
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
            if let Some(index) = app.selected_index(mode) {
                if offset == 0 {
                    return PostEvent::noop(false);
                }
                // FIXME: probably should use Block::inner for these.
                if *index > area.height as usize - 2 {
                    let new_index = *index - (area.height as usize - 2) + offset as usize;
                    *index = new_index;
                } else {
                    if offset as usize > max_items {
                        *index = max_items - 1;
                        return PostEvent::noop(false);
                    }
                    *index = offset as usize - 1;
                }
            }
        }
    }
    PostEvent::noop(false)
}

pub fn handle_mouse_movement(
    index: &mut usize,
    app_mode: &mut Mode,
    area: Rect,
    mode: Mode,
    max_items: usize,
    MouseEvent { row, kind, .. }: crossterm::event::MouseEvent,
) -> PostEvent {
    if max_items == 0 {
        return PostEvent::noop(false);
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
        *app_mode = mode;
        if offset == 0 {
            return PostEvent::noop(false);
        }
        if *index > area.height as usize - 2 {
            let new_index = *index - (area.height as usize - 2) + offset as usize;
            *index = new_index;
        } else {
            if offset as usize > max_items {
                *index = max_items - 1;
                return PostEvent::noop(false);
            }
            *index = offset as usize - 1;
        }
    }
    PostEvent::noop(false)
}

pub fn str_to_colour(colour: &str) -> Result<Color, AppError> {
    if colour.starts_with('#') {
        let red = u8::from_str_radix(&colour[1..3], 16)?;
        let green = u8::from_str_radix(&colour[3..5], 16)?;
        let blue = u8::from_str_radix(&colour[5..7], 16)?;
        Ok(Color::Rgb(red, green, blue))
    } else if let Ok(colour) = colour.parse() {
        Ok(Color::Indexed(colour))
    } else {
        match colour
            .to_lowercase()
            .replace([' ', '_', '-'], "")
            .as_str()
            .parse::<Color>()
        {
            Ok(colour) => Ok(colour),
            Err(_) => Err(AppError::InvalidColour),
        }
    }
}

pub mod task_position {
    use crate::data::data_store::{DataTaskStore, TaskID, TaskIDRef};

    fn find_task_id<T: DataTaskStore>(
        store: &T,
        pos: &mut usize,
        task_id: TaskIDRef,
    ) -> Option<TaskID> {
        if *pos == 0 {
            return Some(task_id.to_string());
        }
        *pos -= 1;

        let task = store.task(task_id)?;
        if !task.opened {
            return None;
        }

        store
            .subtasks(task_id)?
            .iter()
            .find_map(|subtask_id| find_task_id(store, pos, subtask_id))
    }

    pub fn cursor_to_task<T: DataTaskStore>(store: &T, mut pos: usize) -> Option<TaskID> {
        store
            .root_tasks()
            .iter()
            .find_map(|root_task_id| find_task_id(store, &mut pos, root_task_id))
    }

    pub fn cursor_to_completed_task<T: DataTaskStore>(store: &T, mut pos: usize) -> Option<TaskID> {
        store
            .completed_root_tasks()
            .iter()
            .find_map(|root_task_id| find_task_id(store, &mut pos, root_task_id))
    }

    fn find_cursor_position<T: DataTaskStore>(
        store: &T,
        current_index: &mut usize,
        to_find: TaskIDRef,
        curr: TaskIDRef,
    ) -> Option<()> {
        if to_find == curr {
            return Some(());
        }
        *current_index += 1;
        let t = store.task(curr)?;
        if !t.opened {
            return None;
        }
        if let Some(subtasks) = store.subtasks(curr) {
            for task in subtasks {
                if task == to_find {
                    return Some(());
                }
                if let Some(()) = find_cursor_position(store, current_index, to_find, task) {
                    return Some(());
                }
            }
        }
        None
    }

    pub fn task_to_cursor<T: DataTaskStore>(store: &T, id: TaskIDRef) -> Option<usize> {
        let mut current_index = 0;
        for curr in store.root_tasks() {
            if let Some(()) = find_cursor_position(store, &mut current_index, id, curr) {
                return Some(current_index);
            }
        }
        None
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

pub mod wrap {
    use tui::text::{Line, Span, Text};
    use unicode_segmentation::UnicodeSegmentation;

    // FIX: This can be replaced when https://github.com/ratatui-org/ratatui/issues/293 is merged
    pub fn wrap_text<'a, T: Into<Line<'a>>>(line: T, width: u16) -> Text<'a> {
        let mut text = Text::default();
        let mut queue = Vec::new();
        for span in &line.into().spans {
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

    use crate::data::data_store::{DataTaskStore, DataTaskStoreKind};
    use crate::data::json_data_store::JsonDataStore;
    use crate::framework::screen_manager::ScreenManager;
    use crate::task::Task;
    use crate::{app::App, input};

    use super::task_position::cursor_to_task;

    pub fn input_char(character: char, screen_manager: &mut ScreenManager) {
        let result = input::key_event(
            screen_manager,
            crossterm::event::KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
        );
        if let Ok(post_event) = result {
            screen_manager.handle_post_event(post_event);
        }
    }

    pub fn input_code(key: KeyCode, screen_manager: &mut ScreenManager) {
        let result = input::key_event(
            screen_manager,
            crossterm::event::KeyEvent::new(key, KeyModifiers::NONE),
        );
        if let Ok(post_event) = result {
            screen_manager.handle_post_event(post_event);
        }
    }

    pub fn setup(task_store: JsonDataStore) -> ScreenManager {
        ScreenManager {
            overlays: vec![],
            app: App::new(
                crate::config::Config::default(),
                crate::data::data_store::DataTaskStoreKind::Json(task_store),
            ),
        }
    }

    pub fn get_task_from_pos(task_store: &DataTaskStoreKind, pos: usize) -> &Task {
        task_store
            .task(&cursor_to_task(task_store, pos).unwrap())
            .unwrap()
    }
}
