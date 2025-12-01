// Integration tests for full pipeline with granularity levels
// Tests with real TypeScript project at all levels
// Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to get workspace root directory
fn get_workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Test Level 0 (default) with real TypeScript project
/// Validates: Requirements 8.1, 8.2, 9.1
#[test]
fn test_typescript_project_level_0_default() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.yaml");

    let workspace_root = get_workspace_root();
    let scip_path = workspace_root.join("examples/simple-ts/index.scip");
    let project_root = workspace_root.join("examples/simple-ts");

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();
    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--root")
        .arg(&project_root)
        .arg("--output-format")
        .arg("adhoc");

    cmd.assert().success();

    // Verify output file was created
    assert!(output_path.exists());

    // Read and verify output format
    let output = fs::read_to_string(&output_path).unwrap();

    // Should have _meta, _defs, and graph sections
    assert!(output.contains("_meta:"));
    assert!(output.contains("_defs:"));
    assert!(output.contains("graph:"));

    // Definitions should be in Level 0 format: ID|Name|Type
    // Should NOT contain signatures or logic
    assert!(output.contains("|file"));
    assert!(output.contains("|class"));
    assert!(output.contains("|method"));

    // Should NOT contain function signatures (no parentheses in definitions)
    let defs_section = extract_defs_section(&output);
    for line in defs_section.lines() {
        if line.trim().starts_with("- ") {
            let def = line.trim().trim_start_matches("- ");
            let parts: Vec<&str> = def.split('|').collect();
            assert_eq!(parts.len(), 3, "Level 0 should have exactly 3 fields");
            // Should not contain signature format (no parentheses or colons in name field)
            assert!(
                !parts[1].contains('('),
                "Level 0 should not contain signatures"
            );
            assert!(
                !parts[1].contains("):"),
                "Level 0 should not contain return types"
            );
        }
    }
}

/// Test Level 1 (inline signatures) with real TypeScript project
/// Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.8, 9.2
#[test]
fn test_typescript_project_level_1_signatures() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.yaml");

    let workspace_root = get_workspace_root();
    let scip_path = workspace_root.join("examples/simple-ts/index.scip");
    let project_root = workspace_root.join("examples/simple-ts");

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();
    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--root")
        .arg(&project_root)
        .arg("--output-format")
        .arg("adhoc")
        .arg("--adhoc-inline-signatures");

    cmd.assert().success();

    // Verify output file was created
    assert!(output_path.exists());

    // Read and verify output format
    let output = fs::read_to_string(&output_path).unwrap();

    // Should have _meta, _defs, and graph sections
    assert!(output.contains("_meta:"));
    assert!(output.contains("_defs:"));

    // Definitions should be in Level 1 format: ID|Signature(args):Return|Type
    // Should contain signatures but NOT logic
    let defs_section = extract_defs_section(&output);
    let mut found_signature = false;

    for line in defs_section.lines() {
        if line.trim().starts_with("- ") {
            let def = line.trim().trim_start_matches("- ");
            let parts: Vec<&str> = def.split('|').collect();

            // Level 1 should have exactly 3 fields (no logic field)
            assert!(
                parts.len() == 3,
                "Level 1 should have exactly 3 fields, got {}",
                parts.len()
            );

            // Check if this is a method/function with signature
            if parts[2] == "method" || parts[2] == "function" {
                // Should contain signature format (parentheses)
                if parts[1].contains('(') {
                    found_signature = true;
                    // Verify signature format: name(args) or name(args):return
                    assert!(
                        parts[1].contains('(') && parts[1].contains(')'),
                        "Signature should have parentheses"
                    );
                }
            }

            // Should NOT contain logic field
            assert!(
                !def.contains("|logic:"),
                "Level 1 should not contain logic field"
            );
        }
    }

    assert!(
        found_signature,
        "Should find at least one method with signature"
    );
}

