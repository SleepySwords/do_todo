use tui::style::Color;

pub struct Theme {
    pub default_colour: Color,
    pub selected_colour: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            default_colour: Color::Rgb(255, 192, 203),
            selected_colour: Color::Yellow,
        }
    }
}
