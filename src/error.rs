use std::{io, num::ParseIntError};

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
