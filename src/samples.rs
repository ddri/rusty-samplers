use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for sample copying.
pub struct CopyConfig<'a> {
    /// Where to search for source WAV files (typically the AKP parent directory).
    pub search_dir: &'a Path,
    /// Where the output preset lives — samples are copied relative to this.
    pub output_dir: &'a Path,
    /// Sample paths referenced by the preset (from `AkaiProgram::sample_paths()`).
    pub sample_paths: &'a [&'a str],
}

/// Result of attempting to copy a single sample file.
#[derive(Debug, Clone)]
pub enum SampleResult {
    /// Exact-match source found and copied.
    Copied { source: PathBuf, dest: PathBuf },
    /// Source found via case-insensitive match and copied.
    CopiedCaseMismatch { source: PathBuf, dest: PathBuf },
    /// Destination already exists — skipped.
    AlreadyExists(PathBuf),
    /// Source file not found anywhere in search directory.
    Missing(String),
    /// Found but failed to copy.
    CopyError { path: String, error: String },
}

/// Summary of all sample copy operations for one conversion.
#[derive(Debug, Clone, Default)]
pub struct CopyReport {
    pub results: Vec<SampleResult>,
}

impl CopyReport {
    pub fn copied_count(&self) -> usize {
        self.results.iter().filter(|r| matches!(r, SampleResult::Copied { .. } | SampleResult::CopiedCaseMismatch { .. })).count()
    }

    pub fn case_mismatch_count(&self) -> usize {
        self.results.iter().filter(|r| matches!(r, SampleResult::CopiedCaseMismatch { .. })).count()
    }

    pub fn already_exists_count(&self) -> usize {
        self.results.iter().filter(|r| matches!(r, SampleResult::AlreadyExists(_))).count()
    }

    pub fn missing_count(&self) -> usize {
        self.results.iter().filter(|r| matches!(r, SampleResult::Missing(_))).count()
    }

    pub fn error_count(&self) -> usize {
        self.results.iter().filter(|r| matches!(r, SampleResult::CopyError { .. })).count()
    }

    /// One-line summary suitable for CLI output.
    pub fn summary(&self) -> String {
        let copied = self.copied_count();
        let mismatched = self.case_mismatch_count();
        let existing = self.already_exists_count();
        let missing = self.missing_count();
        let errors = self.error_count();

        let mut parts = vec![format!("{copied} copied")];
        if mismatched > 0 {
            parts.push(format!("{mismatched} case mismatch"));
        }
        if existing > 0 {
            parts.push(format!("{existing} already existed"));
        }
        if missing > 0 {
            parts.push(format!("{missing} missing"));
        }
        if errors > 0 {
            parts.push(format!("{errors} errors"));
        }
        parts.join(", ")
    }
}

/// Result of resolving a sample path in the filesystem.
enum ResolveResult {
    /// Found at exact path.
    Exact(PathBuf),
    /// Found via case-insensitive matching (actual path differs from requested).
    CaseMismatch(PathBuf),
    /// Not found.
    NotFound,
}

/// Resolve a sample path case-insensitively under the given root directory.
///
/// Walks each path component, checking for exact match first, then
/// falling back to case-insensitive directory listing.
fn resolve_sample_path(search_dir: &Path, sample_path: &str) -> ResolveResult {
    // Normalize separators: AKP uses backslashes, we need platform paths
    let normalized = sample_path.replace('\\', "/");
    let relative = Path::new(&normalized);

    let exact = search_dir.join(relative);
    if exact.exists() {
        return ResolveResult::Exact(exact);
    }

    // Walk component by component with case-insensitive fallback
    let mut current = search_dir.to_path_buf();
    let mut had_mismatch = false;

    for component in relative.components() {
        let target = match component {
            std::path::Component::Normal(s) => s.to_string_lossy(),
            _ => continue,
        };

        let exact_child = current.join(&*target);
        if exact_child.exists() {
            current = exact_child;
            continue;
        }

        // Case-insensitive search in current directory
        let target_lower = target.to_lowercase();
        let entries = match fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(_) => return ResolveResult::NotFound,
        };

        let mut found = false;
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().to_lowercase() == target_lower {
                current = entry.path();
                had_mismatch = true;
                found = true;
                break;
            }
        }

        if !found {
            return ResolveResult::NotFound;
        }
    }

    if current.is_file() {
        if had_mismatch {
            ResolveResult::CaseMismatch(current)
        } else {
            ResolveResult::Exact(current)
        }
    } else {
        ResolveResult::NotFound
    }
}

