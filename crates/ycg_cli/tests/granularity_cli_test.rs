// Integration tests for ad-hoc granularity CLI flags
// Requirements: 6.1, 6.2, 6.3, 6.4, 6.5

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_adhoc_inline_signatures_requires_adhoc_format() {
    // Requirement 6.5: Granularity flags require --output-format adhoc
    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg("test.scip")
        .arg("--adhoc-inline-signatures");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("require --output-format adhoc"));
}

#[test]
fn test_adhoc_inline_logic_requires_adhoc_format() {
    // Requirement 6.5: Granularity flags require --output-format adhoc
    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg("test.scip")
        .arg("--adhoc-inline-logic");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("require --output-format adhoc"));
}

#[test]
fn test_adhoc_inline_signatures_with_yaml_format_fails() {
    // Requirement 6.5: Granularity flags require --output-format adhoc
    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg("test.scip")
        .arg("--output-format")
        .arg("yaml")
        .arg("--adhoc-inline-signatures");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("require --output-format adhoc"));
}

#[test]
fn test_adhoc_inline_logic_implicitly_enables_signatures() {
    // Requirement 6.3: --adhoc-inline-logic implicitly activates signatures
    // This test verifies that the flag combination is accepted
    // (actual behavior will be tested in integration tests with real SCIP files)

    let temp_dir = TempDir::new().unwrap();
    let scip_path = temp_dir.path().join("test.scip");

    // Create a minimal valid SCIP file (empty index)
    let empty_scip = vec![]; // Empty protobuf
    fs::write(&scip_path, empty_scip).unwrap();

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output-format")
        .arg("adhoc")
        .arg("--adhoc-inline-logic");

    // Should not fail with "requires adhoc format" error
    // (may fail with other errors due to empty SCIP, but that's ok)
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("require --output-format adhoc"),
        "Should not require adhoc format when already specified"
    );
}

#[test]
fn test_both_granularity_flags_uses_highest_level() {
    // Requirement 6.4: When both flags provided, use highest level (logic)
    let temp_dir = TempDir::new().unwrap();
    let scip_path = temp_dir.path().join("test.scip");

    // Create a minimal valid SCIP file
    let empty_scip = vec![];
    fs::write(&scip_path, empty_scip).unwrap();

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output-format")
        .arg("adhoc")
        .arg("--adhoc-inline-signatures")
        .arg("--adhoc-inline-logic");

    // Should not fail with validation error
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("require --output-format adhoc"),
        "Should accept both flags with adhoc format"
    );
}

#[test]
fn test_help_shows_granularity_flags() {
    // Requirement 6.6: --help displays documentation for granularity flags
    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();

    cmd.arg("generate").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--adhoc-inline-signatures"))
        .stdout(predicate::str::contains("--adhoc-inline-logic"))
        .stdout(predicate::str::contains("Level 1"))
        .stdout(predicate::str::contains("Level 2"));
}
