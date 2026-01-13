
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_ucl_compress_decompress_roundtrip() {
    let input_data = "hello world".as_bytes();
    let ucl_process = Command::new("cargo")
        .args(&["run", "--bin", "ucl"])
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
    let compressed_data = ucl_output.stdout;

    let unucl_process = Command::new("cargo")
        .args(&["run", "--bin", "unucl"])
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
    let decompressed_data = unucl_output.stdout;

    assert_eq!(input_data, decompressed_data.as_slice());
}

#[test]
fn test_ucl_compress_decompress_roundtrip_with_files() {
    let input_data = "hello world".as_bytes();
    fs::write("input.txt", input_data).expect("failed to write input file");

    Command::new("cargo")
        .args(&["run", "--bin", "ucl", "--", "-i", "input.txt", "-o", "output.bin"])
        .output()
        .expect("failed to execute ucl");

    Command::new("cargo")
        .args(&["run", "--bin", "unucl", "--", "-i", "output.bin", "-o", "output.txt"])
        .output()
        .expect("failed to execute unucl");

    let decompressed_data = fs::read("output.txt").expect("failed to read output file");

    assert_eq!(input_data, decompressed_data.as_slice());

    fs::remove_file("input.txt").expect("failed to remove input file");
    fs::remove_file("output.bin").expect("failed to remove output file");
    fs::remove_file("output.txt").expect("failed to remove output file");
}

#[test]
fn test_ucl_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ucl", "--", "--help"])
        .output()
        .expect("failed to execute ucl --help");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage"));
}

#[test]
fn test_unucl_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "unucl", "--", "--help"])
        .output()
        .expect("failed to execute unucl --help");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage"));
}

#[test]
fn test_ucl_version() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ucl", "--", "--version"])
        .output()
        .expect("failed to execute ucl --version");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("ucl 0.2"));
}

#[test]
fn test_unucl_version() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "unucl", "--", "--version"])
        .output()
        .expect("failed to execute unucl --version");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("unucl 0.2"));
}

#[test]
fn test_ucl_invalid_input_file() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ucl", "--", "-i", "nonexistent.txt"])
        .output()
        .expect("failed to execute ucl");
    assert!(!output.status.success());
}

#[test]
fn test_unucl_invalid_input_file() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "unucl", "--", "-i", "nonexistent.txt"])
        .output()
        .expect("failed to execute unucl");
    assert!(!output.status.success());
}

#[test]
fn test_ucl_stdin_stdout() {
    let input_data = "hello world".as_bytes();
    let ucl_process = Command::new("cargo")
        .args(&["run", "--bin", "ucl"])
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
    let compressed_data = ucl_output.stdout;

    let unucl_process = Command::new("cargo")
        .args(&["run", "--bin", "unucl"])
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
    let decompressed_data = unucl_output.stdout;

    assert_eq!(input_data, decompressed_data.as_slice());
}
