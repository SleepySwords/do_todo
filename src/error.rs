use std::{io, num::ParseIntError};

use chrono::ParseError;
use io::Error as IoError;
use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Please use a valid colour")]
    InvalidColour,

    #[error("IO error: {0}")]
    IoError(IoError),

    #[error("JSON parsing error: {0}")]
    JsonError(JsonError),

    #[error("YAML parsing error: {0}")]
    YamlError(YamlError),

    #[error("Key parsing error: {0}")]
    InvalidKey(String),

    #[error("Invalid date: {0}")]
    InvalidDate(ParseError),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl AppError {
    pub fn invalid_state<T: Into<String>>(msg: T) -> AppError {
        AppError::InvalidState(msg.into())
    }
}

impl From<ParseIntError> for AppError {
    fn from(_: ParseIntError) -> AppError {
        AppError::InvalidColour
    }
}

impl From<JsonError> for AppError {
    fn from(err: JsonError) -> AppError {
        AppError::JsonError(err)
    }
}

impl From<YamlError> for AppError {
    fn from(err: YamlError) -> AppError {
        AppError::YamlError(err)
    }
}

impl From<IoError> for AppError {
    fn from(err: IoError) -> AppError {
        AppError::IoError(err)
    }
}

impl From<ParseError> for AppError {
    fn from(err: ParseError) -> AppError {
        AppError::InvalidDate(err)
    }
}
