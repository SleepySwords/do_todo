use std::fmt::Display;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;
use serde::de::{Deserializer, Error};
use serde::{Deserialize, Serializer};

use crate::app::App;
use crate::error::AppError;
use crate::framework::event::PostEvent;

#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Key {
    pub fn is_pressed(&self, key_event: KeyEvent) -> bool {
        key_event.code == self.code && key_event.modifiers.contains(self.modifiers)
    }

    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Key {
        Key { code, modifiers }
    }
}

impl<'de> serde::de::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Key, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Key::from(&s).map_err(|_| D::Error::custom("Invalid key"))
    }
}

impl serde::ser::Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

const NOT_VALID: &str = "Not a valid key";

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                _ => Err(std::fmt::Error),
            })
            .collect::<Result<Vec<&str>, std::fmt::Error>>()?
            .join("-");
        f.write_str(
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

impl Key {
    fn from(value: &str) -> Result<Key, AppError> {
        let mut values = value.split('-').collect_vec().into_iter();
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
}

type Action = dyn Fn(&mut App) -> Result<PostEvent, AppError>;

pub struct KeyBinding<'a> {
    pub character: Key,
    pub short_hand: String,
    pub description: &'a str,
    pub function: Option<Box<Action>>,
}

impl KeyBinding<'_> {
    pub fn new(character: Key, description: &str) -> KeyBinding<'_> {
        KeyBinding {
            character,
            short_hand: character.to_string(),
            description,
            function: None,
        }
    }

    pub fn new_multiple(character: [Key; 2], description: &str) -> KeyBinding<'_> {
        KeyBinding {
            character: character[0],
            short_hand: itertools::intersperse(
                character.iter().map(|f| f.to_string()),
                " ".to_string(),
            )
            .collect::<String>(),
            description,
            function: None,
        }
    }
    pub fn register_key<T: 'static>(
        character: Key,
        description: &str,
        function: T,
    ) -> KeyBinding<'_>
    where
        T: Fn(&mut App) -> Result<PostEvent, AppError>,
    {
        KeyBinding {
            character,
            short_hand: character.to_string(),
            description,
            function: Some(Box::new(function)),
        }
    }
}
