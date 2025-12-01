// Integration tests for granularity backward compatibility
// Validates Requirements 8.1, 8.2, 8.3
//
// These tests ensure that:
// - Default behavior (no granularity flags) produces Level 0 output
// - Level 0 output matches the v1.3.1 baseline
// - Existing ad-hoc format continues to work without modification

mod baseline_helpers;

use anyhow::Result;
use std::path::PathBuf;
use ycg_core::model::{AdHocGranularity, FileFilterConfig, OutputFormat};
use ycg_core::{LevelOfDetail, YcgConfig, run_scip_conversion};

/// Test that default ad-hoc format (Level 0) matches baseline
///
/// **Validates: Requirements 8.1, 8.2**
#[test]
fn test_adhoc_default_matches_baseline_simple_ts() -> Result<()> {
    // Skip if baseline doesn't exist
    if !baseline_helpers::baseline_exists("simple_ts_high_adhoc_default") {
        println!(
            "⚠ Skipping: Baseline not found. Generate with: cargo test --test baseline_generator -- --ignored"
        );
        return Ok(());
    }

    // Load baseline (Level 0 output)
    let baseline = baseline_helpers::load_baseline("simple_ts_high_adhoc_default")?;

    // Generate current output with default granularity
    let scip_path = PathBuf::from("../../../examples/simple-ts/index.scip");
    let project_root = PathBuf::from("../../../examples/simple-ts");

    let config = YcgConfig {
        lod: LevelOfDetail::High,
        project_root,
        compact: false,
        output_format: OutputFormat::AdHoc,
        ignore_framework_noise: false,
        file_filter: FileFilterConfig {
            include_patterns: vec![],
            exclude_patterns: vec![],
            use_gitignore: false,
        },
        adhoc_granularity: AdHocGranularity::Default, // Explicit Level 0
    };

    let current_output = run_scip_conversion(&scip_path, config)?;

    // Compare outputs
    let are_equivalent = baseline_helpers::compare_yaml_outputs(&baseline, &current_output)?;

    if !are_equivalent {
        println!("❌ Backward compatibility broken!");
        println!("Baseline size: {} bytes", baseline.len());
        println!("Current size: {} bytes", current_output.len());

        // Save current output for debugging
        std::fs::write(
            "tests/fixtures/baseline/debug_adhoc_default.yaml",
            &current_output,
        )?;
        println!("Current output saved to: tests/fixtures/baseline/debug_adhoc_default.yaml");
    }

    assert!(
        are_equivalent,
        "Default ad-hoc output should match baseline (Requirement 8.1, 8.2)"
    );

    Ok(())
}

/// Test that default ad-hoc format (Level 0) matches baseline for NestJS
///
/// **Validates: Requirements 8.1, 8.2**
#[test]
fn test_adhoc_default_matches_baseline_nestjs() -> Result<()> {
    // Skip if baseline doesn't exist
    if !baseline_helpers::baseline_exists("nestjs_high_adhoc_default") {
        println!(
            "⚠ Skipping: Baseline not found. Generate with: cargo test --test baseline_generator -- --ignored"
        );
        return Ok(());
    }

    // Load baseline (Level 0 output)
    let baseline = baseline_helpers::load_baseline("nestjs_high_adhoc_default")?;

    // Generate current output with default granularity
    let scip_path = PathBuf::from("../../../examples/nestjs-api-ts/index.scip");
    let project_root = PathBuf::from("../../../examples/nestjs-api-ts");

    let config = YcgConfig {
        lod: LevelOfDetail::High,
        project_root,
        compact: false,
        output_format: OutputFormat::AdHoc,
        ignore_framework_noise: false,
        file_filter: FileFilterConfig {
            include_patterns: vec![],
            exclude_patterns: vec![],
            use_gitignore: false,
        },
        adhoc_granularity: AdHocGranularity::Default,
    };

    let current_output = run_scip_conversion(&scip_path, config)?;

    // Compare outputs
    let are_equivalent = baseline_helpers::compare_yaml_outputs(&baseline, &current_output)?;

    if !are_equivalent {
        println!("❌ Backward compatibility broken!");
        println!("Baseline size: {} bytes", baseline.len());
        println!("Current size: {} bytes", current_output.len());

        std::fs::write(
            "tests/fixtures/baseline/debug_adhoc_default_nestjs.yaml",
            &current_output,
        )?;
        println!(
            "Current output saved to: tests/fixtures/baseline/debug_adhoc_default_nestjs.yaml"
        );
    }

    assert!(
        are_equivalent,
        "Default ad-hoc output should match baseline (Requirement 8.1, 8.2)"
    );

    Ok(())
}

/// Test that using default() constructor gives Level 0
///
/// **Validates: Requirement 8.2**
#[test]
fn test_adhoc_granularity_default_constructor() {
    let granularity = AdHocGranularity::default();
    assert_eq!(
        granularity,
        AdHocGranularity::Default,
        "Default constructor should return Level 0 (Requirement 8.2)"
    );
}

/// Test that Level 0 format has exactly 3 fields per definition
///
/// **Validates: Requirements 1.1, 9.1**
#[test]
fn test_level_0_has_three_fields() -> Result<()> {
    if !baseline_helpers::baseline_exists("simple_ts_high_adhoc_default") {
        println!("⚠ Skipping: Baseline not found");
        return Ok(());
    }

    let baseline = baseline_helpers::load_baseline("simple_ts_high_adhoc_default")?;

    // Parse YAML
    let yaml: serde_yaml::Value = serde_yaml::from_str(&baseline)?;

    // Get definitions
    let defs = yaml
        .get("_defs")
        .and_then(|v| v.as_sequence())
        .expect("Should have _defs array");

    // Check each definition has exactly 3 pipe-separated fields
    for def in defs {
        let def_str = def.as_str().expect("Definition should be string");
        let field_count = def_str.matches('|').count() + 1;

        assert_eq!(
            field_count, 3,
            "Level 0 definition should have exactly 3 fields: '{}' (Requirement 1.1, 9.1)",
            def_str
        );
    }

    Ok(())
}