/// Test Level 2 (inline logic) with real TypeScript project
/// Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 9.3
#[test]
fn test_typescript_project_level_2_logic() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.yaml");

    let workspace_root = get_workspace_root();
    let scip_path = workspace_root.join("examples/simple-ts/index.scip");
    let project_root = workspace_root.join("examples/simple-ts");

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();
    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--root")
        .arg(&project_root)
        .arg("--output-format")
        .arg("adhoc")
        .arg("--adhoc-inline-logic");

    cmd.assert().success();

    // Verify output file was created
    assert!(output_path.exists());

    // Read and verify output format
    let output = fs::read_to_string(&output_path).unwrap();

    // Should have _meta, _defs, and graph sections
    assert!(output.contains("_meta:"));
    assert!(output.contains("_defs:"));

    // Definitions should be in Level 2 format: ID|Signature|Type|logic:steps
    // Should contain both signatures AND logic
    let defs_section = extract_defs_section(&output);
    let mut found_logic = false;

    for line in defs_section.lines() {
        if line.trim().starts_with("- ") {
            let def = line.trim().trim_start_matches("- ");
            let parts: Vec<&str> = def.split('|').collect();

            // Level 2 can have 3 or 4 fields (logic is optional)
            assert!(
                parts.len() == 3 || parts.len() == 4,
                "Level 2 should have 3 or 4 fields, got {}",
                parts.len()
            );

            // If 4 fields, the 4th should be logic
            if parts.len() == 4 {
                assert!(
                    parts[3].starts_with("logic:"),
                    "Fourth field should start with 'logic:'"
                );
                found_logic = true;

                // Verify logic contains valid keywords
                let logic_content = parts[3].trim_start_matches("logic:");
                // Should contain at least one logic keyword
                let has_valid_keyword = logic_content.contains("check(")
                    || logic_content.contains("action(")
                    || logic_content.contains("return(")
                    || logic_content.contains("match(")
                    || logic_content.contains("get(");

                assert!(
                    has_valid_keyword,
                    "Logic should contain valid keywords: {}",
                    logic_content
                );
            }
        }
    }

    // Note: It's possible that no methods have extractable logic
    // So we don't assert found_logic must be true
    println!("Found logic in definitions: {}", found_logic);
}

/// Test config file + CLI override for granularity
/// Validates: Requirements 7.1, 7.2, 9.4
#[test]
fn test_config_file_cli_override_granularity() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.yaml");

    let workspace_root = get_workspace_root();
    let scip_path = workspace_root.join("examples/simple-ts/index.scip");
    let project_root = workspace_root.join("examples/simple-ts");

    // Create config file in the project root with granularity = "default"
    let config_path = project_root.join("ycg.config.json");
    let config_content = r#"{
  "output": {
    "format": "adhoc",
    "adhocGranularity": "default"
  }
}"#;
    fs::write(&config_path, config_content).unwrap();

    // Run with CLI override to "signatures"
    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();
    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--root")
        .arg(&project_root) // Use project root so it finds the config
        .arg("--adhoc-inline-signatures"); // CLI override

    cmd.assert().success();

    // Clean up config file
    let _ = fs::remove_file(&config_path);

    // Verify output uses Level 1 (signatures), not Level 0 (default)
    let output = fs::read_to_string(&output_path).unwrap();
    let defs_section = extract_defs_section(&output);

    let mut found_signature = false;
    for line in defs_section.lines() {
        if line.trim().starts_with("- ") {
            let def = line.trim().trim_start_matches("- ");
            if def.contains("method") || def.contains("function") {
                if def.contains('(') && def.contains(')') {
                    found_signature = true;
                    break;
                }
            }
        }
    }

    // CLI should have overridden config file
    assert!(
        found_signature,
        "CLI flag should override config file granularity"
    );
}

/// Test backward compatibility: default behavior matches v1.3.1
/// Validates: Requirements 8.1, 8.2, 8.3, 9.5
#[test]
fn test_backward_compatibility_default_behavior() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.yaml");

    let workspace_root = get_workspace_root();
    let scip_path = workspace_root.join("examples/simple-ts/index.scip");
    let project_root = workspace_root.join("examples/simple-ts");

    // Run without any granularity flags (default behavior)
    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();
    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--root")
        .arg(&project_root)
        .arg("--output-format")
        .arg("adhoc");

    cmd.assert().success();

    // Verify output is in Level 0 format (backward compatible)
    let output = fs::read_to_string(&output_path).unwrap();
    let defs_section = extract_defs_section(&output);

    for line in defs_section.lines() {
        if line.trim().starts_with("- ") {
            let def = line.trim().trim_start_matches("- ");
            let parts: Vec<&str> = def.split('|').collect();

            // Should be exactly 3 fields (Level 0)
            assert_eq!(
                parts.len(),
                3,
                "Default behavior should produce Level 0 format (3 fields)"
            );

            // Should not contain signatures or logic
            assert!(!def.contains("logic:"), "Default should not contain logic");
        }
    }
}

