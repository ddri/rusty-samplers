// AKP format type definitions
// Common types used by both legacy and binrw parsers

use crate::formats::common::*;

/// Complete AKP program data
#[derive(Debug, Default)]
pub struct AkaiProgram {
    pub header: Option<ProgramHeader>,
    pub keygroups: Vec<Keygroup>,
}

/// RIFF chunk header information
#[derive(Debug)]
pub struct RiffChunkHeader {
    pub id: String,
    pub size: u32,
}

impl AkaiProgram {
    /// Create a new empty AKP program
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a keygroup to this program
    pub fn add_keygroup(&mut self, keygroup: Keygroup) {
        self.keygroups.push(keygroup);
    }
    
    /// Get the number of valid keygroups (with samples)
    pub fn valid_keygroup_count(&self) -> usize {
        self.keygroups.iter().filter(|kg| kg.is_valid()).count()
    }
    
    /// Get program statistics for logging/reporting
    pub fn stats(&self) -> ProgramStats {
        ProgramStats {
            total_keygroups: self.keygroups.len(),
            valid_keygroups: self.valid_keygroup_count(),
            has_header: self.header.is_some(),
        }
    }
}

/// Statistics about a parsed AKP program
#[derive(Debug, Clone)]
pub struct ProgramStats {
    pub total_keygroups: usize,
    pub valid_keygroups: usize,
    pub has_header: bool,
}

impl std::fmt::Display for ProgramStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Program: {} keygroups ({} valid), header: {}",
            self.total_keygroups,
            self.valid_keygroups,
            if self.has_header { "yes" } else { "no" }
        )
    }
}