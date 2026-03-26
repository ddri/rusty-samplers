use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use rusty_samplers::{AkpError, AkaiProgram, OutputFormat, Result, CopyConfig, copy_samples};
use rusty_samplers::parser::{validate_riff_header, parse_top_level_chunks};

#[derive(Parser)]
#[command(name = "rusty-samplers-cli")]
#[command(about = "Multi-Format Sampler Converter — converts Akai AKP files to SFZ and Decent Sampler formats")]
#[command(version)]
struct Cli {
    /// Input AKP file or directory (with --batch)
    input: PathBuf,

    /// Output format: sfz, ds
    #[arg(short, long, default_value = "sfz", value_parser = parse_format)]
    format: OutputFormat,

    /// Batch convert all .akp files in a directory
    #[arg(short, long)]
    batch: bool,

    /// Copy referenced sample files alongside the output preset
    #[arg(long)]
    copy_samples: bool,

    /// Directory to search for source sample files (default: same as input)
    #[arg(long)]
    sample_dir: Option<PathBuf>,
}

fn parse_format(s: &str) -> std::result::Result<OutputFormat, String> {
    match s.to_lowercase().as_str() {
        "sfz" => Ok(OutputFormat::Sfz),
        "ds" | "dspreset" | "decent" | "decentsampler" => Ok(OutputFormat::DecentSampler),
        other => Err(format!("Unknown format '{other}'. Valid formats: sfz, ds")),
    }
}

fn main() {
    let cli = Cli::parse();

    let result = if cli.batch {
        run_batch_conversion(&cli.input, cli.format, cli.copy_samples, cli.sample_dir.as_deref())
    } else {
        run_conversion(&cli.input, cli.format, cli.copy_samples, cli.sample_dir.as_deref())
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run_batch_conversion(directory: &Path, format: OutputFormat, do_copy_samples: bool, sample_dir: Option<&Path>) -> Result<()> {
    if !directory.exists() {
        return Err(AkpError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory '{}' not found", directory.display()),
        )));
    }

    if !directory.is_dir() {
        return Err(AkpError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("'{}' is not a directory", directory.display()),
        )));
    }

    let mut akp_files = Vec::new();
    collect_akp_files(directory, &mut akp_files)?;
    akp_files.sort();

    if akp_files.is_empty() {
        println!("No .akp files found in directory: {}", directory.display());
        return Ok(());
    }

    println!("Starting batch conversion of {} files...", akp_files.len());
    println!();

    let batch_progress = ProgressBar::new(akp_files.len() as u64);
    batch_progress.set_style(
        ProgressStyle::with_template(
            "[{bar:40.cyan/blue}] {pos:>3}/{len:3} files ({percent}%) {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    for akp_file in &akp_files {
        let file_name = akp_file.file_name().unwrap_or(akp_file.as_os_str()).to_string_lossy();
        batch_progress.set_message(format!("Processing {file_name}"));

        match run_conversion(akp_file, format, do_copy_samples, sample_dir) {
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

fn collect_akp_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_akp_files(&path, files)?;
        } else if path.extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("akp"))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn run_conversion(file_path: &Path, format: OutputFormat, do_copy_samples: bool, sample_dir: Option<&Path>) -> Result<()> {
    if !file_path.exists() {
        return Err(AkpError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File '{}' not found", file_path.display()),
        )));
    }

    let progress = ProgressBar::new(100);
    progress.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}/{len:3} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    progress.set_message("Opening file...");
    progress.inc(10);

    let mut file = File::open(file_path)?;

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

    progress.set_message(format!("Generating {format_name} output..."));
    let (output_content, file_extension) = match format {
        OutputFormat::Sfz => (program.to_sfz_string(), "sfz"),
        OutputFormat::DecentSampler => (program.to_dspreset_string(), "dspreset"),
    };

    let output_path = file_path.with_extension(file_extension);

    progress.set_message("Writing output...");
    fs::write(&output_path, output_content)?;

    progress.finish_with_message(format!("Created {}", output_path.display()));

    if do_copy_samples {
        let search = sample_dir
            .unwrap_or_else(|| file_path.parent().unwrap_or(Path::new(".")));
        let output_dir = output_path.parent().unwrap_or(Path::new("."));
        let sample_paths = program.sample_paths();
        let path_refs: Vec<&str> = sample_paths.to_vec();
        let config = CopyConfig {
            search_dir: search,
            output_dir,
            sample_paths: &path_refs,
        };
        let report = copy_samples(&config);
        println!("Samples: {}", report.summary());
    }

    Ok(())
}
