use std::any::Any;
use std::backtrace::{Backtrace, BacktraceStatus};
use std::fmt;
use std::sync::Mutex;

/// An error returned when a live app exits.
#[derive(Debug)]
pub enum Error {
    /// A terminal I/O failure, including a closed stdout pipe.
    Io(std::io::Error),
    /// An error supplied through [`crate::AppHandle::exit_with_error`].
    App(Box<dyn std::error::Error + Send + Sync>),
    /// A panic caught while rendering a component or running a live effect.
    Panic(PanicError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::App(error) => error.fmt(f),
            Self::Panic(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => error.source(),
            Self::App(error) => error.source(),
            Self::Panic(_) => None,
        }
    }
}

/// The source position reported by the panic hook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanicLocation {
    /// Source path, as reported by `std::panic::Location::file`.
    pub file: String,
    /// One-based source line.
    pub line: u32,
    /// One-based source column.
    pub column: u32,
}

/// A component panic, retaining its original payload.
///
/// After `wait_until_exit`, `std::panic::resume_unwind(e.into_payload())`
/// gives the usual panic exit code 101. Nothing is caught under `panic = "abort"`.
/// A component's own `catch_unwind` suppresses std's message for panics it catches.
/// If another panic escapes that same guarded region, the hook takes the
/// uncaught path because it cannot distinguish it from a panic during unwinding.
#[derive(Debug)]
pub struct PanicError {
    message: String,
    location: Option<PanicLocation>,
    backtrace: Box<Backtrace>,
    payload: Mutex<Box<dyn Any + Send>>,
}

impl PanicError {
    pub(crate) fn new(
        payload: Box<dyn Any + Send>,
        location: Option<PanicLocation>,
        backtrace: Backtrace,
    ) -> Self {
        let message = if let Some(message) = payload.downcast_ref::<&str>() {
            (*message).to_owned()
        } else if let Some(message) = payload.downcast_ref::<String>() {
            message.clone()
        } else {
            "Box<dyn Any>".to_owned()
        };
        Self {
            message,
            location,
            backtrace: Box::new(backtrace),
            payload: Mutex::new(payload),
        }
    }

    /// The string payload, or `Box<dyn Any>` for other payload types.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The source position, when supplied by the panic hook.
    pub fn location(&self) -> Option<&PanicLocation> {
        self.location.as_ref()
    }

    /// Returns the original payload for downcasting or `resume_unwind`.
    pub fn into_payload(self) -> Box<dyn Any + Send> {
        self.payload
            .into_inner()
            .unwrap_or_else(|error| error.into_inner())
    }

    pub(crate) fn backtrace(&self) -> Option<String> {
        (self.backtrace.status() == BacktraceStatus::Captured).then(|| self.backtrace.to_string())
    }
}

impl fmt::Display for PanicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for PanicError {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
