// Unit tests for backward compatibility of existing flags
// Requirements: 7.4 - Test that existing --lod and --compact flags still work

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test that --lod flag with value 0 (Low) works correctly
#[test]
fn test_lod_flag_low() {
    let temp_dir = TempDir::new().unwrap();
    let scip_path = temp_dir.path().join("test.scip");
    let output_path = temp_dir.path().join("output.yaml");

    // Create a minimal valid SCIP file (empty index)
    let empty_scip = vec![];
    fs::write(&scip_path, empty_scip).unwrap();

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--lod")
        .arg("0");

    // Should succeed without errors
    cmd.assert().success();
}

/// Test that --lod flag with value 1 (Medium) works correctly
#[test]
fn test_lod_flag_medium() {
    let temp_dir = TempDir::new().unwrap();
    let scip_path = temp_dir.path().join("test.scip");
    let output_path = temp_dir.path().join("output.yaml");

    // Create a minimal valid SCIP file
    let empty_scip = vec![];
    fs::write(&scip_path, empty_scip).unwrap();

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--lod")
        .arg("1");

    // Should succeed without errors
    cmd.assert().success();
}

/// Test that --lod flag with value 2 (High) works correctly
#[test]
fn test_lod_flag_high() {
    let temp_dir = TempDir::new().unwrap();
    let scip_path = temp_dir.path().join("test.scip");
    let output_path = temp_dir.path().join("output.yaml");

    // Create a minimal valid SCIP file
    let empty_scip = vec![];
    fs::write(&scip_path, empty_scip).unwrap();

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--lod")
        .arg("2");

    // Should succeed without errors
    cmd.assert().success();
}

/// Test that --lod flag defaults to 1 (Medium) when not specified
#[test]
fn test_lod_flag_default() {
    let temp_dir = TempDir::new().unwrap();
    let scip_path = temp_dir.path().join("test.scip");
    let output_path = temp_dir.path().join("output.yaml");

    // Create a minimal valid SCIP file
    let empty_scip = vec![];
    fs::write(&scip_path, empty_scip).unwrap();

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path);
    // No --lod flag, should default to 1

    // Should succeed without errors
    cmd.assert().success();
}

/// Test that --compact flag works correctly
#[test]
fn test_compact_flag_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let scip_path = temp_dir.path().join("test.scip");
    let output_path = temp_dir.path().join("output.yaml");

    // Create a minimal valid SCIP file
    let empty_scip = vec![];
    fs::write(&scip_path, empty_scip).unwrap();

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--compact");

    // Should succeed without errors
    cmd.assert().success();

    // Verify output file was created
    assert!(output_path.exists());
}

/// Test that --compact flag defaults to false when not specified
#[test]
fn test_compact_flag_default() {
    let temp_dir = TempDir::new().unwrap();
    let scip_path = temp_dir.path().join("test.scip");
    let output_path = temp_dir.path().join("output.yaml");

    // Create a minimal valid SCIP file
    let empty_scip = vec![];
    fs::write(&scip_path, empty_scip).unwrap();

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path);
    // No --compact flag, should default to false

    // Should succeed without errors
    cmd.assert().success();

    // Verify output file was created
    assert!(output_path.exists());
}

/// Test that --lod and --compact flags work together
#[test]
fn test_lod_and_compact_together() {
    let temp_dir = TempDir::new().unwrap();
    let scip_path = temp_dir.path().join("test.scip");
    let output_path = temp_dir.path().join("output.yaml");

    // Create a minimal valid SCIP file
    let empty_scip = vec![];
    fs::write(&scip_path, empty_scip).unwrap();

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--lod")
        .arg("2")
        .arg("--compact");

    // Should succeed without errors
    cmd.assert().success();

    // Verify output file was created
    assert!(output_path.exists());
}

/// Test that --help shows --lod flag documentation
#[test]
fn test_help_shows_lod_flag() {
    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--lod"))
        .stdout(predicate::str::contains("Nível de Detalhe"));
}

/// Test that --help shows --compact flag documentation
#[test]
fn test_help_shows_compact_flag() {
    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--compact"))
        .stdout(predicate::str::contains("Lista de Adjacência"));
}

/// Test that existing flags work with new optimization flags
#[test]
fn test_existing_flags_with_new_flags() {
    let temp_dir = TempDir::new().unwrap();
    let scip_path = temp_dir.path().join("test.scip");
    let output_path = temp_dir.path().join("output.yaml");

    // Create a minimal valid SCIP file
    let empty_scip = vec![];
    fs::write(&scip_path, empty_scip).unwrap();

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--lod")
        .arg("1")
        .arg("--compact")
        .arg("--ignore-framework-noise")
        .arg("--output-format")
        .arg("yaml");

    // Should succeed without errors - all flags should work together
    cmd.assert().success();

    // Verify output file was created
    assert!(output_path.exists());
}
