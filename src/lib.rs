// Re-export main types and functions for GUI
pub use crate::main::*;

// Import the main module
#[path = "main.rs"]
pub mod main;

// Additional public API for GUI integration
use std::path::Path;

#[derive(Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Sfz,
    DecentSampler,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Sfz
    }
}

// Conversion function for GUI use
pub fn convert_file(input_path: &Path, format: OutputFormat) -> Result<String, String> {
    use std::fs::File;
    use std::io::BufReader;
    
    // Open and validate the AKP file
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    // Parse the AKP file
    let mut program = crate::main::AkaiProgram::default();
    
    // Use a dummy progress bar for GUI conversion
    let progress = indicatif::ProgressBar::hidden();
    
    // Validate RIFF header
    crate::main::validate_riff_header(&mut file)
        .map_err(|e| format!("Invalid AKP file: {:?}", e))?;
    
    // Get file size for parsing and seek past RIFF header
    let file_size = file.metadata()
        .map_err(|e| format!("Failed to read file size: {}", e))?.len();
    
    // Skip the RIFF size field (4 bytes)
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::Current(4))
        .map_err(|e| format!("Failed to seek past RIFF header: {}", e))?;
    
    let end_pos = file_size;
    
    // Parse all chunks
    crate::main::parse_top_level_chunks(&mut file, end_pos, &mut program, &progress)
        .map_err(|e| format!("Failed to parse AKP chunks: {:?}", e))?;
    
    // Generate output based on format
    let output = match format {
        OutputFormat::Sfz => program.to_sfz_string(),
        OutputFormat::DecentSampler => program.to_dspreset_string(),
    };
    
    Ok(output)
}