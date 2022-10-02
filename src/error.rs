use std::{fmt::Display, num::ParseIntError};

#[derive(Debug)]
pub enum AppError {
    InvalidColour(ParseIntError),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InvalidColour(_) => write!(f, "Please use a valid colour"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> AppError {
        AppError::InvalidColour(err)
    }
}
