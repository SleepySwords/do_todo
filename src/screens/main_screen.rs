use crate::{
    app::App,
    component::{completed_list::CompletedList, task_list::TaskList, viewer::Viewer},
    framework::{
        component::{Component, Drawer},
        event::{Action, PostEvent},
    },
    utils,
};
use crossterm::event::MouseEvent;
use tui::layout::{Constraint, Direction, Layout, Rect};

const MINIMUM_SCREEN: u16 = 100;

pub struct MainScreen {
    task_list: TaskList,
    completed_list: CompletedList,
    layout: Rect,
    viewer: Viewer,
}

impl MainScreen {
    pub fn new() -> MainScreen {
        // The use of a RefCell means that we have to be more carefull in where we borrow this
        // variable. Ie: No storing borrowed references.
        MainScreen {
            task_list: TaskList::new(),
            completed_list: CompletedList::new(),
            layout: Rect::default(),
            viewer: Viewer::new(),
        }
    }
}

impl Component for MainScreen {
    fn draw(&self, app: &App, drawer: &mut Drawer) {
        drawer.draw_component(app, &self.task_list);
        drawer.draw_component(app, &self.completed_list);
        drawer.draw_component(app, &self.viewer);
    }

    fn mouse_event(
        &mut self,
        app: &mut App,
        mouse_event: crossterm::event::MouseEvent,
    ) -> PostEvent {
        let MouseEvent { row, column, .. } = mouse_event;
        if utils::inside_rect((row, column), self.task_list.area) {
            self.task_list.mouse_event(app, mouse_event);
        } else if utils::inside_rect((row, column), self.completed_list.area) {
            self.completed_list.mouse_event(app, mouse_event);
        }
        PostEvent {
            propegate_further: true,
            action: Action::Noop,
        }
    }

    fn update_layout(&mut self, layout: Rect) {
        self.layout = layout;
        let (task_layout, completed_layout, viewer_layout) = if layout.width < MINIMUM_SCREEN {
            let main_chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(layout);
            (main_chunk[1], main_chunk[2], main_chunk[0])
        } else {
            let main_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(layout);

            let layout_chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(main_chunk[0]);

            (layout_chunk[0], layout_chunk[1], main_chunk[1])
        };

        self.task_list.update_layout(task_layout);
        self.completed_list.update_layout(completed_layout);
        self.viewer.update_layout(viewer_layout);
    }
}
