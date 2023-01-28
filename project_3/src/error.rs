use anyhow::Error as AnyHowError;
use tracing::error;
use thiserror::Error ;
use serde::Serialize;
use std::io;

use sled::Error as ErrorSled;
pub type Result<T> = std::result::Result<T, CacheError>;

#[derive(Debug, Error)]
pub enum CacheError { 

    #[error("Could not find resource")]
    NotFound,

    #[error("Input / Output operation fails")]
    IoError(io::Error), 
    
    #[error("Serialisation Error")]
    SerdeError,

    #[error("Provided data was malformed")]
    MalformedData,

    #[error("Unexpected command type")]
    UnexpectedError,

    #[error("Key not found")]
    KeyNotFound,

    #[error(transparent)]
    SledError(ErrorSled),

    #[error("Server error: {0}")]
    ServerError(String)
}

impl From<io::Error> for CacheError { 
    fn from(value: io::Error) -> Self {
        CacheError::IoError(value)
    }
}

impl From<serde_json::Error> for CacheError { 
    fn from(value: serde_json::Error) -> Self {
        use serde_json::error::Category::{Data, Syntax};
        error!(err = ?value,  "JSON Serde error ocurred");

        match value.classify() { 
            Syntax | Data => CacheError::MalformedData,
            _ => CacheError::UnexpectedError
        }
    }
}

impl From<sled::Error> for CacheError {
    fn from(value: ErrorSled) -> Self {
        Self::SledError(value)
    }
}
