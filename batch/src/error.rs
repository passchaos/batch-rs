//! Error and Result module.

use std::fmt;
use std::result::Result as StdResult;
use failure::{Backtrace, Context, Fail};

/// `Error` type for the batch crate. Implements `Fail`.
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

/// A set of errors that can occur interacting with queues & workers.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    /// Couldn't serialize `Task` or `Job`.
    #[fail(display = "Couldn't serialize Task or Job: {}", _0)]
    Serialization(#[cause] ::serde_json::Error),

    /// Couldn't deserialize `Task` or `Job`.
    #[fail(display = "Couldn't deserialize Task or Job: {}", _0)]
    Deserialization(#[cause] ::serde_json::Error),

    /// Couldn't create Tokio reactor
    #[fail(display = "Couldn't create Tokio reactor: {}", _0)]
    Reactor(#[cause] ::std::io::Error),

    /// Generic I/O error
    #[fail(display = "Generic I/O error: {}", _0)]
    Io(#[cause] ::std::io::Error),

    /// Couldn't spawn child process and execute given job.
    #[fail(display = "Couldn't spawn child process and execute given job: {}", _0)]
    SubProcessManagement(#[cause] ::std::io::Error),

    /// The broker wasn't provided any queue.
    #[fail(display = "A handle to a Tokio reactor was not provided")]
    NoHandle,

    /// The given URL's scheme is not handled.
    #[fail(display = "This URL scheme is invalid: {}", _0)]
    InvalidScheme(::std::string::String),

    /// An error occured in the RabbitMQ broker.
    #[fail(display = "An error occured in the RabbitMQ broker: {}", _0)]
    Rabbitmq(#[cause] ::std::io::Error),
}

impl Error {
    /// Returns the underlying `Kind` of this error
    pub(crate) fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }

    /// Returns true if the error is from the serialization of a `Job` or a `Task`.
    pub fn is_serialization(&self) -> bool {
        match *self.kind() {
            ErrorKind::Serialization(_) => true,
            _ => false,
        }
    }

    /// Returns true if the error is from the deserialization of a `Job` or a `Task`.
    pub fn is_deserialization(&self) -> bool {
        match *self.kind() {
            ErrorKind::Deserialization(_) => true,
            _ => false,
        }
    }

    /// Returns true if the error is from the underlying I/O event loop.
    pub fn is_reactor(&self) -> bool {
        match *self.kind() {
            ErrorKind::Reactor(_) => true,
            _ => false,
        }
    }

    /// Returns true if the error is a generic I/O one.
    pub fn is_generic_io(&self) -> bool {
        match *self.kind() {
            ErrorKind::Io(_) => true,
            _ => false,
        }
    }

    /// Returns true if the error is from the spawning or execution of a subprocess.
    pub fn is_subprocess(&self) -> bool {
        match *self.kind() {
            ErrorKind::SubProcessManagement(_) => true,
            _ => false,
        }
    }

    /// Returns true if the error is from a missing Tokio reactor handle.
    pub fn is_no_handle(&self) -> bool {
        match *self.kind() {
            ErrorKind::NoHandle => true,
            _ => false,
        }
    }

    /// Returns true if the error is from the `RabbitMQ` connection.
    pub fn is_rabbitmq(&self) -> bool {
        match *self.kind() {
            ErrorKind::Rabbitmq(_) => true,
            _ => false,
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(inner: ErrorKind) -> Error {
        Error {
            inner: Context::new(inner),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}

/// Result type returned from `batch` functions that can fail.
pub type Result<T> = StdResult<T, Error>;
