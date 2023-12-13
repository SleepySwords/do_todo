use crossterm::event::{KeyCode, KeyModifiers};

use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

use crate::key::Key;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Theme {
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

    pub complete_key: Key,
    pub flip_progress_key: Key,
    pub edit_key: Key,
    pub delete_key: Key,

    #[serde(with = "border_parser")]
    pub border_style: BorderType,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            default_border_colour: Color::White,
            selected_border_colour: Color::Green,
            selected_task_colour: Color::LightBlue,
            high_priority_colour: Color::Red,
            normal_priority_colour: Color::LightYellow,
            low_priority_colour: Color::Rgb(0, 0, 0),
            use_fuzzy: true,
            up_keys: [
                Key {
                    code: KeyCode::Char('k'),
                    modifiers: KeyModifiers::NONE,
                },
                Key {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                },
            ],
            down_keys: [
                Key {
                    code: KeyCode::Char('j'),
                    modifiers: KeyModifiers::NONE,
                },
                Key {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                },
            ],
            move_up_fuzzy: Key {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::CONTROL,
            },
            move_down_fuzzy: Key {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
            },
            move_top: Key {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::NONE,
            },
            move_bottom: Key {
                code: KeyCode::Char('G'),
                modifiers: KeyModifiers::NONE,
            },
            complete_key: Key {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::NONE,
            },
            flip_progress_key: Key {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
            },
            edit_key: Key {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::NONE,
            },
            delete_key: Key {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
            },
            border_style: BorderType::Plain,
        }
    }
}

impl Theme {
    pub fn highlight_dropdown_style(&self) -> Style {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(tui::style::Color::LightMagenta)
    }

    pub fn styled_block<'a>(&self, title: &'a str, border_color: Color) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(self.border_style)
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
