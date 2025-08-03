// Rusty Samplers: AKP to SFZ Converter - v0.9
//
// Professional-grade AKP to SFZ converter with modular architecture
//
// Features:
// - Comprehensive error handling with graceful recovery
// - Modular parsing architecture (legacy + future binrw support)
// - Professional parameter conversion algorithms
// - Detailed logging and progress reporting

use rusty_samplers::{conversion, error::ConversionError};
use std::env;
use std::fs::{self, File};
use std::path::Path;
use log::{info, error, debug};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("--- Rusty Samplers: AKP to SFZ Converter v0.9 ---");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- <path_to_akp_file>");
        println!("Example: cargo run -- samples/my_program.akp");
        println!(""); 
        println!("Options:");
        println!("  RUST_LOG=debug cargo run -- file.akp  # Enable debug logging");
        return Ok(());
    }
    
    let file_path_str = &args[1];
    info!("Processing file: {}", file_path_str);

    // Validate file exists and is readable
    if !Path::new(file_path_str).exists() {
        error!("File '{}' not found", file_path_str);
        return Ok(());
    }

    match convert_file(file_path_str) {
        Ok(()) => {
            info!("--- Conversion Complete ---");
        },
        Err(ConversionError::PartialSuccess { warning_count, warnings }) => {
            println!("\n--- Conversion Completed with Warnings ---");
            println!("Warning count: {}", warning_count);
            for warning in warnings {
                println!("  Warning: {}", warning);
            }
        },
        Err(e) => {
            error!("Conversion failed: {}", e);
            error!("Error category: {}", e.category());
            
            // Provide helpful suggestions based on error type
            match e.category() {
                "format" => {
                    println!("Suggestion: Verify this is a valid AKP file from an Akai S5000/S6000 sampler");
                },
                "parsing" => {
                    println!("Suggestion: File may be corrupted or use an unsupported AKP variant");
                },
                "io" => {
                    println!("Suggestion: Check file permissions and available disk space");
                },
                _ => {}
            }
            
            return Err(Box::new(e));
        }
    }

    Ok(())
}

fn convert_file(file_path_str: &str) -> Result<(), ConversionError> {
    debug!("Opening file for conversion");
    let file = File::open(file_path_str)?;
    
    debug!("Starting conversion process");
    let sfz_content = conversion::convert_akp_to_sfz(file)?;
    
    // Generate output path
    let input_path = Path::new(file_path_str);
    let sfz_path = input_path.with_extension("sfz");
    
    info!("Saving SFZ file to: {:?}", sfz_path);
    fs::write(&sfz_path, sfz_content)?;
    
    info!("Successfully created {:?}", sfz_path);
    
    Ok(())
}