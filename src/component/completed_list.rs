use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crossterm::event::{KeyCode, MouseEvent, MouseEventKind};

use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, ListState};

use crate::actions::HelpAction;
use crate::app::{App, SelectedComponent};
use crate::view::{DrawableComponent, EventResult};
use crate::{actions, utils};

const COMPONENT_TYPE: SelectedComponent = SelectedComponent::CompletedTasks;

pub struct CompletedList {
    pub area: Rect,
    selected_index: Rc<RefCell<usize>>,
}

impl CompletedList {
    pub fn new(selected_index: Rc<RefCell<usize>>) -> Self {
        Self {
            area: Rect::default(),
            selected_index,
        }
    }

    fn selected(&self) -> Ref<usize> {
        self.selected_index.borrow()
    }

    fn selected_mut(&self) -> RefMut<usize> {
        self.selected_index.borrow_mut()
    }

    pub fn available_actions() -> Vec<HelpAction<'static>> {
        vec![HelpAction::new(
            KeyCode::Char('r'),
            "r",
            "Restores the selected task",
        )]
    }
}

impl DrawableComponent for CompletedList {
    fn draw(&self, app: &App, _: Rect, drawer: &mut crate::view::Drawer) {
        let theme = &app.theme;

        let selected_index = *self.selected();

        let completed_tasks: Vec<ListItem> = app
            .task_store
            .completed_tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let colour = if let SelectedComponent::CompletedTasks = app.selected_component {
                    if selected_index == i {
                        theme.selected_task_colour
                    } else {
                        Color::White
                    }
                } else {
                    Color::White
                };
                let content = Spans::from(Span::styled(
                    format!(
                        "{} {}",
                        task.time_completed.format("%d/%m/%y %-I:%M:%S %p"),
                        task.task.title
                    ),
                    Style::default().fg(colour),
                ));
                ListItem::new(content)
            })
            .collect();

        let completed_list = List::new(completed_tasks)
            .block(utils::generate_default_block(
                "Completed tasks",
                COMPONENT_TYPE,
                app,
            ))
            .style(Style::default().fg(Color::White));

        let mut completed_state = ListState::default();
        if !app.task_store.completed_tasks.is_empty() {
            completed_state.select(Some(selected_index));
        }

        drawer.draw_stateful_widget(completed_list, &mut completed_state, self.area);
    }

    fn key_pressed(&mut self, app: &mut App, key_code: crossterm::event::KeyCode) -> EventResult {
        let mut selected_index = self.selected_mut();

        let result = utils::handle_movement(
            key_code,
            &mut selected_index,
            app.task_store.completed_tasks.len(),
        );

        if result == EventResult::Consumed {
            return result;
        }

        if let KeyCode::Char('r') = key_code {
            actions::restore_task(app, &mut selected_index);
            return EventResult::Consumed;
        }

        EventResult::Ignored
    }

    fn mouse_event(
        &mut self,
        app: &mut App,
        MouseEvent { row, kind, .. }: crossterm::event::MouseEvent,
    ) -> EventResult {
        let row = row - self.area.y;
        if let MouseEventKind::ScrollUp = kind {
            if *self.selected_index.borrow() != 0 {
                *self.selected_index.borrow_mut() -= 1;
            }
        }

        if let MouseEventKind::ScrollDown = kind {
            if *self.selected_index.borrow() < app.task_store.completed_tasks.len() - 1 {
                *self.selected_index.borrow_mut() += 1;
            }
        }

        if let MouseEventKind::Down(_) = kind {
            if let COMPONENT_TYPE = app.selected_component {
            } else {
                app.selected_component = COMPONENT_TYPE;
            }
            if row == 0 {
                return EventResult::Ignored;
            }
            if *self.selected_index.borrow()
                > self.area.height as usize - 2
            {
                let new_index = *self.selected_index.borrow()
                    - (self.area.height as usize - 2)
                    + row as usize;
                *self.selected_index.borrow_mut() = new_index;
            } else {
                if row as usize > app.task_store.completed_tasks.len() {
                    *self.selected_index.borrow_mut() = app.task_store.completed_tasks.len() - 1;
                    return EventResult::Ignored;
                }
                *self.selected_index.borrow_mut() = row as usize - 1;
            }
        }
        EventResult::Ignored
    }

    fn update_layout(&mut self, rect: Rect) {
        self.area = rect;
    }
}
