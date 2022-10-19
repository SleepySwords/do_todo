use std::{fmt::Display, num::ParseIntError};

#[derive(Debug)]
pub enum AppError {
    InvalidColour(ParseIntError),
    InvalidContext,
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InvalidColour(_) => write!(f, "Please use a valid colour"),
            // TODO: Provide more details
            AppError::InvalidContext => write!(f, "An internal error occured"),
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
