use crate::{app::App, component::overlay::Overlay};

use super::event::{Action, PostEvent};

// Above should be data, this should be map.
pub struct ScreenManager {
    pub app: App,
    pub overlays: Vec<Overlay<'static>>,
}

impl ScreenManager {
    pub fn push_layer(&mut self, component: Overlay<'static>) {
        self.overlays.push(component);
    }

    pub fn pop_layer(&mut self) -> Option<Overlay<'static>> {
        self.overlays.pop()
    }

    pub(crate) fn handle_post_event(&mut self, post_event: PostEvent) {
        match post_event.action {
            Action::PushLayer(mut overlay) => {
                overlay.component_mut().mount(&mut self.app);
                self.push_layer(overlay);
            }
            Action::Noop => {}
            Action::PopLayer(event) => {
                if let Some(mut overlay) = self.pop_layer() {
                    let event = overlay.component_mut().unmount(&mut self.app, event);
                    self.handle_post_event(event);
                }
            },
        }
    }
}
