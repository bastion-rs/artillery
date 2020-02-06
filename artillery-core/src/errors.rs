use failure::*;
use std::io;
use std::option;
use std::result;
use std::sync::mpsc::SendError;

/// Result type for operations that could result in an `ArtilleryError`
pub type Result<T> = result::Result<T, ArtilleryError>;

#[derive(Fail, Debug)]
pub enum ArtilleryError {
    // General Error Types
    #[fail(display = "Artillery :: Orphan Node Error: {}", _0)]
    OrphanNodeError(String),
    #[fail(display = "Artillery :: I/O error occurred: {}", _0)]
    IoError(io::Error),
    #[fail(display = "Artillery :: Cluster Message Decode Error: {}", _0)]
    ClusterMessageDecodeError(String),
    #[fail(display = "Artillery :: Message Send Error: {}", _0)]
    SendError(String),
    #[fail(display = "Artillery :: Unexpected Error: {}", _0)]
    UnexpectedError(String),
}

impl From<io::Error> for ArtilleryError {
    fn from(e: io::Error) -> Self {
        ArtilleryError::IoError(e)
    }
}

impl From<serde_json::error::Error> for ArtilleryError {
    fn from(e: serde_json::error::Error) -> Self {
        ArtilleryError::ClusterMessageDecodeError(e.to_string())
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for ArtilleryError {
    fn from(e: SendError<T>) -> Self {
        ArtilleryError::SendError(e.to_string())
    }
}

#[macro_export]
macro_rules! bail {
    ($kind:expr, $e:expr) => {
        return Err($kind($e.to_owned()));
    };
    ($kind:expr, $fmt:expr, $($arg:tt)+) => {
        return Err($kind(format!($fmt, $($arg)+).to_owned()));
    };
}