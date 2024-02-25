use crate::app::App;

use super::{
    component::Component,
    event::{Action, PostEvent},
};

// Above should be data, this should be map.
pub struct ScreenManager {
    pub app: App,
    pub overlays: Vec<Box<dyn Component>>,
}

impl ScreenManager {
    pub fn push_layer<T: Component + 'static>(&mut self, component: T) {
        self.overlays.push(Box::new(component));
    }

    pub fn push_boxed_layer(&mut self, component: Box<dyn Component>) {
        self.overlays.push(component);
    }

    pub fn pop_layer(&mut self) -> Option<Box<dyn Component>> {
        self.overlays.pop()
    }

    pub(crate) fn handle_post_event(&mut self, post_event: PostEvent) {
        match post_event.action {
            Action::PushLayer(mut overlay) => {
                overlay.mount(&mut self.app);
                self.push_boxed_layer(overlay);
            }
            Action::Noop => {}
            Action::PopLayer(event) => {
                if let Some(mut overlay) = self.pop_layer() {
                    let event = overlay.unmount(&mut self.app, event);
                    self.handle_post_event(event);
                }
            }
        }
    }
}
