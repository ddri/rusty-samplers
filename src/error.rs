// Error handling for Rusty Samplers
// Comprehensive error types with context and recovery strategies

use thiserror::Error;
use std::io;

/// Comprehensive error types for AKP to SFZ conversion
#[derive(Error, Debug)]
pub enum ConversionError {
    /// IO operation failed
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    /// Invalid RIFF header in AKP file
    #[error("Invalid RIFF header: expected 'RIFF' but found '{found}'")]
    InvalidRiffHeader { found: String },
    
    /// Invalid format identifier in AKP file
    #[error("Invalid format: expected 'APRG' but found '{found}'")]
    InvalidFormat { found: String },
    
    /// Malformed chunk encountered during parsing
    #[error("Malformed chunk '{chunk_type}' at offset {offset}: {reason}")]
    MalformedChunk {
        chunk_type: String,
        offset: u64,
        reason: String,
        recoverable: bool,
    },
    
    /// Required chunk missing from AKP file
    #[error("Missing required chunk: '{chunk_type}'")]
    MissingChunk { chunk_type: String },
    
    /// UTF-8 conversion error
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    
    /// Binary parsing error (when using binrw)
    #[cfg(feature = "binrw-parser")]
    #[error("Binary parsing error: {0}")]
    BinRw(#[from] binrw::Error),
    
    /// Partial conversion completed with warnings
    #[error("Partial conversion completed with {warning_count} warnings")]
    PartialSuccess { 
        warning_count: usize, 
        warnings: Vec<String>,
    },
    
    /// Conversion parameter out of valid range
    #[error("Parameter '{parameter}' value {value} out of range [{min}, {max}]")]
    ParameterOutOfRange {
        parameter: String,
        value: f64,
        min: f64,
        max: f64,
    },
    
    /// Custom error for specific scenarios
    #[error("Conversion error: {message}")]
    Custom { message: String },
}

impl ConversionError {
    /// Check if this error allows for recovery and continued processing
    pub fn is_recoverable(&self) -> bool {
        match self {
            ConversionError::MalformedChunk { recoverable, .. } => *recoverable,
            ConversionError::ParameterOutOfRange { .. } => true,
            ConversionError::Utf8Error(_) => true,
            _ => false,
        }
    }
    
    /// Get the error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            ConversionError::Io(_) => "io",
            ConversionError::InvalidRiffHeader { .. } => "format",
            ConversionError::InvalidFormat { .. } => "format",
            ConversionError::MalformedChunk { .. } => "parsing",
            ConversionError::MissingChunk { .. } => "parsing",
            ConversionError::Utf8Error(_) => "encoding",
            #[cfg(feature = "binrw-parser")]
            ConversionError::BinRw(_) => "parsing",
            ConversionError::PartialSuccess { .. } => "partial",
            ConversionError::ParameterOutOfRange { .. } => "parameter",
            ConversionError::Custom { .. } => "custom",
        }
    }
    
    /// Create a recoverable malformed chunk error
    pub fn recoverable_chunk_error(
        chunk_type: impl Into<String>,
        offset: u64,
        reason: impl Into<String>,
    ) -> Self {
        ConversionError::MalformedChunk {
            chunk_type: chunk_type.into(),
            offset,
            reason: reason.into(),
            recoverable: true,
        }
    }
    
    /// Create a non-recoverable malformed chunk error
    pub fn fatal_chunk_error(
        chunk_type: impl Into<String>,
        offset: u64,
        reason: impl Into<String>,
    ) -> Self {
        ConversionError::MalformedChunk {
            chunk_type: chunk_type.into(),
            offset,
            reason: reason.into(),
            recoverable: false,
        }
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, ConversionError>;

/// Recovery-aware parsing wrapper for error handling strategies
pub fn parse_with_recovery<T, F>(
    operation: F,
    context: &str,
) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    match operation() {
        Ok(result) => Ok(result),
        Err(ConversionError::Io(io_err)) if io_err.kind() == io::ErrorKind::UnexpectedEof => {
            Err(ConversionError::recoverable_chunk_error(
                "unknown",
                0,
                format!("Unexpected EOF during {}", context),
            ))
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recoverability() {
        let recoverable = ConversionError::recoverable_chunk_error("kgrp", 1024, "test error");
        assert!(recoverable.is_recoverable());
        
        let fatal = ConversionError::fatal_chunk_error("prg", 0, "critical error");
        assert!(!fatal.is_recoverable());
    }
    
    #[test]
    fn test_error_categories() {
        let io_err = ConversionError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        assert_eq!(io_err.category(), "io");
        
        let parse_err = ConversionError::MalformedChunk {
            chunk_type: "zone".to_string(),
            offset: 512,
            reason: "invalid data".to_string(),
            recoverable: true,
        };
        assert_eq!(parse_err.category(), "parsing");
    }
}