use failure::Fail;
use std::io;

use std::result;

/// Result type for operations that could result in an `CraqError`
pub type Result<T> = result::Result<T, CraqError>;

#[derive(Fail, Debug)]
pub enum CraqError {
    #[fail(display = "Artillery :: CRAQ :: I/O error occurred: {}", _0)]
    IOError(io::Error),
    #[fail(display = "Artillery :: CRAQ :: Socket addr: {}", _0)]
    SocketAddrError(String),
    #[fail(display = "Artillery :: CRAQ :: Assertion failed: {}", _0)]
    AssertionError(String, failure::Backtrace),
    #[fail(display = "Artillery :: CRAQ :: Protocol error: {}", _0)]
    ProtocolError(thrift::Error),
    #[fail(display = "Artillery :: CRAQ :: Read error: {}", _0)]
    ReadError(String),
}

impl From<io::Error> for CraqError {
    fn from(e: io::Error) -> Self {
        CraqError::IOError(e)
    }
}

impl From<thrift::Error> for CraqError {
    fn from(e: thrift::Error) -> Self {
        CraqError::ProtocolError(e)
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

macro_rules! ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            return Err(CraqError::AssertionError($e.to_string(), failure::Backtrace::new()));
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)+) => {
        if !($cond) {
            return Err(CraqError::AssertionError(format!($fmt, $($arg)+).to_string(), failure::Backtrace::new()));
        }
    };
}
