//! Sample path validator for converted presets.
//!
//! Walks a directory of converted .dspreset and .sfz files, extracts every sample
//! reference, and checks if the referenced file exists on disk. Reports found,
//! case-mismatch, and missing samples.
//!
//! Usage:
//!   cargo run --example validate_samples [directory]
//!
//! Defaults to test_akp_files/ if no directory given.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let dir = env::args().nth(1).unwrap_or_else(|| "test_akp_files".to_string());
    let dir = Path::new(&dir);

    if !dir.is_dir() {
        eprintln!("Error: '{}' is not a directory", dir.display());
        std::process::exit(1);
    }

    let mut presets_scanned = 0u32;
    let mut samples_total = 0u32;
    let mut found = 0u32;
    let mut case_mismatch = 0u32;
    let mut missing = 0u32;
    let mut missing_details: Vec<(String, String)> = Vec::new();
    let mut case_mismatch_details: Vec<(String, String, String)> = Vec::new();

    // Walk all preset files
    let preset_files = collect_preset_files(dir);

    for preset_path in &preset_files {
        presets_scanned += 1;
        let content = match fs::read_to_string(preset_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: couldn't read {}: {e}", preset_path.display());
                continue;
            }
        };

        let preset_dir = preset_path.parent().unwrap_or(dir);
        let sample_refs = extract_sample_refs(&content, preset_path);

        for sample_ref in &sample_refs {
            samples_total += 1;
            let resolved = preset_dir.join(sample_ref);

            match check_file_exists(&resolved) {
                FileStatus::Found => found += 1,
                FileStatus::CaseMismatch(actual) => {
                    case_mismatch += 1;
                    case_mismatch_details.push((
                        preset_path.display().to_string(),
                        sample_ref.clone(),
                        actual,
                    ));
                }
                FileStatus::Missing => {
                    missing += 1;
                    missing_details.push((
                        preset_path.display().to_string(),
                        sample_ref.clone(),
                    ));
                }
            }
        }
    }

    // Report
    println!("\n=== Sample Path Validation ===");
    println!("Directory:        {}", dir.display());
    println!("Presets scanned:  {presets_scanned}");
    println!("Samples total:    {samples_total}");
    println!("  Found:          {found}");
    println!("  Case mismatch:  {case_mismatch}  (would fail on Linux)");
    println!("  Missing:        {missing}");

    if !case_mismatch_details.is_empty() {
        println!("\n--- Case Mismatches ---");
        for (preset, reference, actual) in &case_mismatch_details {
            println!("{preset}:");
            println!("  REF:    \"{reference}\"");
            println!("  ACTUAL: \"{actual}\"");
        }
    }

    if !missing_details.is_empty() {
        println!("\n--- Missing ---");
        let mut current_preset = String::new();
        for (preset, reference) in &missing_details {
            if *preset != current_preset {
                println!("{preset}:");
                current_preset = preset.clone();
            }
            println!("  MISSING: \"{reference}\"");
        }
    }

    if missing > 0 {
        std::process::exit(1);
    }
}

/// Recursively collect all .dspreset and .sfz files.
fn collect_preset_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_preset_files(&path));
            } else if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_ascii_lowercase();
                if ext == "dspreset" || ext == "sfz" {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    files
}

/// Extract sample references from preset content.
fn extract_sample_refs(content: &str, path: &Path) -> Vec<String> {
    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "dspreset" => extract_dspreset_paths(content),
        "sfz" => extract_sfz_samples(content),
        _ => Vec::new(),
    }
}

/// Extract path="..." from dspreset XML.
fn extract_dspreset_paths(content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let pattern = "path=\"";
    let mut search: &str = content;
    while let Some(start) = search.find(pattern) {
        let after = &search[start + pattern.len()..];
        if let Some(end) = after.find('"') {
            let path = &after[..end];
            // Skip parameter references like $FILTER_CUTOFF
            if !path.starts_with('$') && !path.is_empty() {
                paths.push(path.to_string());
            }
            search = &after[end + 1..];
        } else {
            break;
        }
    }
    paths
}

/// Extract sample= values from SFZ.
fn extract_sfz_samples(content: &str) -> Vec<String> {
    let mut samples = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("sample=") {
            let value = value.trim();
            if !value.is_empty() {
                samples.push(value.to_string());
            }
        }
    }
    samples
}

enum FileStatus {
    Found,
    CaseMismatch(String),
    Missing,
}

/// Check if a file exists, with case-insensitive fallback.
fn check_file_exists(path: &Path) -> FileStatus {
    // Exact match first
    if path.exists() {
        return FileStatus::Found;
    }

    // Case-insensitive: check parent directory listing
    let parent = match path.parent() {
        Some(p) if p.is_dir() => p,
        _ => return FileStatus::Missing,
    };

    let target_name = match path.file_name() {
        Some(n) => n.to_string_lossy().to_ascii_lowercase(),
        None => return FileStatus::Missing,
    };

    // Build case-insensitive lookup of directory
    let dir_map = build_dir_map(parent);

    if let Some(actual_name) = dir_map.get(&target_name) {
        FileStatus::CaseMismatch(actual_name.clone())
    } else {
        FileStatus::Missing
    }
}

/// Build a lowercase → actual-name map for a directory's entries.
fn build_dir_map(dir: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                map.insert(name.to_ascii_lowercase(), name.to_string());
            }
        }
    }
    map
}
