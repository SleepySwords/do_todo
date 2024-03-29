use crossterm::event::{KeyCode, KeyEvent};
use tui_textarea::{CursorMove, Input};

use crate::framework::event::{AppEvent, PostEvent};

use super::input_box::{InputBox, InputMode};

pub enum VimMode {
    Normal,
    Insert,
    Visual,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Operator {
    Delete,
    Change,
    // Yank,
    None,
}

pub struct Vim {
    pub mode: VimMode,
    pub operator: Operator,
    pub pending: Option<char>,
}

impl InputBox {
    pub fn input_vim(&mut self, key_event: KeyEvent) -> PostEvent {
        let InputMode::Vim(ref mut vim) = self.input_mode else {
            return PostEvent::noop(true);
        };
        match vim.mode {
            VimMode::Normal | VimMode::Visual => match key_event.code {
                KeyCode::Char('$') => {
                    if vim.operator == Operator::Change || vim.operator == Operator::Delete {
                        self.text_area.delete_line_by_end();
                        if vim.operator == Operator::Change {
                            vim.mode = VimMode::Insert;
                        }
                        vim.operator = Operator::None;
                    } else {
                        self.text_area.move_cursor(CursorMove::End);
                    }
                }
                KeyCode::Char('^') => {
                    if vim.operator == Operator::Change || vim.operator == Operator::Delete {
                        self.text_area.delete_line_by_head();
                        if vim.operator == Operator::Change {
                            vim.mode = VimMode::Insert;
                        }
                        vim.operator = Operator::None;
                    } else {
                        self.text_area.move_cursor(CursorMove::Head);
                    }
                }
                KeyCode::Char('w') => {
                    if vim.operator == Operator::Change || vim.operator == Operator::Delete {
                        if vim.pending == Some('i') {
                            self.text_area.move_cursor(CursorMove::WordForward);
                            self.text_area.move_cursor(CursorMove::WordBack);
                            vim.pending = None;
                        }
                        self.text_area.delete_next_word();
                        if vim.operator == Operator::Change {
                            vim.mode = VimMode::Insert;
                        }
                        vim.operator = Operator::None;
                    } else {
                        self.text_area.move_cursor(CursorMove::WordForward)
                    }
                }
                KeyCode::Char('b') => {
                    if vim.operator == Operator::Change || vim.operator == Operator::Delete {
                        self.text_area.delete_word();
                        if vim.operator == Operator::Change {
                            vim.mode = VimMode::Insert;
                        }
                        vim.operator = Operator::None;
                    } else {
                        self.text_area.move_cursor(CursorMove::WordBack)
                    }
                }
                KeyCode::Char('h') => self.text_area.move_cursor(CursorMove::Back),
                KeyCode::Char('l') => self.text_area.move_cursor(CursorMove::Forward),
                KeyCode::Char('j') => self.text_area.move_cursor(CursorMove::Down),
                KeyCode::Char('k') => self.text_area.move_cursor(CursorMove::Up),
                _ => {}
            },
            VimMode::Insert => {
                if let KeyCode::Esc = key_event.code {
                    vim.mode = VimMode::Normal;
                    return PostEvent::noop(false);
                } else if let KeyCode::Enter = key_event.code {
                    return self.submit();
                } else if let KeyCode::Tab = key_event.code {
                    self.text_area.insert_newline();
                } else {
                    self.text_area.input(Input::from(key_event));
                }
            }
        }

        match vim.mode {
            VimMode::Normal => match key_event.code {
                KeyCode::Enter => {
                    return self.submit();
                }
                KeyCode::Esc => {
                    if vim.operator != Operator::None {
                        vim.operator = Operator::None;
                    } else {
                        return PostEvent::pop_layer(Some(AppEvent::Cancel));
                    }
                }
                KeyCode::Char('q') => {
                    return PostEvent::pop_layer(Some(AppEvent::Cancel));
                }
                KeyCode::Char('c') => {
                    if vim.operator == Operator::Change {
                        self.text_area.move_cursor(CursorMove::Head);
                        self.text_area.delete_line_by_end();
                        vim.operator = Operator::None;
                        vim.mode = VimMode::Insert;
                    } else {
                        vim.operator = Operator::Change;
                    }
                }
                KeyCode::Char('d') => {
                    if vim.operator == Operator::Delete {
                        if self.text_area.cursor().0 == self.text_area.lines().len() - 1 {
                            self.text_area.move_cursor(CursorMove::End);
                            self.text_area.start_selection();
                            self.text_area.move_cursor(CursorMove::Up);
                            self.text_area.move_cursor(CursorMove::End);
                            self.text_area.delete_line_by_head();
                        } else {
                            self.text_area.move_cursor(CursorMove::Head);
                            self.text_area.start_selection();
                            self.text_area.move_cursor(CursorMove::Down);
                            self.text_area.delete_line_by_end();
                        }
                        vim.operator = Operator::None;
                    } else {
                        vim.operator = Operator::Delete;
                    }
                }
                KeyCode::Char('x') => {
                    self.text_area.delete_next_char();
                    vim.operator = Operator::None;
                }
                KeyCode::Char('v') => {
                    self.text_area.start_selection();
                    vim.mode = VimMode::Visual;
                    vim.operator = Operator::None;
                }
                KeyCode::Char('i') => {
                    if vim.operator == Operator::None {
                        vim.mode = VimMode::Insert;
                        vim.operator = Operator::None;
                    } else {
                        vim.pending = Some('i');
                    }
                }
                KeyCode::Char('a') => {
                    self.text_area.move_cursor(CursorMove::Forward);
                    vim.mode = VimMode::Insert;
                    vim.operator = Operator::None;
                }
                KeyCode::Char('u') => {
                    self.text_area.undo();
                    vim.operator = Operator::None;
                }
                KeyCode::Char('r') => {
                    self.text_area.redo();
                    vim.operator = Operator::None;
                }
                KeyCode::Char('o') => {
                    self.text_area.move_cursor(CursorMove::End);
                    self.text_area.insert_newline();
                    vim.mode = VimMode::Insert;
                    vim.operator = Operator::None;
                }
                KeyCode::Char('O') => {
                    self.text_area.move_cursor(CursorMove::Head);
                    self.text_area.insert_newline();
                    self.text_area.move_cursor(CursorMove::Up);
                    vim.mode = VimMode::Insert;
                    vim.operator = Operator::None;
                }
                KeyCode::Char('p') => {
                    self.text_area.paste();
                    vim.operator = Operator::None;
                }
                _ => {}
            },
            VimMode::Visual => match key_event.code {
                KeyCode::Esc | KeyCode::Char('v') => {
                    self.text_area.cancel_selection();
                    vim.mode = VimMode::Normal;
                }
                KeyCode::Enter => {
                    return self.submit();
                }
                KeyCode::Char('d') | KeyCode::Char('x') => {
                    self.text_area.delete_char();
                    vim.mode = VimMode::Normal;
                }
                KeyCode::Char('c') => {
                    self.text_area.delete_char();
                    vim.mode = VimMode::Insert;
                }
                KeyCode::Char('y') => {
                    self.text_area.copy();
                    vim.mode = VimMode::Normal;
                }
                KeyCode::Char('p') => {
                    self.text_area.paste();
                }
                _ => {}
            },
            _ => {}
        }

        PostEvent::noop(false)
    }
}
