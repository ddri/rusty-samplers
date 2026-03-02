pub mod error;
pub mod types;
pub mod parser;
pub mod sfz;
pub mod dspreset;

pub use error::{AkpError, Result};
pub use types::{AkaiProgram, OutputFormat};
pub use parser::{validate_riff_header, parse_top_level_chunks};

use std::path::Path;
use std::io::{Seek, SeekFrom};

/// Conversion function for GUI use
pub fn convert_file(input_path: &Path, format: OutputFormat) -> std::result::Result<String, String> {
    use std::fs::File;

    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    validate_riff_header(&mut file)
        .map_err(|e| format!("Invalid AKP file: {}", e))?;

    let file_size = file.metadata()
        .map_err(|e| format!("Failed to read file size: {}", e))?.len();

    file.seek(SeekFrom::Current(4))
        .map_err(|e| format!("Failed to seek past RIFF header: {}", e))?;

    let end_pos = file_size;

    let mut program = AkaiProgram::default();
    let progress = indicatif::ProgressBar::hidden();

    parse_top_level_chunks(&mut file, end_pos, &mut program, &progress)
        .map_err(|e| format!("Failed to parse AKP chunks: {}", e))?;

    let output = match format {
        OutputFormat::Sfz => program.to_sfz_string(),
        OutputFormat::DecentSampler => program.to_dspreset_string(),
    };

    Ok(output)
}
