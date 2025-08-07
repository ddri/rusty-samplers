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

/// Enhanced recovery context for professional error handling
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    pub total_chunks_processed: usize,
    pub chunks_skipped: usize,
    pub chunks_recovered: usize,
    pub warnings: Vec<String>,
    pub file_position: u64,
}

impl RecoveryContext {
    pub fn new() -> Self {
        Self {
            total_chunks_processed: 0,
            chunks_skipped: 0,
            chunks_recovered: 0,
            warnings: Vec::new(),
            file_position: 0,
        }
    }
    
    /// Record a successful chunk recovery
    pub fn record_recovery(&mut self, chunk_type: &str, reason: &str) {
        self.chunks_recovered += 1;
        self.warnings.push(format!("Recovered malformed '{}' chunk: {}", chunk_type, reason));
    }
    
    /// Record a chunk skip (when recovery impossible)
    pub fn record_skip(&mut self, chunk_type: &str, reason: &str) {
        self.chunks_skipped += 1;
        self.warnings.push(format!("Skipped unrecoverable '{}' chunk: {}", chunk_type, reason));
    }
    
    /// Get recovery statistics for reporting
    pub fn stats(&self) -> String {
        format!(
            "Processed {} chunks ({} recovered, {} skipped)", 
            self.total_chunks_processed, 
            self.chunks_recovered, 
            self.chunks_skipped
        )
    }
}

impl Default for RecoveryContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Chunk-level recovery strategies for malformed AKP files
pub mod recovery {
    use super::*;
    use std::io::{Read, Seek, SeekFrom};
    use log::{warn, debug};

    /// Attempt to recover from malformed chunk by skipping to next valid chunk
    pub fn skip_to_next_chunk<R: Read + Seek>(
        reader: &mut R,
        current_offset: u64,
        context: &mut RecoveryContext,
    ) -> Result<Option<u64>> {
        const MAX_SKIP_BYTES: u64 = 4096; // Don't skip more than 4KB looking for recovery
        let start_pos = reader.stream_position().unwrap_or(current_offset);
        
        debug!("Attempting chunk recovery from offset {}", start_pos);
        
        // Look for next chunk signature (4-byte chunk ID followed by 4-byte size)
        let mut buffer = [0u8; 8];
        let mut bytes_scanned = 0;
        
        while bytes_scanned < MAX_SKIP_BYTES {
            match reader.read_exact(&mut buffer) {
                Ok(()) => {
                    // Check if this looks like a valid chunk header
                    if is_valid_chunk_header(&buffer) {
                        let recovery_pos = start_pos + bytes_scanned;
                        debug!("Found potential chunk at offset {}", recovery_pos);
                        
                        // Seek back to the start of this chunk
                        reader.seek(SeekFrom::Start(recovery_pos))?;
                        context.record_recovery("unknown", "found next valid chunk signature");
                        return Ok(Some(recovery_pos));
                    }
                    
                    // Move back 7 bytes so we scan with 1-byte overlap
                    reader.seek(SeekFrom::Current(-7))?;
                    bytes_scanned += 1;
                }
                Err(_) => break, // EOF or other error
            }
        }
        
        warn!("Unable to recover - no valid chunk found within {} bytes", MAX_SKIP_BYTES);
        context.record_skip("unknown", "no valid chunk signature found");
        Ok(None)
    }
    
    /// Check if 8 bytes look like a valid AKP chunk header
    pub fn is_valid_chunk_header(buffer: &[u8; 8]) -> bool {
        // Valid AKP chunk IDs we recognize
        const VALID_CHUNKS: &[&[u8; 4]] = &[
            b"prg ", b"kgrp", b"out ", b"zone", 
            b"smpl", b"tune", b"filt", b"env ", b"lfo "
        ];
        
        let chunk_id = &buffer[0..4];
        let chunk_size = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        
        // Check if chunk ID is valid and size is reasonable
        VALID_CHUNKS.iter().any(|&valid_id| chunk_id == valid_id) 
            && chunk_size < 1024 * 1024 // Reasonable chunk size limit (1MB)
            && chunk_size > 0
    }
    
    /// Attempt to recover keygroup data with partial information
    pub fn recover_keygroup_with_partial_data(
        available_chunks: &[&str],
        context: &mut RecoveryContext,
    ) -> crate::formats::common::Keygroup {
        use crate::formats::common::*;
        
        let mut keygroup = Keygroup::default();
        
        // Always create a valid default zone if none exists
        if !available_chunks.contains(&"zone") {
            keygroup.zone = Zone {
                low_key: 0,
                high_key: 127,
                low_vel: 1,
                high_vel: 127,
            };
            context.record_recovery("zone", "created default key/velocity ranges");
        }
        
        // If no sample chunk, create placeholder
        if !available_chunks.contains(&"smpl") {
            keygroup.sample = Some(Sample {
                filename: "missing_sample.wav".to_string(),
            });
            context.record_recovery("smpl", "created placeholder sample reference");
        }
        
        keygroup
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    
    #[test]
    fn test_recovery_context() {
        let mut context = RecoveryContext::new();
        context.record_recovery("kgrp", "fixed invalid size");
        context.record_skip("unknown", "unrecognized format");
        
        assert_eq!(context.chunks_recovered, 1);
        assert_eq!(context.chunks_skipped, 1);
        assert_eq!(context.warnings.len(), 2);
    }
    
    #[test]
    fn test_chunk_header_validation() {
        // Valid chunk header: "kgrp" + size 32
        let valid_header = [b'k', b'g', b'r', b'p', 32, 0, 0, 0];
        assert!(recovery::is_valid_chunk_header(&valid_header));
        
        // Invalid chunk header: random data
        let invalid_header = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        assert!(!recovery::is_valid_chunk_header(&invalid_header));
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