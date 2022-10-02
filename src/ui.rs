use crate::{app::App, screens::main_screen::MainScreenLayer};
use tui::{backend::Backend, Frame};

pub fn render_ui<B: Backend>(app: &mut App, f: &mut Frame<B>) {
    MainScreenLayer::draw(app, f);
}
