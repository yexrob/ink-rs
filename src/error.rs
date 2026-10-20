use std::fmt;

/// An error returned when a live app exits.
#[derive(Debug)]
pub enum Error {
    /// A terminal I/O failure, including a closed stdout pipe.
    Io(std::io::Error),
    /// An error supplied through [`crate::AppHandle::exit_with_error`].
    App(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::App(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => error.source(),
            Self::App(error) => error.source(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
