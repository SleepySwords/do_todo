use crossterm::event::{KeyCode, KeyEvent};
use tui_textarea::{CursorMove, Input};

use crate::{app::App, draw::PostEvent};

use super::{
    input_box::{InputBox, InputMode},
    Overlay,
};

pub enum VimMode {
    Normal,
    Insert,
    Visual,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Operator {
    Delete,
    Change,
    None,
}

pub struct Vim {
    pub mode: VimMode,
    pub operator: Operator,
    pub pending: char,
}

impl InputBox {
    pub fn vim_title(&self) -> String {
        let mode = match &self.input_mode {
            InputMode::Normal => todo!(),
            InputMode::Vim(vim) => match vim.mode {
                VimMode::Normal => "Normal",
                VimMode::Insert => "Insert",
                VimMode::Visual => "Visual",
            },
        };
        return format!("Vim - {}", mode);
    }

    fn motion(&mut self, key_event: KeyEvent) -> Option<&[CursorMove]> {
        let InputMode::Vim(ref mut vim) = self.input_mode else {
            return Some(&[]);
        };
        match key_event.code {
            KeyCode::Char('w') => {
                if vim.pending == 'i' {
                    Some(&[CursorMove::WordBack, CursorMove::WordForward])
                } else {
                    Some(&[CursorMove::WordForward])
                }
            }
            KeyCode::Char('b') => Some(&[CursorMove::WordBack]),
            KeyCode::Char('h') => Some(&[CursorMove::Back]),
            KeyCode::Char('l') => Some(&[CursorMove::Forward]),
            KeyCode::Char('k') => Some(&[CursorMove::Up]),
            KeyCode::Char('j') => Some(&[CursorMove::Down]),
            KeyCode::Char('i') => {
                if vim.operator != Operator::None {
                    vim.pending = 'i';
                }
                None
            }
            _ => None,
        }
    }

    pub fn input_vim(&mut self, key_event: KeyEvent) -> PostEvent {
        let InputMode::Vim(ref mut vim) = self.input_mode else {
            return PostEvent::noop(true);
        };
        match vim.mode {
            VimMode::Normal => match key_event.code {
                KeyCode::Enter => {
                    return self.submit();
                }
                KeyCode::Esc => {
                    return PostEvent::pop_overlay(false, |app: &mut App, overlay| {
                        if let Overlay::Input(InputBox {
                            prev_mode: Some(mode),
                            ..
                        }) = overlay
                        {
                            app.mode = mode;
                        }
                        PostEvent::noop(false)
                    })
                }
                KeyCode::Char('c') => {
                    vim.operator = Operator::Change;
                }
                KeyCode::Char('x') => {
                    self.text_area.delete_next_char();
                }
                KeyCode::Char('i') => {
                    if vim.operator == Operator::None {
                        vim.mode = VimMode::Insert;
                    } else {
                        vim.pending = 'i';
                    }
                }
                KeyCode::Char('w') => {
                    if vim.operator == Operator::Change || vim.operator == Operator::Delete {
                        if vim.pending == 'i' {
                            self.text_area.move_cursor(CursorMove::WordBack);
                        }
                        self.text_area.delete_next_word();
                        if vim.operator == Operator::Change {
                            vim.mode = VimMode::Insert;
                        }
                    } else {
                        self.text_area.move_cursor(CursorMove::WordForward)
                    }
                }
                KeyCode::Char('b') => self.text_area.move_cursor(CursorMove::WordBack),
                KeyCode::Char('h') => self.text_area.move_cursor(CursorMove::Back),
                KeyCode::Char('l') => self.text_area.move_cursor(CursorMove::Forward),
                KeyCode::Char('k') => self.text_area.move_cursor(CursorMove::Up),
                KeyCode::Char('j') => self.text_area.move_cursor(CursorMove::Down),
                KeyCode::Char('u') => {
                    self.text_area.undo();
                }
                KeyCode::Char('r') => {
                    self.text_area.redo();
                }
                KeyCode::Char('o') => {
                    self.text_area.move_cursor(CursorMove::End);
                    self.text_area.insert_newline();
                    vim.mode = VimMode::Insert;
                }
                _ => {}
            },
            VimMode::Insert => {
                if let KeyCode::Esc = key_event.code {
                    vim.mode = VimMode::Normal;
                } else {
                    self.text_area.input(Input::from(key_event));
                }
            }
            VimMode::Visual => todo!(),
        }

        return PostEvent::noop(false);
    }
}
