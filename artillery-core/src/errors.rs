use failure::Backtrace;
use failure::*;
use std::io;
use std::option;
use std::result;

/// Result type for operations that could result in an `ExecutionError`
pub type Result<T> = result::Result<T, ArtilleryError>;

#[derive(Fail, Debug)]
pub enum ArtilleryError {
    // General Error Types
    #[fail(display = "Artillery :: Orphan Node Error: {}", _0)]
    OrphanNodeError(String),
    #[fail(display = "Artillery :: I/O error occurred: {}", _0)]
    IoError(io::Error)
}

impl From<io::Error> for ArtilleryError {
    fn from(e: io::Error) -> Self {
        ArtilleryError::IoError(e)
    }
}