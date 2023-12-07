use serde::{Deserialize, Serialize};
use tui::{style::Color, widgets::BorderType};

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
            use_fuzzy: false,
            border_style: BorderStyle::default(),
        }
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
