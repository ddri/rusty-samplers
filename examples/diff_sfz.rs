//! SFZ diff tool — compares our converter output against reference SFZ files.
//!
//! Reference files are manually exported from ConvertWithMoss and stored in
//! tools/reference_sfz/. This tool parses both files into opcode-value maps
//! per region, matches regions by sample path, and compares with tolerances.
//!
//! Usage:
//!   cargo run --example diff_sfz -- <our_file.sfz> <reference_file.sfz>
//!   cargo run --example diff_sfz -- --dir tools/reference_sfz/ output_dir/
//!
//! Tolerances:
//!   - Envelope times: within 20%
//!   - Filter cutoff: within 10%
//!   - Volume: within 1 dB
//!   - Key/velocity ranges: exact
//!   - Tuning: exact

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage:");
        eprintln!("  {} <our.sfz> <reference.sfz>", args[0]);
        eprintln!("  {} --dir <reference_dir> <our_dir>", args[0]);
        std::process::exit(1);
    }

    if args[1] == "--dir" {
        if args.len() < 4 {
            eprintln!("--dir requires two directory arguments");
            std::process::exit(1);
        }
        diff_directories(&args[2], &args[3]);
    } else {
        let diffs = diff_files(Path::new(&args[1]), Path::new(&args[2]));
        print_diffs(&diffs, &args[1]);
        if diffs.iter().any(|d| d.severity == Severity::Error) {
            std::process::exit(1);
        }
    }
}

fn diff_directories(ref_dir: &str, our_dir: &str) {
    let ref_path = Path::new(ref_dir);
    let our_path = Path::new(our_dir);

    if !ref_path.is_dir() || !our_path.is_dir() {
        eprintln!("Both arguments must be directories");
        std::process::exit(1);
    }

    let ref_files = collect_sfz_files(ref_path);
    if ref_files.is_empty() {
        eprintln!("No .sfz files found in {ref_dir}");
        std::process::exit(1);
    }

    let mut total_diffs = 0u32;
    let mut files_compared = 0u32;
    let mut files_missing = 0u32;

    for ref_file in &ref_files {
        let stem = ref_file.file_stem().unwrap().to_string_lossy();
        let our_file = our_path.join(format!("{stem}.sfz"));

        if !our_file.exists() {
            files_missing += 1;
            println!("MISSING: {stem}.sfz (no matching output file)");
            continue;
        }

        files_compared += 1;
        let diffs = diff_files(&our_file, ref_file);
        if !diffs.is_empty() {
            print_diffs(&diffs, &stem);
            total_diffs += diffs.len() as u32;
        }
    }

    println!("\n=== Summary ===");
    println!("Reference files:  {}", ref_files.len());
    println!("Compared:         {files_compared}");
    println!("Missing:          {files_missing}");
    println!("Differences:      {total_diffs}");
}

fn collect_sfz_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("sfz")) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug)]
struct Diff {
    region: String,
    opcode: String,
    ours: String,
    reference: String,
    severity: Severity,
}

/// A parsed SFZ region: map of opcode → value.
type OpcodeMap = HashMap<String, String>;

fn diff_files(our_file: &Path, ref_file: &Path) -> Vec<Diff> {
    let our_content = match fs::read_to_string(our_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {e}", our_file.display());
            return Vec::new();
        }
    };
    let ref_content = match fs::read_to_string(ref_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {e}", ref_file.display());
            return Vec::new();
        }
    };

    let our_regions = parse_sfz_regions(&our_content);
    let ref_regions = parse_sfz_regions(&ref_content);

    let mut diffs = Vec::new();

    // Match regions by sample path (case-insensitive)
    for (ref_sample, ref_opcodes) in &ref_regions {
        let ref_sample_lower = ref_sample.to_ascii_lowercase();
        let our_opcodes = our_regions.iter()
            .find(|(k, _)| k.to_ascii_lowercase() == ref_sample_lower)
            .map(|(_, v)| v);

        match our_opcodes {
            None => {
                diffs.push(Diff {
                    region: ref_sample.clone(),
                    opcode: "*".into(),
                    ours: "(missing region)".into(),
                    reference: "(present)".into(),
                    severity: Severity::Error,
                });
            }
            Some(ours) => {
                compare_opcodes(ref_sample, ours, ref_opcodes, &mut diffs);
            }
        }
    }

    diffs
}

