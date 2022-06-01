use tui::widgets::{Clear, Widget};

struct PopUp;

impl Widget for PopUp {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        Clear.render(area, buf);
    }
}
