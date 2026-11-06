use std::error::Error;
use std::fmt;

/// Core error type for transport operations
#[derive(Debug)]
pub enum TransportError {
    Network(String),
    Timeout(String),
    Authentication(String),
    NotFound(String),
    InvalidResponse(String),
    NotCapable(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Other(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(message) => write!(f, "Network error: {message}"),
            Self::Timeout(message) => write!(f, "Timeout: {message}"),
            Self::Authentication(message) => write!(f, "Authentication failed: {message}"),
            Self::NotFound(message) => write!(f, "Not found: {message}"),
            Self::InvalidResponse(message) => write!(f, "Invalid response: {message}"),
            Self::NotCapable(message) => write!(f, "Transport not capable: {message}"),
            Self::Io(error) => write!(f, "IO error: {error}"),
            Self::Json(error) => write!(f, "JSON error: {error}"),
            Self::Other(message) => write!(f, "{message}"),
        }
    }
}

impl Error for TransportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TransportError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<String> for TransportError {
    fn from(message: String) -> Self {
        Self::Other(message)
    }
}

pub type Result<T> = std::result::Result<T, TransportError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_error_other_displays_inner_message() {
        let err = TransportError::Other("ssh task failed: boom".into());
        assert_eq!(err.to_string(), "ssh task failed: boom");
    }

    #[test]
    fn transport_error_other_from_string_preserves_message() {
        let err: TransportError = "Failed to create HTTP client: tls".to_string().into();
        assert_eq!(err.to_string(), "Failed to create HTTP client: tls");
    }

    #[test]
    fn transport_error_variants_keep_stable_display_prefixes() {
        assert_eq!(
            TransportError::Network("down".into()).to_string(),
            "Network error: down"
        );
        assert_eq!(
            TransportError::Timeout("slow".into()).to_string(),
            "Timeout: slow"
        );
        assert_eq!(
            TransportError::Authentication("bad key".into()).to_string(),
            "Authentication failed: bad key"
        );
        assert_eq!(
            TransportError::NotFound("missing".into()).to_string(),
            "Not found: missing"
        );
        assert_eq!(
            TransportError::InvalidResponse("weird".into()).to_string(),
            "Invalid response: weird"
        );
        assert_eq!(
            TransportError::NotCapable("push".into()).to_string(),
            "Transport not capable: push"
        );
        let io = TransportError::from(std::io::Error::other("disk"));
        assert!(io.to_string().starts_with("IO error:"));
        assert!(io.source().is_some());
        let json = TransportError::from(serde_json::from_str::<u8>("nope").unwrap_err());
        assert!(json.to_string().starts_with("JSON error:"));
        assert!(json.source().is_some());
    }
}