/// Copy all referenced sample files from `search_dir` to `output_dir`,
/// preserving relative subdirectory structure.
///
/// Missing samples are reported but do not cause failure.
pub fn copy_samples(config: &CopyConfig) -> CopyReport {
    let mut report = CopyReport::default();

    for &sample_path in config.sample_paths {
        // Build destination path (always use the original relative path from the preset)
        let normalized = sample_path.replace('\\', "/");

        // Append .wav if no recognized audio extension
        let dest_name = ensure_wav_extension(&normalized);
        let dest = config.output_dir.join(&dest_name);

        // Skip if destination already exists
        if dest.exists() {
            report.results.push(SampleResult::AlreadyExists(dest));
            continue;
        }

        // Try to find the source file (with .wav appended if needed)
        let result = resolve_sample_path(config.search_dir, &dest_name);

        match result {
            ResolveResult::Exact(source) => {
                match copy_with_dirs(&source, &dest) {
                    Ok(()) => report.results.push(SampleResult::Copied { source, dest }),
                    Err(e) => report.results.push(SampleResult::CopyError {
                        path: sample_path.to_string(),
                        error: e.to_string(),
                    }),
                }
            }
            ResolveResult::CaseMismatch(source) => {
                match copy_with_dirs(&source, &dest) {
                    Ok(()) => report.results.push(SampleResult::CopiedCaseMismatch { source, dest }),
                    Err(e) => report.results.push(SampleResult::CopyError {
                        path: sample_path.to_string(),
                        error: e.to_string(),
                    }),
                }
            }
            ResolveResult::NotFound => {
                report.results.push(SampleResult::Missing(sample_path.to_string()));
            }
        }
    }

    report
}

/// Append `.wav` if the path doesn't already have a recognized audio extension.
fn ensure_wav_extension(path: &str) -> String {
    let known_extensions = ["wav", "aif", "aiff"];
    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
        if known_extensions.iter().any(|&k| k.eq_ignore_ascii_case(ext)) {
            return path.to_string();
        }
    }
    format!("{path}.wav")
}