fn parse_sfz_regions(content: &str) -> Vec<(String, OpcodeMap)> {
    let mut regions = Vec::new();
    let mut current: Option<OpcodeMap> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("//") || line.is_empty() {
            continue;
        }

        if line == "<region>" {
            if let Some(map) = current.take() {
                let sample = map.get("sample").cloned().unwrap_or_default();
                regions.push((sample, map));
            }
            current = Some(HashMap::new());
            continue;
        }

        if line.starts_with('<') {
            if let Some(map) = current.take() {
                let sample = map.get("sample").cloned().unwrap_or_default();
                regions.push((sample, map));
            }
            continue;
        }

        if let Some(ref mut map) = current {
            if let Some((opcode, value)) = line.split_once('=') {
                map.insert(opcode.trim().to_string(), value.trim().to_string());
            }
        }
    }

    if let Some(map) = current {
        let sample = map.get("sample").cloned().unwrap_or_default();
        regions.push((sample, map));
    }

    regions
}

fn compare_opcodes(region: &str, ours: &OpcodeMap, reference: &OpcodeMap, diffs: &mut Vec<Diff>) {
    // Opcodes to compare, with their tolerance type
    let exact_opcodes = ["lokey", "hikey", "lovel", "hivel", "transpose", "tune"];
    let pct_20_opcodes = ["ampeg_attack", "ampeg_decay", "ampeg_release", "fileg_attack", "fileg_decay", "fileg_release"];
    let pct_10_opcodes = ["cutoff"];
    let abs_1_opcodes = ["volume"];

    for opcode in &exact_opcodes {
        if let Some(ref_val) = reference.get(*opcode) {
            let our_val = ours.get(*opcode).map(|s| s.as_str()).unwrap_or("(missing)");
            if our_val != ref_val {
                diffs.push(Diff {
                    region: region.into(),
                    opcode: opcode.to_string(),
                    ours: our_val.into(),
                    reference: ref_val.clone(),
                    severity: Severity::Error,
                });
            }
        }
    }

    for opcode in &pct_20_opcodes {
        compare_float_pct(region, opcode, ours, reference, 0.20, diffs);
    }

    for opcode in &pct_10_opcodes {
        compare_float_pct(region, opcode, ours, reference, 0.10, diffs);
    }

    for opcode in &abs_1_opcodes {
        compare_float_abs(region, opcode, ours, reference, 1.0, diffs);
    }
}

fn compare_float_pct(region: &str, opcode: &str, ours: &OpcodeMap, reference: &OpcodeMap, tolerance: f64, diffs: &mut Vec<Diff>) {
    let ref_val = match reference.get(opcode).and_then(|v| v.parse::<f64>().ok()) {
        Some(v) => v,
        None => return,
    };
    let our_val = match ours.get(opcode).and_then(|v| v.parse::<f64>().ok()) {
        Some(v) => v,
        None => {
            diffs.push(Diff {
                region: region.into(),
                opcode: opcode.into(),
                ours: "(missing)".into(),
                reference: format!("{ref_val}"),
                severity: Severity::Warning,
            });
            return;
        }
    };

    let diff = if ref_val.abs() < 0.001 {
        (our_val - ref_val).abs()
    } else {
        ((our_val - ref_val) / ref_val).abs()
    };

    if diff > tolerance {
        diffs.push(Diff {
            region: region.into(),
            opcode: opcode.into(),
            ours: format!("{our_val}"),
            reference: format!("{ref_val}"),
            severity: Severity::Warning,
        });
    }
}

fn compare_float_abs(region: &str, opcode: &str, ours: &OpcodeMap, reference: &OpcodeMap, tolerance: f64, diffs: &mut Vec<Diff>) {
    let ref_val = match reference.get(opcode).and_then(|v| v.parse::<f64>().ok()) {
        Some(v) => v,
        None => return,
    };
    let our_val = match ours.get(opcode).and_then(|v| v.parse::<f64>().ok()) {
        Some(v) => v,
        None => {
            diffs.push(Diff {
                region: region.into(),
                opcode: opcode.into(),
                ours: "(missing)".into(),
                reference: format!("{ref_val}"),
                severity: Severity::Warning,
            });
            return;
        }
    };

    if (our_val - ref_val).abs() > tolerance {
        diffs.push(Diff {
            region: region.into(),
            opcode: opcode.into(),
            ours: format!("{our_val}"),
            reference: format!("{ref_val}"),
            severity: Severity::Warning,
        });
    }
}

fn print_diffs(diffs: &[Diff], label: &str) {
    if diffs.is_empty() {
        return;
    }
    println!("\n--- {label} ---");
    for diff in diffs {
        let severity = match diff.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN ",
            Severity::Info => "INFO ",
        };
        println!("  [{severity}] {}/{}: ours={} ref={}", diff.region, diff.opcode, diff.ours, diff.reference);
    }
}
