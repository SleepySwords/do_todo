use tui::widgets::{Widget, Clear};

struct PopUp;

impl Widget for PopUp {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        Clear.render(area, buf);
    }
}
