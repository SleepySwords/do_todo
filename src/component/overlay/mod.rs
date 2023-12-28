use self::{input_box::InputBox, dialog::DialogBox, fuzzy::FuzzyBox};

pub mod dialog;
pub mod fuzzy;
pub mod input_box;

pub enum OverlayContext<'a> {
    InputBox(InputBox),
    DialogBox(DialogBox<'a>),
    FuzzyBox(FuzzyBox<'a>)
}
