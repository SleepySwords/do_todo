pub mod main_screen;

// use tui::{backend::Backend, Frame};

// use crate::app::App;

// use self::main_screen::MainScreenLayer;

// pub enum ScreenLayer {
//     MainScreen,
//     Empty,
// }

// impl ScreenLayer {
//     pub fn draw<B>(&self, app: &mut App, f: &mut Frame<B>)
//     where
//         B: Backend,
//     {
//         match self {
//             ScreenLayer::MainScreen => MainScreenLayer::draw(app, f),
//             ScreenLayer::Empty => {}
//         }
//     }

//     pub fn handle_event<B>(self, app: &mut App, f: &mut Frame<B>)
//     where
//         B: Backend,
//     {
//     }
// }
