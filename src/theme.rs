use serde::{Deserialize, Serialize};
use tui::style::Color;

#[derive(Deserialize, Serialize)]
pub struct Theme {
    pub default_border_colour: Color,
    pub selected_border_colour: Color,
    pub selected_task_colour: Color,
    pub high_priority_colour: Color,
    pub normal_priority_colour: Color,
    pub low_priority_colour: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            default_border_colour: Color::Rgb(255, 192, 203),
            selected_border_colour: Color::Green,
            selected_task_colour: Color::Magenta,
            high_priority_colour: Color::Red,
            normal_priority_colour: Color::White,
            low_priority_colour: Color::Green
        }
    }
}
