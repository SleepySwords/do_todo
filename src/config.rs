use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

use crate::key::Key;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    #[serde(with = "color_parser")]
    pub default_border_colour: Color,
    #[serde(with = "color_parser")]
    pub selected_border_colour: Color,
    #[serde(with = "color_parser")]
    pub selected_task_colour: Color,
    #[serde(with = "color_parser")]
    pub high_priority_colour: Color,
    #[serde(with = "color_parser")]
    pub normal_priority_colour: Color,
    #[serde(with = "color_parser")]
    pub low_priority_colour: Color,

    pub use_fuzzy: bool,
    pub up_keys: [Key; 2],
    pub down_keys: [Key; 2],
    pub move_up_fuzzy: Key,
    pub move_down_fuzzy: Key,
    pub move_top: Key,
    pub move_bottom: Key,
    pub move_task_up: Key,
    pub move_task_down: Key,

    pub complete_key: Key,
    pub edit_key: Key,
    pub delete_key: Key,
    pub add_key: Key,
    pub flip_progress_key: Key,
    pub change_priority_key: Key,
    pub restore_key: Key,

    pub tasks_menu_key: Key,
    pub completed_tasks_menu_key: Key,
    pub open_help_key: Key,
    pub quit_key: Key,

    pub sort_key: Key,
    pub enable_autosort_key: Key,
    pub tag_menu: Key,

    pub flip_subtask_key: Key,
    pub move_subtask_level_up: Key,
    pub move_subtask_level_down: Key,

    #[serde(with = "border_parser")]
    pub border_type: BorderType,

    pub selected_cursor: String,
    pub nested_padding: String,
    pub closed_subtask: String,
    pub open_subtask: String,

    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_border_colour: Color::White,
            selected_border_colour: Color::Green,
            selected_task_colour: Color::LightBlue,
            high_priority_colour: Color::Red,
            normal_priority_colour: Color::LightYellow,
            low_priority_colour: Color::Green,
            use_fuzzy: true,
            up_keys: [
                Key::new(KeyCode::Char('k'), KeyModifiers::NONE),
                Key::new(KeyCode::Up, KeyModifiers::NONE),
            ],
            down_keys: [
                Key::new(KeyCode::Char('j'), KeyModifiers::NONE),
                Key::new(KeyCode::Down, KeyModifiers::NONE),
            ],
            move_task_up: Key::new(KeyCode::Char('K'), KeyModifiers::NONE),
            move_task_down: Key::new(KeyCode::Char('J'), KeyModifiers::NONE),
            move_up_fuzzy: Key::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
            move_down_fuzzy: Key::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
            move_top: Key::new(KeyCode::Char('g'), KeyModifiers::NONE),
            move_bottom: Key::new(KeyCode::Char('G'), KeyModifiers::NONE),

            complete_key: Key::new(KeyCode::Char('c'), KeyModifiers::NONE),
            flip_progress_key: Key::new(KeyCode::Char(' '), KeyModifiers::NONE),
            edit_key: Key::new(KeyCode::Char('e'), KeyModifiers::NONE),
            delete_key: Key::new(KeyCode::Char('d'), KeyModifiers::NONE),
            add_key: Key::new(KeyCode::Char('a'), KeyModifiers::NONE),
            change_priority_key: Key::new(KeyCode::Char('p'), KeyModifiers::NONE),
            restore_key: Key::new(KeyCode::Char('r'), KeyModifiers::NONE),

            tasks_menu_key: Key::new(KeyCode::Char('1'), KeyModifiers::NONE),
            completed_tasks_menu_key: Key::new(KeyCode::Char('2'), KeyModifiers::NONE),
            tag_menu: Key::new(KeyCode::Char('t'), KeyModifiers::NONE),
            open_help_key: Key::new(KeyCode::Char('x'), KeyModifiers::NONE),
            quit_key: Key::new(KeyCode::Char('q'), KeyModifiers::NONE),

            enable_autosort_key: Key::new(KeyCode::Char('S'), KeyModifiers::NONE),
            sort_key: Key::new(KeyCode::Char('s'), KeyModifiers::NONE),

            flip_subtask_key: Key::new(KeyCode::Enter, KeyModifiers::NONE),
            move_subtask_level_up: Key::new(KeyCode::Char('L'), KeyModifiers::NONE),
            move_subtask_level_down: Key::new(KeyCode::Char('H'), KeyModifiers::NONE),

            border_type: BorderType::Plain,
            selected_cursor: String::from(" > "),
            nested_padding: String::from(" │  "),
            closed_subtask: String::from(" >  "),
            open_subtask: String::from(" v  "),
            debug: false,
        }
    }
}

