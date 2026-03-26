pub mod error;
pub mod types;
pub mod parser;
pub mod sfz;
pub mod dspreset;
pub mod validate;
pub mod samples;

pub use error::{AkpError, Result};
pub use types::{AkaiProgram, OutputFormat};
pub use parser::{validate_riff_header, parse_top_level_chunks};
pub use samples::{copy_samples, CopyConfig, CopyReport, SampleResult};

use std::path::Path;

/// Conversion function for GUI use — returns only the output string.
pub fn convert_file(input_path: &Path, format: OutputFormat) -> std::result::Result<String, String> {
    let (output, _program) = convert_file_with_program(input_path, format)?;
    Ok(output)
}

/// Like `convert_file()` but also returns the parsed `AkaiProgram`,
/// so callers can access `sample_paths()` for sample copying.
pub fn convert_file_with_program(input_path: &Path, format: OutputFormat) -> std::result::Result<(String, AkaiProgram), String> {
    use std::fs::File;

    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open file: {e}"))?;

    validate_riff_header(&mut file)
        .map_err(|e| format!("Invalid AKP file: {e}"))?;

    let file_size = file.metadata()
        .map_err(|e| format!("Failed to read file size: {e}"))?.len();

    let mut program = AkaiProgram::default();
    let progress = indicatif::ProgressBar::hidden();

    parse_top_level_chunks(&mut file, file_size, &mut program, &progress)
        .map_err(|e| format!("Failed to parse AKP chunks: {e}"))?;

    let output = match format {
        OutputFormat::Sfz => program.to_sfz_string(),
        OutputFormat::DecentSampler => program.to_dspreset_string(),
    };

    Ok((output, program))
}
