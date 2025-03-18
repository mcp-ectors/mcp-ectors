use std::fmt;
use std::error::Error;

/// Custom error type for handling transport layer errors.
#[derive(Debug)]
pub enum TransportError {
    /// Represents network-related errors (e.g., connection issues).
    NetworkError(String),
    
    /// Represents protocol errors (e.g., malformed messages).
    ProtocolError(String),
    
    /// Represents internal server errors.
    InternalError(String),
    
    /// Represents missing or incorrect configuration.
    ConfigurationError(String),
}

impl TransportError {
    /// Creates a new `TransportError::NetworkError`.
    pub fn network_error(msg: impl Into<String>) -> Self {
        TransportError::NetworkError(msg.into())
    }

    /// Creates a new `TransportError::ProtocolError`.
    pub fn protocol_error(msg: impl Into<String>) -> Self {
        TransportError::ProtocolError(msg.into())
    }

    /// Creates a new `TransportError::InternalError`.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        TransportError::InternalError(msg.into())
    }

    /// Creates a new `TransportError::ConfigurationError`.
    pub fn configuration_error(msg: impl Into<String>) -> Self {
        TransportError::ConfigurationError(msg.into())
    }
}

// Implement `fmt::Display` to allow easy formatting of error messages.
impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            TransportError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            TransportError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            TransportError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

// Implement `std::error::Error` to allow for use with error handling functions and libraries.
impl Error for TransportError {}

// Helper function to quickly convert errors to a `TransportError`.
impl From<&str> for TransportError {
    fn from(s: &str) -> Self {
        TransportError::InternalError(s.to_string())
    }
}

impl From<String> for TransportError {
    fn from(s: String) -> Self {
        TransportError::InternalError(s)
    }
}