impl Config {
    pub fn highlight_dropdown_style(&self) -> Style {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(self.selected_task_colour)
    }

    pub fn styled_block<'a>(&self, title: &'a str, border_color: Color) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(self.border_type)
            .title(title)
            .border_style(Style::default().fg(border_color))
    }
}

mod color_parser {
    use serde::{Deserialize, Deserializer, Serializer};
    use tui::style::Color;

    pub fn serialize<S>(colour: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&colour.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Color>().map_err(serde::de::Error::custom)
    }
}

mod border_parser {
    use serde::{Deserialize, Deserializer, Serializer};
    use tui::widgets::BorderType;

    pub fn serialize<S>(border: &BorderType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&border.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BorderType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "plain" => Ok(BorderType::Plain),
            "rounded" => Ok(BorderType::Rounded),
            "double" => Ok(BorderType::Double),
            "thick" => Ok(BorderType::Thick),
            "quadrantinside" => Ok(BorderType::QuadrantInside),
            "quadrantoutside" => Ok(BorderType::QuadrantOutside),
            _ => Err(serde::de::Error::custom("Test")),
        }
    }
}

pub enum KeyBindings {
    UpKeys,
    DownKeys,
    MoveTaskUp,
    MoveTaskDown,
    MoveUpFuzzy,
    MoveDownFuzzy,
    MoveTop,
    MoveBottom,

    CompleteKey,
    FlipProgressKey,
    EditKey,
    DeleteKey,
    AddKey,
    ChangePriorityKey,
    RestoreKey,

    TasksMenuKey,
    CompletedTasksMenuKey,
    TagMenu,
    OpenHelpKey,
    QuitKey,

    EnableAutosortKey,
    SortKey,

    OpenSubtasksKey,
    MoveSubtaskLevelUp,
    MoveSubtaskLevelDown,

    None,
}

impl KeyBindings {
    pub fn from_event(config: &Config, event: KeyEvent) -> KeyBindings {
        if config.up_keys.iter().any(|f| f.is_pressed(event)) {
            return KeyBindings::UpKeys;
        }
        if config.down_keys.iter().any(|f| f.is_pressed(event)) {
            return KeyBindings::DownKeys;
        }
        if config.move_task_up.is_pressed(event) {
            return KeyBindings::MoveTaskUp;
        }
        if config.move_task_down.is_pressed(event) {
            return KeyBindings::MoveTaskDown;
        }
        if config.move_up_fuzzy.is_pressed(event) {
            return KeyBindings::MoveUpFuzzy;
        }
        if config.move_down_fuzzy.is_pressed(event) {
            return KeyBindings::MoveDownFuzzy;
        }
        if config.move_top.is_pressed(event) {
            return KeyBindings::MoveTop;
        }
        if config.move_bottom.is_pressed(event) {
            return KeyBindings::MoveBottom;
        }

        if config.complete_key.is_pressed(event) {
            return KeyBindings::CompleteKey;
        }
        if config.flip_progress_key.is_pressed(event) {
            return KeyBindings::FlipProgressKey;
        }
        if config.edit_key.is_pressed(event) {
            return KeyBindings::EditKey;
        }
        if config.delete_key.is_pressed(event) {
            return KeyBindings::DeleteKey;
        }
        if config.add_key.is_pressed(event) {
            return KeyBindings::AddKey;
        }
        if config.change_priority_key.is_pressed(event) {
            return KeyBindings::ChangePriorityKey;
        }
        if config.restore_key.is_pressed(event) {
            return KeyBindings::RestoreKey;
        }

        if config.tasks_menu_key.is_pressed(event) {
            return KeyBindings::TasksMenuKey;
        }
        if config.completed_tasks_menu_key.is_pressed(event) {
            return KeyBindings::CompletedTasksMenuKey;
        }
        if config.tag_menu.is_pressed(event) {
            return KeyBindings::TagMenu;
        }
        if config.open_help_key.is_pressed(event) {
            return KeyBindings::OpenHelpKey;
        }
        if config.quit_key.is_pressed(event) {
            return KeyBindings::QuitKey;
        }

        if config.enable_autosort_key.is_pressed(event) {
            return KeyBindings::EnableAutosortKey;
        }
        if config.sort_key.is_pressed(event) {
            return KeyBindings::SortKey;
        }
        if config.flip_subtask_key.is_pressed(event) {
            return KeyBindings::OpenSubtasksKey;
        }
        if config.move_subtask_level_up.is_pressed(event) {
            return KeyBindings::MoveSubtaskLevelUp;
        }
        if config.move_subtask_level_down.is_pressed(event) {
            return KeyBindings::MoveSubtaskLevelDown;
        }

        KeyBindings::None
    }
}
