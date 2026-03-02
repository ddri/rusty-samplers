use std::fmt;
use std::io;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum AkpError {
    Io(io::Error),
    InvalidRiffHeader,
    InvalidAprgSignature,
    UnknownChunkType(String),
    InvalidChunkSize(String, u32),
    CorruptedChunk(String, String),
    InvalidKeyRange(u8, u8),
    InvalidVelocityRange(u8, u8),
    MissingRequiredChunk(String),
    InvalidParameterValue(String, u8),
}

impl fmt::Display for AkpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AkpError::Io(err) => write!(f, "I/O error: {err}"),
            AkpError::InvalidRiffHeader => write!(f, "Invalid file format: Expected RIFF header but found different signature"),
            AkpError::InvalidAprgSignature => write!(f, "Invalid file format: Expected APRG signature but found different signature (not an Akai program file)"),
            AkpError::UnknownChunkType(chunk) => write!(f, "Unknown chunk type '{chunk}' encountered"),
            AkpError::InvalidChunkSize(chunk, size) => write!(f, "Invalid size {size} for chunk '{chunk}'"),
            AkpError::CorruptedChunk(chunk, reason) => write!(f, "Corrupted '{chunk}' chunk: {reason}"),
            AkpError::InvalidKeyRange(low, high) => write!(f, "Invalid key range: low_key ({low}) must be <= high_key ({high})"),
            AkpError::InvalidVelocityRange(low, high) => write!(f, "Invalid velocity range: low_vel ({low}) must be <= high_vel ({high})"),
            AkpError::MissingRequiredChunk(chunk) => write!(f, "Missing required '{chunk}' chunk"),
            AkpError::InvalidParameterValue(param, value) => write!(f, "Invalid value {value} for parameter '{param}'"),
        }
    }
}

impl StdError for AkpError {}

impl From<io::Error> for AkpError {
    fn from(err: io::Error) -> Self {
        AkpError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, AkpError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_akp_error_display() {
        let error = AkpError::InvalidKeyRange(72, 60);
        assert_eq!(
            error.to_string(),
            "Invalid key range: low_key (72) must be <= high_key (60)"
        );

        let error = AkpError::InvalidRiffHeader;
        assert_eq!(
            error.to_string(),
            "Invalid file format: Expected RIFF header but found different signature"
        );
    }
}
