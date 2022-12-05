use std::{fmt::Display, num::ParseIntError};

#[derive(Debug)]
pub enum AppError {
    InvalidColour(),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InvalidColour() => write!(f, "Please use a valid colour"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<ParseIntError> for AppError {
    fn from(_: ParseIntError) -> AppError {
        AppError::InvalidColour()
    }
}