/// Test that Level 1 format has exactly 3 fields per definition
///
/// **Validates: Requirements 2.8, 9.1**
#[test]
fn test_level_1_has_three_fields() -> Result<()> {
    if !baseline_helpers::baseline_exists("simple_ts_high_adhoc_signatures") {
        println!("⚠ Skipping: Baseline not found");
        return Ok(());
    }

    let baseline = baseline_helpers::load_baseline("simple_ts_high_adhoc_signatures")?;

    // Parse YAML
    let yaml: serde_yaml::Value = serde_yaml::from_str(&baseline)?;

    // Get definitions
    let defs = yaml
        .get("_defs")
        .and_then(|v| v.as_sequence())
        .expect("Should have _defs array");

    // Check each definition has exactly 3 pipe-separated fields
    for def in defs {
        let def_str = def.as_str().expect("Definition should be string");
        let field_count = def_str.matches('|').count() + 1;

        assert_eq!(
            field_count, 3,
            "Level 1 definition should have exactly 3 fields: '{}' (Requirement 2.8, 9.1)",
            def_str
        );
    }

    Ok(())
}

/// Test that Level 2 format has 3 or 4 fields per definition
///
/// **Validates: Requirements 3.2, 9.2**
#[test]
fn test_level_2_has_three_or_four_fields() -> Result<()> {
    if !baseline_helpers::baseline_exists("simple_ts_high_adhoc_logic") {
        println!("⚠ Skipping: Baseline not found");
        return Ok(());
    }

    let baseline = baseline_helpers::load_baseline("simple_ts_high_adhoc_logic")?;

    // Parse YAML
    let yaml: serde_yaml::Value = serde_yaml::from_str(&baseline)?;

    // Get definitions
    let defs = yaml
        .get("_defs")
        .and_then(|v| v.as_sequence())
        .expect("Should have _defs array");

    // Check each definition has 3 or 4 pipe-separated fields
    for def in defs {
        let def_str = def.as_str().expect("Definition should be string");
        let field_count = def_str.matches('|').count() + 1;

        assert!(
            field_count == 3 || field_count == 4,
            "Level 2 definition should have 3 or 4 fields: '{}' (Requirement 3.2, 9.2)",
            def_str
        );

        // If it has 4 fields, the 4th should start with "logic:"
        if field_count == 4 {
            let parts: Vec<&str> = def_str.split('|').collect();
            assert!(
                parts[3].starts_with("logic:"),
                "Fourth field should start with 'logic:': '{}' (Requirement 3.2)",
                def_str
            );
        }
    }

    Ok(())
}

/// Test that existing ad-hoc format still works
///
/// **Validates: Requirement 8.3**
#[test]
fn test_existing_adhoc_format_works() -> Result<()> {
    // This test verifies that the existing ad-hoc format (without granularity)
    // continues to work as expected

    let scip_path = PathBuf::from("../../../examples/simple-ts/index.scip");
    let project_root = PathBuf::from("../../../examples/simple-ts");

    if !scip_path.exists() {
        println!("⚠ Skipping: SCIP file not found");
        return Ok(());
    }

    // Use ad-hoc format with default granularity
    let config = YcgConfig {
        lod: LevelOfDetail::Medium,
        project_root,
        compact: false,
        output_format: OutputFormat::AdHoc,
        ignore_framework_noise: false,
        file_filter: FileFilterConfig {
            include_patterns: vec![],
            exclude_patterns: vec![],
            use_gitignore: false,
        },
        adhoc_granularity: AdHocGranularity::default(),
    };

    // Should not panic or error
    let output = run_scip_conversion(&scip_path, config)?;

    // Verify output is valid YAML
    let _yaml: serde_yaml::Value = serde_yaml::from_str(&output)?;

    // Verify it has the expected structure
    assert!(output.contains("_meta"));
    assert!(output.contains("_defs"));
    assert!(output.contains("graph"));

    Ok(())
}

/// Test that all granularity levels produce valid output
///
/// **Validates: Requirements 1.1, 2.1, 3.1**
#[test]
fn test_all_granularity_levels_produce_valid_output() -> Result<()> {
    let scip_path = PathBuf::from("../../../examples/simple-ts/index.scip");
    let project_root = PathBuf::from("../../../examples/simple-ts");

    if !scip_path.exists() {
        println!("⚠ Skipping: SCIP file not found");
        return Ok(());
    }

    let levels = vec![
        AdHocGranularity::Default,
        AdHocGranularity::InlineSignatures,
        AdHocGranularity::InlineLogic,
    ];

    for level in levels {
        let config = YcgConfig {
            lod: LevelOfDetail::High,
            project_root: project_root.clone(),
            compact: false,
            output_format: OutputFormat::AdHoc,
            ignore_framework_noise: false,
            file_filter: FileFilterConfig {
                include_patterns: vec![],
                exclude_patterns: vec![],
                use_gitignore: false,
            },
            adhoc_granularity: level,
        };

        // Should not panic or error
        let output = run_scip_conversion(&scip_path, config)?;

        // Verify output is valid YAML
        let yaml: serde_yaml::Value = serde_yaml::from_str(&output)?;

        // Verify structure
        assert!(yaml.get("_meta").is_some(), "Should have _meta");
        assert!(yaml.get("_defs").is_some(), "Should have _defs");
        assert!(yaml.get("graph").is_some(), "Should have graph");
    }

    Ok(())
}
