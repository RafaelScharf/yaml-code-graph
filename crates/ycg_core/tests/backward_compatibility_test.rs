// Integration test demonstrating backward compatibility verification
// This test shows how Property 4 (Backward Compatibility Without Flags) will be implemented
// Requirements: 7.1, 7.2, 7.3

mod baseline_helpers;

use anyhow::Result;
use std::path::PathBuf;
use ycg_core::model::{AdHocGranularity, FileFilterConfig, OutputFormat};
use ycg_core::{LevelOfDetail, YcgConfig, run_scip_conversion};

/// Test that default configuration produces output matching baseline
#[test]
fn test_simple_ts_medium_backward_compatibility() -> Result<()> {
    // Skip if baseline doesn't exist
    if !baseline_helpers::baseline_exists("simple_ts_medium") {
        println!(
            "⚠ Skipping: Baseline not found. Generate with: cargo test --test baseline_generator -- --ignored"
        );
        return Ok(());
    }

    // Load baseline (represents "previous version" output)
    let baseline = baseline_helpers::load_baseline("simple_ts_medium")?;

    // Generate current output with default configuration
    let scip_path = PathBuf::from("../../../examples/simple-ts/index.scip");
    let project_root = PathBuf::from("../../../examples/simple-ts");

    let config = YcgConfig {
        lod: LevelOfDetail::Medium,
        project_root,
        compact: false,                    // Default: no optimization
        output_format: OutputFormat::Yaml, // Default: YAML
        ignore_framework_noise: false,     // Default: no filtering
        file_filter: FileFilterConfig {
            include_patterns: vec![],
            exclude_patterns: vec![],
            use_gitignore: false,
        },
        adhoc_granularity: AdHocGranularity::default(), // Default: Level 0
    };

    let current_output = run_scip_conversion(&scip_path, config)?;

    // Compare outputs - they should be semantically equivalent
    let are_equivalent = baseline_helpers::compare_yaml_outputs(&baseline, &current_output)?;

    if !are_equivalent {
        println!("❌ Backward compatibility broken!");
        println!("Baseline size: {} bytes", baseline.len());
        println!("Current size: {} bytes", current_output.len());

        // Save current output for debugging
        std::fs::write(
            "tests/fixtures/baseline/debug_current.yaml",
            &current_output,
        )?;
        println!("Current output saved to: tests/fixtures/baseline/debug_current.yaml");
    }

    assert!(
        are_equivalent,
        "Output with default settings should match baseline (Requirement 7.1, 7.2, 7.3)"
    );

    Ok(())
}

/// Test that all baseline test cases maintain backward compatibility
#[test]
fn test_all_baselines_backward_compatibility() -> Result<()> {
    let test_cases = baseline_helpers::list_baseline_test_cases()?;

    if test_cases.is_empty() {
        println!(
            "⚠ No baselines found. Generate with: cargo test --test baseline_generator -- --ignored"
        );
        return Ok(());
    }

    println!(
        "Testing {} baseline cases for backward compatibility",
        test_cases.len()
    );

    let mut failures = Vec::new();

    for test_case in &test_cases {
        // Skip adhoc baselines - they're tested separately in granularity_backward_compatibility_test
        if test_case.contains("adhoc") {
            continue;
        }

        println!("  Testing: {}", test_case);

        // Parse test case name to extract example and LOD
        let (example_dir, lod) = parse_test_case_name(test_case);

        // Load baseline
        let baseline = match baseline_helpers::load_baseline(test_case) {
            Ok(b) => b,
            Err(e) => {
                println!("    ⚠ Skipping: {}", e);
                continue;
            }
        };

        // Generate current output
        let scip_path = PathBuf::from(format!("../../../examples/{}/index.scip", example_dir));
        let project_root = PathBuf::from(format!("../../../examples/{}", example_dir));

        if !scip_path.exists() {
            println!("    ⚠ Skipping: SCIP file not found");
            continue;
        }

        let config = YcgConfig {
            lod,
            project_root,
            compact: false,
            output_format: OutputFormat::Yaml,
            ignore_framework_noise: false,
            file_filter: FileFilterConfig {
                include_patterns: vec![],
                exclude_patterns: vec![],
                use_gitignore: false,
            },
            adhoc_granularity: AdHocGranularity::default(),
        };

        let current_output = match run_scip_conversion(&scip_path, config) {
            Ok(o) => o,
            Err(e) => {
                println!("    ❌ Failed to generate output: {}", e);
                failures.push(test_case.clone());
                continue;
            }
        };

        // Compare
        match baseline_helpers::compare_yaml_outputs(&baseline, &current_output) {
            Ok(true) => println!("    ✓ Backward compatible"),
            Ok(false) => {
                println!("    ❌ Output differs from baseline");
                failures.push(test_case.clone());
            }
            Err(e) => {
                println!("    ❌ Comparison failed: {}", e);
                failures.push(test_case.clone());
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "Backward compatibility broken for {} test cases: {:?}",
            failures.len(),
            failures
        );
    }

    Ok(())
}

/// Parse test case name to extract example directory and LOD
fn parse_test_case_name(name: &str) -> (&str, LevelOfDetail) {
    let lod = if name.ends_with("_low") {
        LevelOfDetail::Low
    } else if name.ends_with("_medium") {
        LevelOfDetail::Medium
    } else {
        LevelOfDetail::High
    };

    let example_dir = if name.starts_with("simple_ts") {
        "simple-ts"
    } else if name.starts_with("nestjs") {
        "nestjs-api-ts"
    } else {
        "simple-ts" // default
    };

    (example_dir, lod)
}

#[test]
fn test_parse_test_case_name() {
    let (dir, lod) = parse_test_case_name("simple_ts_low");
    assert_eq!(dir, "simple-ts");
    assert!(matches!(lod, LevelOfDetail::Low));

    let (dir, lod) = parse_test_case_name("nestjs_medium");
    assert_eq!(dir, "nestjs-api-ts");
    assert!(matches!(lod, LevelOfDetail::Medium));

    let (dir, lod) = parse_test_case_name("simple_ts_high");
    assert_eq!(dir, "simple-ts");
    assert!(matches!(lod, LevelOfDetail::High));
}
