use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;
use serde::de::{Deserializer, Error};
use serde::{Deserialize, Serialize, Serializer};
use tui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

use crate::error::AppError;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Theme {
    pub default_border_colour: Color,
    pub selected_border_colour: Color,
    pub selected_task_colour: Color,
    pub high_priority_colour: Color,
    pub normal_priority_colour: Color,
    pub low_priority_colour: Color,

    pub use_fuzzy: bool,
    pub up_keys: [Key; 2],
    pub down_keys: [Key; 2],
    pub move_up_fuzzy: Key,
    pub move_down_fuzzy: Key,
    pub move_top: Key,
    pub move_bottom: Key,

    #[serde(skip_serializing, skip_deserializing)]
    pub border_style: BorderStyle,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            default_border_colour: Color::White,
            selected_border_colour: Color::Green,
            selected_task_colour: Color::LightBlue,
            high_priority_colour: Color::Red,
            normal_priority_colour: Color::LightYellow,
            low_priority_colour: Color::Green,
            use_fuzzy: true,
            border_style: BorderStyle::default(),
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
            .border_type(self.border_style.border_type)
            .title(title)
            .border_style(Style::default().fg(border_color))
    }
}

pub struct BorderStyle {
    pub border_type: BorderType,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle {
            border_type: BorderType::Plain,
        }
    }
}

#[derive(Debug)]
pub struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl Key {
    pub fn is_pressed(&self, key_event: KeyEvent) -> bool {
        return key_event.code == self.code && key_event.modifiers.contains(self.modifiers);
    }
}

impl<'de> serde::de::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Key, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        from(&s).map_err(|_| D::Error::custom("Invalid key"))
    }
}

impl<'de> serde::ser::Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let modifiers = self
            .modifiers
            .iter()
            .map(|f| match f {
                KeyModifiers::CONTROL => Ok("ctrl"),
                KeyModifiers::SHIFT => Ok("shift"),
                KeyModifiers::ALT => Ok("alt"),
                KeyModifiers::SUPER => Ok("super"),
                KeyModifiers::HYPER => Ok("hyper"),
                KeyModifiers::META => Ok("meta"),
                _ => Err(serde::ser::Error::custom(NOT_VALID)),
            })
            .collect::<Result<Vec<&str>, S::Error>>()?
            .join("-");
        serializer.serialize_str(
            &(if modifiers.is_empty() {
                String::from("")
            } else {
                modifiers + "-"
            } + &match self.code {
                KeyCode::Backspace => "backspace".to_string(),
                KeyCode::Enter => "enter".to_string(),
                KeyCode::Left => "left".to_string(),
                KeyCode::Right => "right".to_string(),
                KeyCode::Up => "up".to_string(),
                KeyCode::Down => "down".to_string(),
                KeyCode::Home => "home".to_string(),
                KeyCode::End => "end".to_string(),
                KeyCode::PageUp => "pageup".to_string(),
                KeyCode::PageDown => "pagedown".to_string(),
                KeyCode::Tab => "tab".to_string(),
                KeyCode::BackTab => "backtab".to_string(),
                KeyCode::Delete => "delete".to_string(),
                KeyCode::Insert => "insert".to_string(),
                KeyCode::F(num) => format!("f{}", num),
                KeyCode::Char(' ') => "space".to_string(),
                KeyCode::Char(c) => c.to_string(),
                KeyCode::Null => "null".to_string(),
                KeyCode::Esc => "esc".to_string(),
                KeyCode::CapsLock => "capslock".to_string(),
                KeyCode::ScrollLock => "scrolllock".to_string(),
                KeyCode::NumLock => "numlock".to_string(),
                KeyCode::PrintScreen => "printscreen".to_string(),
                KeyCode::Pause => "pause".to_string(),
                KeyCode::Menu => "menu".to_string(),
                KeyCode::KeypadBegin => "keypadbegin".to_string(),
                _ => "Unknown".to_string(),
            }),
        )
    }
}

const NOT_VALID: &str = "Not a valid key";

fn from(value: &str) -> Result<Key, AppError> {
    let mut values = value.split("-").collect_vec().into_iter();
    let code = match values
        .next_back()
        .ok_or_else(|| AppError::InvalidKey("Empty key".to_string()))?
    {
        "backspace" => KeyCode::Backspace,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "null" => KeyCode::Null,
        "esc" => KeyCode::Esc,
        "capslock" => KeyCode::CapsLock,
        "scrolllock" => KeyCode::ScrollLock,
        "numlock" => KeyCode::NumLock,
        "printscreen" => KeyCode::PrintScreen,
        "pause" => KeyCode::Pause,
        "menu" => KeyCode::Menu,
        "keypadbegin" => KeyCode::KeypadBegin,
        "space" => KeyCode::Char(' '),
        a if a.starts_with('f') && a.len() > 1 => KeyCode::F(
            a.strip_prefix('f')
                .ok_or_else(|| AppError::InvalidKey(NOT_VALID.to_string()))?
                .parse::<u8>()?,
        ),
        a => KeyCode::Char(
            a.chars()
                .next()
                .ok_or_else(|| AppError::InvalidKey(NOT_VALID.to_string()))?,
        ),
    };
    let modifiers = values
        .map(|f| match f.to_lowercase().as_str() {
            "shift" => Ok(KeyModifiers::SHIFT),
            "control" | "ctrl" => Ok(KeyModifiers::CONTROL),
            "alt" => Ok(KeyModifiers::ALT),
            "super" => Ok(KeyModifiers::SUPER),
            "hyper" => Ok(KeyModifiers::HYPER),
            "meta" => Ok(KeyModifiers::META),
            _ => Err(AppError::InvalidKey(NOT_VALID.to_string())),
        })
        .fold_ok(KeyModifiers::NONE, |f, acc| f.union(acc))?;
    Ok(Key { code, modifiers })
}
