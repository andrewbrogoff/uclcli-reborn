
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_ucl_compress_decompress_roundtrip() {
    let input_data = "hello world".as_bytes();
    let ucl_process = Command::new("cargo")
        .args(["run", "--bin", "ucl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute ucl");

    ucl_process
        .stdin
        .as_ref()
        .unwrap()
        .write_all(input_data)
        .expect("failed to write to ucl stdin");

    let ucl_output = ucl_process
        .wait_with_output()
        .expect("failed to wait for ucl");
    assert!(ucl_output.status.success(), "ucl failed: stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&ucl_output.stdout),
        String::from_utf8_lossy(&ucl_output.stderr));
    let compressed_data = ucl_output.stdout;

    let unucl_process = Command::new("cargo")
        .args(["run", "--bin", "unucl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute unucl");

    unucl_process
        .stdin
        .as_ref()
        .unwrap()
        .write_all(&compressed_data)
        .expect("failed to write to unucl stdin");

    let unucl_output = unucl_process
        .wait_with_output()
        .expect("failed to wait for unucl");
    assert!(unucl_output.status.success(), "unucl failed: stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&unucl_output.stdout),
        String::from_utf8_lossy(&unucl_output.stderr));
    let decompressed_data = unucl_output.stdout;

    assert_eq!(input_data, decompressed_data.as_slice());
}

#[test]
fn test_ucl_compress_decompress_roundtrip_with_files() {
    use tempfile::NamedTempFile;
    use std::io::Write as _;

    let input_data = b"hello world";
    let mut input = NamedTempFile::new().expect("failed to create temp input");
    input.write_all(input_data).expect("failed to write to temp input");
    let input_path = input.path().to_str().unwrap().to_owned();

    let output = NamedTempFile::new().expect("failed to create temp output");
    let output_path = output.path().to_str().unwrap().to_owned();

    let ucl_out = Command::new("cargo")
        .args(["run", "--bin", "ucl", "--"])
        .arg("-i").arg(&input_path)
        .arg("-o").arg(&output_path)
        .output()
        .expect("failed to execute ucl");
    assert!(ucl_out.status.success(), "ucl failed: stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&ucl_out.stdout),
        String::from_utf8_lossy(&ucl_out.stderr));

    let decompressed = NamedTempFile::new().expect("failed to create temp decompressed output");
    let decompressed_path = decompressed.path().to_str().unwrap().to_owned();

    let unucl_out = Command::new("cargo")
        .args(["run", "--bin", "unucl", "--"])
        .arg("-i").arg(&output_path)
        .arg("-o").arg(&decompressed_path)
        .output()
        .expect("failed to execute unucl");
    assert!(unucl_out.status.success(), "unucl failed: stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&unucl_out.stdout),
        String::from_utf8_lossy(&unucl_out.stderr));

    let decompressed_data = fs::read(&decompressed_path).expect("failed to read decompressed output");
    assert_eq!(input_data, decompressed_data.as_slice());
}

#[test]
fn test_ucl_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ucl", "--", "--help"])
        .output()
        .expect("failed to execute ucl --help");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage"));
}

#[test]
fn test_unucl_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "unucl", "--", "--help"])
        .output()
        .expect("failed to execute unucl --help");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage"));
}

#[test]
fn test_ucl_version() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ucl", "--", "--version"])
        .output()
        .expect("failed to execute ucl --version");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("ucl 0.2"));
}

#[test]
fn test_unucl_version() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "unucl", "--", "--version"])
        .output()
        .expect("failed to execute unucl --version");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("unucl 0.2"));
}

#[test]
fn test_ucl_invalid_input_file() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ucl", "--", "-i", "nonexistent.txt"])
        .output()
        .expect("failed to execute ucl");
    assert!(!output.status.success());
}

#[test]
fn test_unucl_invalid_input_file() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "unucl", "--", "-i", "nonexistent.txt"])
        .output()
        .expect("failed to execute unucl");
    assert!(!output.status.success());
}

#[test]
fn test_ucl_stdin_stdout_large_input() {
    // Use a larger binary input to validate streaming behavior and differ from the basic roundtrip test
    let pattern = [0u8, 1, 2, 3];
    let mut input_data = Vec::with_capacity(64 * 1024);
    while input_data.len() < 64 * 1024 {
        input_data.extend_from_slice(&pattern);
    }

    let ucl_process = Command::new("cargo")
        .args(["run", "--bin", "ucl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute ucl");

    ucl_process
        .stdin
        .as_ref()
        .unwrap()
        .write_all(&input_data)
        .expect("failed to write to ucl stdin");

    let ucl_output = ucl_process
        .wait_with_output()
        .expect("failed to wait for ucl");
    assert!(ucl_output.status.success(), "ucl failed: stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&ucl_output.stdout),
        String::from_utf8_lossy(&ucl_output.stderr));
    let compressed_data = ucl_output.stdout;

    let unucl_process = Command::new("cargo")
        .args(["run", "--bin", "unucl"]) 
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute unucl");

    unucl_process
        .stdin
        .as_ref()
        .unwrap()
        .write_all(&compressed_data)
        .expect("failed to write to unucl stdin");

    let unucl_output = unucl_process
        .wait_with_output()
        .expect("failed to wait for unucl");
    assert!(unucl_output.status.success(), "unucl failed: stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&unucl_output.stdout),
        String::from_utf8_lossy(&unucl_output.stderr));
    let decompressed_data = unucl_output.stdout;

    assert_eq!(input_data.as_slice(), decompressed_data.as_slice());
}
