use serde::{Deserialize, Serialize};
use tui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

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
            .border_style(Style::default().fg(tui::style::Color::Green))
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
