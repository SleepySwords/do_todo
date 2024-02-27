use crossterm::event::{KeyEvent, MouseEvent};

use crate::{
    error::AppError,
    framework::{event::PostEvent, screen_manager::ScreenManager},
};

pub mod dialog;
pub mod fuzzy;
pub mod input_box;
pub mod vim;

pub struct Overlay;

impl Overlay {
    pub fn key_event(
        screen_manager: &mut ScreenManager,
        key_event: KeyEvent,
    ) -> Result<PostEvent, AppError> {
        if let Some(overlay) = screen_manager.overlays.last_mut() {
            return Ok(overlay.key_event(&mut screen_manager.app, key_event));
        }
        Ok(PostEvent::noop(true))
    }

    pub fn mouse_event(screen_manager: &mut ScreenManager, mouse_event: MouseEvent) -> PostEvent {
        if let Some(overlay) = screen_manager.overlays.last_mut() {
            return overlay.mouse_event(&mut screen_manager.app, mouse_event);
        }
        PostEvent::noop(true)
    }
}
