use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn cli_binary() -> PathBuf {
    // Use the binary built by `cargo test`, avoiding nested cargo invocations
    // that deadlock on the build directory lock.
    let path = PathBuf::from(env!("CARGO_BIN_EXE_rusty-samplers-cli"));
    assert!(path.exists(), "CLI binary not found at {path:?}");
    path
}

fn create_test_riff_file(temp_dir: &TempDir, content: &[u8]) -> PathBuf {
    let file_path = temp_dir.path().join("test.akp");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content).unwrap();
    file_path
}

#[test]
fn test_invalid_riff_header() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_riff_file(&temp_dir, b"INVALID_HEADER");

    let output = Command::new(cli_binary())
        .arg(&file_path)
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid file format: Expected RIFF header"));
    assert!(!output.status.success());
}

#[test]
fn test_invalid_aprg_signature() {
    let temp_dir = TempDir::new().unwrap();
    let mut content = Vec::new();
    content.extend_from_slice(b"RIFF");
    content.extend_from_slice(&[8, 0, 0, 0]);
    content.extend_from_slice(b"INVD");

    let file_path = create_test_riff_file(&temp_dir, &content);

    let output = Command::new(cli_binary())
        .arg(&file_path)
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Expected APRG signature but found different signature"));
    assert!(!output.status.success());
}

#[test]
fn test_missing_file() {
    let output = Command::new(cli_binary())
        .arg("/nonexistent/file.akp")
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("File '/nonexistent/file.akp' not found"));
    assert!(!output.status.success());
}

#[test]
fn test_no_arguments() {
    let output = Command::new(cli_binary())
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage:"));
    assert!(!output.status.success());
}

#[test]
fn test_valid_riff_but_empty_program() {
    let temp_dir = TempDir::new().unwrap();
    let mut content = Vec::new();
    content.extend_from_slice(b"RIFF");
    content.extend_from_slice(&[8, 0, 0, 0]);
    content.extend_from_slice(b"APRG");

    let file_path = create_test_riff_file(&temp_dir, &content);

    let output = Command::new(cli_binary())
        .arg(&file_path)
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Missing required 'keygroup' chunk"));
    assert!(!output.status.success());
}
