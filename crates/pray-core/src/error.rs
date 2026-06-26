use std::fmt::{self, Display, Formatter};
use std::io;

pub type PrayResult<T> = Result<T, PrayError>;

#[derive(Debug)]
pub enum PrayError {
    Parse { kind: &'static str, message: String },
    Manifest(String),
    Resolution(String),
    Integrity(String),
    Render(String),
    Verify(String),
    Unsupported(String),
    Io(io::Error),
}

impl PrayError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Parse { .. } => 2,
            Self::Manifest(_) => 1,
            Self::Resolution(_) => 3,
            Self::Integrity(_) => 4,
            Self::Render(_) => 5,
            Self::Verify(_) => 6,
            Self::Unsupported(_) => 8,
            Self::Io(_) => 1,
        }
    }
}

impl Display for PrayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { kind, message } => write!(f, "{} parse error: {}", kind, message),
            Self::Manifest(message) => write!(f, "manifest error: {}", message),
            Self::Resolution(message) => write!(f, "resolution error: {}", message),
            Self::Integrity(message) => write!(f, "integrity error: {}", message),
            Self::Render(message) => write!(f, "render error: {}", message),
            Self::Verify(message) => write!(f, "verify error: {}", message),
            Self::Unsupported(message) => write!(f, "unsupported feature: {}", message),
            Self::Io(error) => Display::fmt(error, f),
        }
    }
}

impl std::error::Error for PrayError {}

impl From<io::Error> for PrayError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for PrayError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Resolution(error.to_string())
    }
}
