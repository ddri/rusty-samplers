use std::env;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle};

use rusty_samplers::{AkpError, AkaiProgram, OutputFormat, Result};
use rusty_samplers::parser::{validate_riff_header, parse_top_level_chunks};

fn main() -> Result<()> {
    println!("Rusty Samplers: Multi-Format Sampler Converter v1.0");
    println!();

    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("Usage:");
            println!("  Single file:  rusty-samplers-cli [OPTIONS] <path_to_akp_file>");
            println!("  Batch mode:   rusty-samplers-cli --batch [OPTIONS] <directory>");
            println!("  Help:         rusty-samplers-cli --help");
            println!();
            println!("Options:");
            println!("  --format sfz|ds     Output format (default: sfz)");
            return Ok(());
        }
        2 => {
            let arg = &args[1];
            match arg.as_str() {
                "--help" | "-h" => {
                    print_help();
                    return Ok(());
                }
                _ => {
                    if let Err(e) = run_conversion(arg, OutputFormat::Sfz) {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
        3 => {
            let first_arg = &args[1];
            let second_arg = &args[2];

            match first_arg.as_str() {
                "--batch" | "-b" => {
                    if let Err(e) = run_batch_conversion(second_arg, OutputFormat::Sfz) {
                        eprintln!("Batch Error: {e}");
                        std::process::exit(1);
                    }
                }
                "--format" => {
                    eprintln!("Error: Missing file path after format option.");
                    std::process::exit(1);
                }
                _ => {
                    eprintln!("Error: Invalid arguments. Use --help for usage information.");
                    std::process::exit(1);
                }
            }
        }
        4 => {
            let first_arg = &args[1];
            let second_arg = &args[2];
            let third_arg = &args[3];

            if first_arg == "--format" {
                let format = parse_format(second_arg)?;
                if let Err(e) = run_conversion(third_arg, format) {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: Invalid arguments. Use --help for usage information.");
                std::process::exit(1);
            }
        }
        5 => {
            let first_arg = &args[1];
            let second_arg = &args[2];
            let third_arg = &args[3];
            let fourth_arg = &args[4];

            if first_arg == "--batch" && second_arg == "--format" {
                let format = parse_format(third_arg)?;
                if let Err(e) = run_batch_conversion(fourth_arg, format) {
                    eprintln!("Batch Error: {e}");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: Invalid arguments. Use --help for usage information.");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Error: Too many arguments. Use --help for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn parse_format(format_str: &str) -> Result<OutputFormat> {
    match format_str.to_lowercase().as_str() {
        "sfz" => Ok(OutputFormat::Sfz),
        "ds" | "dspreset" | "decent" | "decentsampler" => Ok(OutputFormat::DecentSampler),
        _ => Err(AkpError::InvalidParameterValue("output_format".to_string(), 0))
    }
}

fn print_help() {
    println!("Rusty Samplers - Multi-Format Sampler Converter");
    println!();
    println!("USAGE:");
    println!("    rusty-samplers-cli [OPTIONS] <INPUT>");
    println!();
    println!("OPTIONS:");
    println!("    --format <FORMAT>    Output format: sfz, ds (default: sfz)");
    println!("    --batch, -b <DIR>    Convert all .akp files in directory");
    println!("    --help, -h           Show this help message");
    println!();
    println!("OUTPUT FORMATS:");
    println!("    sfz                  SFZ format (default)");
    println!("    ds, dspreset         Decent Sampler XML format");
    println!();
    println!("EXAMPLES:");
    println!("    rusty-samplers-cli my_sample.akp");
    println!("    rusty-samplers-cli --format ds my_sample.akp");
    println!("    rusty-samplers-cli --batch ./samples/");
    println!("    rusty-samplers-cli --batch --format ds ./samples/");
    println!();
    println!("FEATURES:");
    println!("    Comprehensive AKP chunk parsing");
    println!("    Advanced SFZ and Decent Sampler parameter mapping");
    println!("    Envelope, filter, and LFO conversion");
    println!("    Modulation routing support");
    println!("    Progress indicators and error handling");
    println!("    Multi-format output support");
}

fn run_batch_conversion(directory: &str, format: OutputFormat) -> Result<()> {
    let dir_path = Path::new(directory);
    if !dir_path.exists() {
        return Err(AkpError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory '{directory}' not found")
        )));
    }

    if !dir_path.is_dir() {
        return Err(AkpError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("'{directory}' is not a directory")
        )));
    }

    let mut akp_files = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("akp") {
            akp_files.push(path);
        }
    }

    if akp_files.is_empty() {
        println!("No .akp files found in directory: {directory}");
        return Ok(());
    }

    println!("Starting batch conversion of {} files...", akp_files.len());
    println!();

    let batch_progress = ProgressBar::new(akp_files.len() as u64);
    batch_progress.set_style(
        ProgressStyle::with_template(
            "[{bar:40.cyan/blue}] {pos:>3}/{len:3} files ({percent}%) {msg}"
        ).unwrap().progress_chars("##-")
    );

    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    for akp_file in &akp_files {
        let file_name = akp_file.file_name().unwrap().to_string_lossy();
        batch_progress.set_message(format!("Processing {file_name}"));

        match run_conversion(&akp_file.to_string_lossy(), format) {
            Ok(()) => {
                success_count += 1;
                batch_progress.println(format!("OK: {file_name}"));
            }
            Err(e) => {
                error_count += 1;
                let error_msg = format!("{file_name}: {e}");
                errors.push(error_msg.clone());
                batch_progress.println(format!("FAIL: {error_msg}"));
            }
        }

        batch_progress.inc(1);
    }

    batch_progress.finish_with_message("Batch conversion complete!");

    println!();
    println!("BATCH SUMMARY:");
    println!("   Successful: {success_count}");
    println!("   Failed:     {error_count}");
    println!("   Total:      {}", akp_files.len());

    if !errors.is_empty() {
        println!();
        println!("ERRORS:");
        for error in &errors {
            println!("   - {error}");
        }
    }

    Ok(())
}

fn run_conversion(file_path_str: &str, format: OutputFormat) -> Result<()> {
    if !Path::new(file_path_str).exists() {
        return Err(AkpError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File '{file_path_str}' not found")
        )));
    }

    let progress = ProgressBar::new(100);
    progress.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}/{len:3} {msg}"
        ).unwrap().progress_chars("##-")
    );

    progress.set_message("Opening file...");
    progress.inc(10);

    let mut file = File::open(file_path_str)?;

    progress.set_message("Validating RIFF header...");
    progress.inc(10);
    validate_riff_header(&mut file)?;

    progress.set_message("Parsing chunks...");
    progress.inc(20);
    let mut program = AkaiProgram::default();
    let file_len = file.metadata()?.len();
    parse_top_level_chunks(&mut file, file_len, &mut program, &progress)?;

    progress.set_message("Validating structure...");
    progress.inc(10);

    if program.keygroups.is_empty() {
        return Err(AkpError::MissingRequiredChunk("keygroup".to_string()));
    }

    let format_name = match format {
        OutputFormat::Sfz => "SFZ",
        OutputFormat::DecentSampler => "Decent Sampler",
    };

    println!("-> Generating {format_name} content...");
    let (output_content, file_extension) = match format {
        OutputFormat::Sfz => (program.to_sfz_string(), "sfz"),
        OutputFormat::DecentSampler => (program.to_dspreset_string(), "dspreset"),
    };

    let input_path = Path::new(file_path_str);
    let output_path = input_path.with_extension(file_extension);

    println!("-> Saving {format_name} file to: {output_path:?}");
    fs::write(&output_path, output_content)?;

    println!("\n--- Conversion Complete ---");
    println!("Successfully created {output_path:?}.");

    Ok(())
}
