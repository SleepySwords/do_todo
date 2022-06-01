use serde::{Deserialize, Serialize};
use tui::style::{Color, Modifier, Style};

#[derive(Deserialize, Serialize)]
pub struct Theme {
    pub default_border_colour: Color,
    pub selected_border_colour: Color,
    pub selected_task_colour: Color,
    pub high_priority_colour: Color,
    pub normal_priority_colour: Color,
    pub low_priority_colour: Color,
    pub test_thing: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            default_border_colour: Color::Rgb(255, 192, 203),
            selected_border_colour: Color::Green,
            selected_task_colour: Color::Red,
            high_priority_colour: Color::Red,
            normal_priority_colour: Color::LightYellow,
            low_priority_colour: Color::Green,
            test_thing: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        }
    }
}
