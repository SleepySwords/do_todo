use super::component::Component;

pub enum Action {
    PopLayer(Option<AppEvent>),
    PushLayer(Box<dyn Component>),
    Noop,
}

#[derive(PartialEq, Eq)]
pub enum AppEvent {
    Submit,
    Cancel,
}

pub struct PostEvent {
    pub propegate_further: bool,
    pub action: Action,
}

impl PostEvent {
    pub fn noop(propagate_further: bool) -> PostEvent {
        PostEvent {
            propegate_further: propagate_further,
            action: Action::Noop,
        }
    }

    /// Unmounts the top on the stack
    ///
    /// # Arguments
    ///
    /// * `event` - The event that is passed to the `unmount` function of the component
    pub fn pop_layer(event: Option<AppEvent>) -> PostEvent {
        PostEvent {
            propegate_further: false,
            action: Action::PopLayer(event),
        }
    }

    pub fn push_layer<T: Component + 'static>(overlay: T) -> PostEvent {
        PostEvent {
            propegate_further: false,
            action: Action::PushLayer(Box::new(overlay)),
        }
    }
}