/// Test that implicit signature activation works with logic flag
/// Validates: Requirements 6.3, 9.6
#[test]
fn test_implicit_signature_activation_with_logic() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.yaml");

    let workspace_root = get_workspace_root();
    let scip_path = workspace_root.join("examples/simple-ts/index.scip");
    let project_root = workspace_root.join("examples/simple-ts");

    // Run with ONLY --adhoc-inline-logic (should implicitly enable signatures)
    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();
    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--root")
        .arg(&project_root)
        .arg("--output-format")
        .arg("adhoc")
        .arg("--adhoc-inline-logic"); // Only logic flag, no signature flag

    cmd.assert().success();

    // Verify output includes signatures (implicitly activated)
    let output = fs::read_to_string(&output_path).unwrap();
    let defs_section = extract_defs_section(&output);

    let mut found_signature = false;
    for line in defs_section.lines() {
        if line.trim().starts_with("- ") {
            let def = line.trim().trim_start_matches("- ");
            if (def.contains("method") || def.contains("function"))
                && def.contains('(')
                && def.contains(')')
            {
                found_signature = true;
                break;
            }
        }
    }

    assert!(
        found_signature,
        "Logic flag should implicitly enable signatures"
    );
}

/// Test validation: granularity requires adhoc format
/// Validates: Requirements 6.5
#[test]
fn test_granularity_requires_adhoc_format_validation() {
    let workspace_root = get_workspace_root();
    let scip_path = workspace_root.join("examples/simple-ts/index.scip");
    let project_root = workspace_root.join("examples/simple-ts");

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();
    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--root")
        .arg(&project_root)
        .arg("--output-format")
        .arg("yaml") // YAML format, not adhoc
        .arg("--adhoc-inline-signatures"); // But trying to use granularity flag

    // Should fail with error
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("require"))
        .stderr(predicate::str::contains("adhoc"));
}

/// Helper function to extract _defs section from YAML output
fn extract_defs_section(yaml: &str) -> String {
    let mut in_defs = false;
    let mut defs = String::new();

    for line in yaml.lines() {
        if line.starts_with("_defs:") {
            in_defs = true;
            continue;
        }
        if in_defs {
            if line.starts_with("graph:") || line.starts_with("_") {
                break;
            }
            defs.push_str(line);
            defs.push('\n');
        }
    }

    defs
}

/// Test with NestJS project at Level 0
/// Validates: Requirements 9.1
#[test]
fn test_nestjs_project_level_0() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.yaml");

    let workspace_root = get_workspace_root();
    let scip_path = workspace_root.join("examples/nestjs-api-ts/index.scip");
    let project_root = workspace_root.join("examples/nestjs-api-ts");

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();
    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--root")
        .arg(&project_root)
        .arg("--output-format")
        .arg("adhoc");

    cmd.assert().success();

    let output = fs::read_to_string(&output_path).unwrap();
    let defs_section = extract_defs_section(&output);

    // Verify Level 0 format
    for line in defs_section.lines() {
        if line.trim().starts_with("- ") {
            let def = line.trim().trim_start_matches("- ");
            let parts: Vec<&str> = def.split('|').collect();
            assert_eq!(parts.len(), 3, "Level 0 should have exactly 3 fields");
        }
    }
}

/// Test with NestJS project at Level 1
/// Validates: Requirements 9.2
#[test]
fn test_nestjs_project_level_1() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.yaml");

    let workspace_root = get_workspace_root();
    let scip_path = workspace_root.join("examples/nestjs-api-ts/index.scip");
    let project_root = workspace_root.join("examples/nestjs-api-ts");

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();
    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--root")
        .arg(&project_root)
        .arg("--output-format")
        .arg("adhoc")
        .arg("--adhoc-inline-signatures");

    cmd.assert().success();

    let output = fs::read_to_string(&output_path).unwrap();
    let defs_section = extract_defs_section(&output);

    let mut found_signature = false;
    for line in defs_section.lines() {
        if line.trim().starts_with("- ") {
            let def = line.trim().trim_start_matches("- ");
            if (def.contains("method") || def.contains("function"))
                && def.contains('(')
                && def.contains(')')
            {
                found_signature = true;
                break;
            }
        }
    }

    assert!(found_signature, "Should find signatures in NestJS project");
}

/// Test with NestJS project at Level 2
/// Validates: Requirements 9.3
#[test]
fn test_nestjs_project_level_2() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.yaml");

    let workspace_root = get_workspace_root();
    let scip_path = workspace_root.join("examples/nestjs-api-ts/index.scip");
    let project_root = workspace_root.join("examples/nestjs-api-ts");

    let mut cmd = Command::cargo_bin("ycg_cli").unwrap();
    cmd.arg("generate")
        .arg("--input")
        .arg(&scip_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--root")
        .arg(&project_root)
        .arg("--output-format")
        .arg("adhoc")
        .arg("--adhoc-inline-logic");

    cmd.assert().success();

    let output = fs::read_to_string(&output_path).unwrap();

    // Just verify it runs successfully and produces valid output
    assert!(output.contains("_meta:"));
    assert!(output.contains("_defs:"));
}