/// Copy a file, creating parent directories as needed.
fn copy_with_dirs(source: &Path, dest: &Path) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_file(dir: &Path, relative: &str, content: &[u8]) {
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_exact_match_copy() {
        let src = TempDir::new().unwrap();
        let out = TempDir::new().unwrap();
        create_file(src.path(), "Piano_C3.wav", b"RIFF_FAKE_WAV");

        let paths = vec!["Piano_C3"];
        let config = CopyConfig {
            search_dir: src.path(),
            output_dir: out.path(),
            sample_paths: &paths.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(),
        };
        let report = copy_samples(&config);

        assert_eq!(report.copied_count(), 1);
        assert_eq!(report.missing_count(), 0);
        assert!(out.path().join("Piano_C3.wav").exists());
    }

    #[test]
    fn test_exact_match_with_subdirectory() {
        let src = TempDir::new().unwrap();
        let out = TempDir::new().unwrap();
        create_file(src.path(), "Strings/Violin_C3.wav", b"RIFF_FAKE_WAV");

        let paths = vec!["Strings/Violin_C3"];
        let config = CopyConfig {
            search_dir: src.path(),
            output_dir: out.path(),
            sample_paths: &paths.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(),
        };
        let report = copy_samples(&config);

        assert_eq!(report.copied_count(), 1);
        assert!(out.path().join("Strings/Violin_C3.wav").exists());
    }

    #[test]
    fn test_case_mismatch_found() {
        let src = TempDir::new().unwrap();
        let out = TempDir::new().unwrap();
        // Source has different case than what the preset references
        create_file(src.path(), "strings/violin_c3.wav", b"RIFF_FAKE_WAV");

        // Detect case-insensitive filesystem (macOS default)
        let case_insensitive_fs = src.path().join("Strings/Violin_C3.wav").exists();

        let paths = vec!["Strings/Violin_C3"];
        let config = CopyConfig {
            search_dir: src.path(),
            output_dir: out.path(),
            sample_paths: &paths.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(),
        };
        let report = copy_samples(&config);

        assert_eq!(report.copied_count(), 1);
        if case_insensitive_fs {
            // On case-insensitive FS, the exact path check succeeds
            assert_eq!(report.case_mismatch_count(), 0);
        } else {
            assert_eq!(report.case_mismatch_count(), 1);
        }
        // Dest uses the original preset path
        assert!(out.path().join("Strings/Violin_C3.wav").exists());
    }

    #[test]
    fn test_missing_file_reported() {
        let src = TempDir::new().unwrap();
        let out = TempDir::new().unwrap();

        let paths = vec!["NonExistent_Sample"];
        let config = CopyConfig {
            search_dir: src.path(),
            output_dir: out.path(),
            sample_paths: &paths.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(),
        };
        let report = copy_samples(&config);

        assert_eq!(report.copied_count(), 0);
        assert_eq!(report.missing_count(), 1);
    }

    #[test]
    fn test_already_exists_skipped() {
        let src = TempDir::new().unwrap();
        let out = TempDir::new().unwrap();
        create_file(src.path(), "Piano_C3.wav", b"SOURCE_DATA");
        create_file(out.path(), "Piano_C3.wav", b"EXISTING_DATA");

        let paths = vec!["Piano_C3"];
        let config = CopyConfig {
            search_dir: src.path(),
            output_dir: out.path(),
            sample_paths: &paths.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(),
        };
        let report = copy_samples(&config);

        assert_eq!(report.already_exists_count(), 1);
        assert_eq!(report.copied_count(), 0);
        // Original data preserved
        assert_eq!(fs::read(out.path().join("Piano_C3.wav")).unwrap(), b"EXISTING_DATA");
    }

    #[test]
    fn test_sample_paths_unique() {
        use crate::types::{AkaiProgram, Keygroup, Zone};

        let program = AkaiProgram {
            keygroups: vec![
                Keygroup {
                    zones: vec![
                        Zone { sample_name: "Piano_C3".to_string(), ..Default::default() },
                        Zone { sample_name: "Piano_C3".to_string(), ..Default::default() },
                    ],
                    ..Default::default()
                },
                Keygroup {
                    zones: vec![
                        Zone { sample_name: "Piano_C4".to_string(), ..Default::default() },
                        Zone { sample_name: "Piano_C3".to_string(), ..Default::default() },
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let paths = program.sample_paths();
        assert_eq!(paths, vec!["Piano_C3", "Piano_C4"]);
    }

    #[test]
    fn test_backslash_path_normalized() {
        let src = TempDir::new().unwrap();
        let out = TempDir::new().unwrap();
        create_file(src.path(), "Strings/Violin_C3.wav", b"RIFF_FAKE_WAV");

        // AKP files use backslash paths
        let paths = vec!["Strings\\Violin_C3"];
        let config = CopyConfig {
            search_dir: src.path(),
            output_dir: out.path(),
            sample_paths: &paths.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(),
        };
        let report = copy_samples(&config);

        assert_eq!(report.copied_count(), 1);
        assert!(out.path().join("Strings/Violin_C3.wav").exists());
    }

    #[test]
    fn test_ensure_wav_extension() {
        assert_eq!(ensure_wav_extension("Piano_C3"), "Piano_C3.wav");
        assert_eq!(ensure_wav_extension("Piano_C3.wav"), "Piano_C3.wav");
        assert_eq!(ensure_wav_extension("Piano_C3.WAV"), "Piano_C3.WAV");
        assert_eq!(ensure_wav_extension("Piano_C3.aif"), "Piano_C3.aif");
        assert_eq!(ensure_wav_extension("BRASS 02-C.1"), "BRASS 02-C.1.wav");
    }

    #[test]
    fn test_summary_formatting() {
        let report = CopyReport {
            results: vec![
                SampleResult::Copied { source: PathBuf::from("a"), dest: PathBuf::from("b") },
                SampleResult::CopiedCaseMismatch { source: PathBuf::from("c"), dest: PathBuf::from("d") },
                SampleResult::Missing("e".to_string()),
            ],
        };
        assert_eq!(report.summary(), "2 copied, 1 case mismatch, 1 missing");
    }
}
