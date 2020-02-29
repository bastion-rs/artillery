use failure::*;
use std::io;

use std::result;
use std::sync::mpsc::{RecvError, SendError};

/// Result type for operations that could result in an `ArtilleryError`
pub type Result<T> = result::Result<T, ArtilleryError>;

#[derive(Fail, Debug)]
pub enum ArtilleryError {
    // General Error Types
    #[fail(display = "Artillery :: Orphan Node Error: {}", _0)]
    OrphanNode(String),
    #[fail(display = "Artillery :: I/O error occurred: {}", _0)]
    Io(io::Error),
    #[fail(display = "Artillery :: Cluster Message Decode Error: {}", _0)]
    ClusterMessageDecode(String),
    #[fail(display = "Artillery :: Message Send Error: {}", _0)]
    Send(String),
    #[fail(display = "Artillery :: Message Receive Error: {}", _0)]
    Receive(String),
    #[fail(display = "Artillery :: Unexpected Error: {}", _0)]
    Unexpected(String),
    #[fail(display = "Artillery :: Decoding Error: {}", _0)]
    Decoding(String),
    #[fail(display = "Artillery :: Numeric Cast Error: {}", _0)]
    NumericCast(String),
}

impl From<io::Error> for ArtilleryError {
    fn from(e: io::Error) -> Self {
        ArtilleryError::Io(e)
    }
}

impl From<serde_json::error::Error> for ArtilleryError {
    fn from(e: serde_json::error::Error) -> Self {
        ArtilleryError::ClusterMessageDecode(e.to_string())
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for ArtilleryError {
    fn from(e: SendError<T>) -> Self {
        ArtilleryError::Send(e.to_string())
    }
}

impl From<std::sync::mpsc::RecvError> for ArtilleryError {
    fn from(e: RecvError) -> Self {
        ArtilleryError::Receive(e.to_string())
    }
}

impl From<std::str::Utf8Error> for ArtilleryError {
    fn from(e: std::str::Utf8Error) -> Self {
        ArtilleryError::Decoding(e.to_string())
    }
}

impl From<std::num::TryFromIntError> for ArtilleryError {
    fn from(e: std::num::TryFromIntError) -> Self {
        ArtilleryError::NumericCast(e.to_string())
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
