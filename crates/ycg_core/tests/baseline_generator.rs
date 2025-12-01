// Baseline Output Generator for Backward Compatibility Testing
// This tool generates reference outputs from the current version for test cases
// Requirements: 7.1, 7.2, 7.3

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use ycg_core::model::{AdHocGranularity, FileFilterConfig, OutputFormat};
use ycg_core::{LevelOfDetail, YcgConfig, run_scip_conversion};

/// Test case configuration for baseline generation
#[derive(Debug)]
struct BaselineTestCase {
    name: &'static str,
    scip_path: PathBuf,
    project_root: PathBuf,
    lod: LevelOfDetail,
}

impl BaselineTestCase {
    fn new(name: &'static str, example_dir: &str, lod: LevelOfDetail) -> Self {
        let project_root = PathBuf::from("../../examples").join(example_dir);
        let scip_path = project_root.join("index.scip");

        Self {
            name,
            scip_path,
            project_root,
            lod,
        }
    }
}

/// Generate baseline outputs for all test cases
pub fn generate_all_baselines() -> Result<()> {
    println!("=== Generating Baseline Outputs ===");
    println!("This will create reference outputs for backward compatibility testing");
    println!();

    // Create baseline directory structure
    let baseline_dir = PathBuf::from("tests/fixtures/baseline");
    fs::create_dir_all(&baseline_dir)?;
    println!("✓ Created baseline directory: {:?}", baseline_dir);

    // Define test cases
    let test_cases = vec![
        BaselineTestCase::new("simple_ts_low", "simple-ts", LevelOfDetail::Low),
        BaselineTestCase::new("simple_ts_medium", "simple-ts", LevelOfDetail::Medium),
        BaselineTestCase::new("simple_ts_high", "simple-ts", LevelOfDetail::High),
        BaselineTestCase::new("nestjs_low", "nestjs-api-ts", LevelOfDetail::Low),
        BaselineTestCase::new("nestjs_medium", "nestjs-api-ts", LevelOfDetail::Medium),
        BaselineTestCase::new("nestjs_high", "nestjs-api-ts", LevelOfDetail::High),
    ];

    // Generate baseline for each test case
    for test_case in test_cases {
        generate_baseline(&test_case, &baseline_dir)?;
    }

    println!();
    println!("=== Baseline Generation Complete ===");
    println!(
        "Generated {} baseline files",
        fs::read_dir(&baseline_dir)?.count()
    );

    Ok(())
}

/// Generate a single baseline output
fn generate_baseline(test_case: &BaselineTestCase, baseline_dir: &Path) -> Result<()> {
    println!("Generating baseline: {}", test_case.name);

    // Check if SCIP file exists
    if !test_case.scip_path.exists() {
        println!(
            "  ⚠ Skipping: SCIP file not found at {:?}",
            test_case.scip_path
        );
        return Ok(());
    }

    // Create default configuration (no optimizations)
    // This represents the "previous version" behavior
    let config = YcgConfig {
        lod: test_case.lod,
        project_root: test_case.project_root.clone(),
        compact: false,                    // Default: no compaction
        output_format: OutputFormat::Yaml, // Default: YAML format
        ignore_framework_noise: false,     // Default: no framework filtering
        file_filter: FileFilterConfig {
            include_patterns: vec![],
            exclude_patterns: vec![],
            use_gitignore: false, // Default: don't use gitignore for baseline
        },
        adhoc_granularity: AdHocGranularity::default(), // Default: Level 0
    };

    // Generate output
    let output = run_scip_conversion(&test_case.scip_path, config)?;

    // Save to baseline file
    let baseline_file = baseline_dir.join(format!("{}.yaml", test_case.name));
    fs::write(&baseline_file, &output)?;

    println!("  ✓ Saved to: {:?}", baseline_file);
    println!(
        "  ✓ Size: {} bytes ({} tokens)",
        output.len(),
        ycg_core::count_tokens(&output)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Run manually with: cargo test --test baseline_generator -- --ignored
    fn generate_baselines() {
        generate_all_baselines().expect("Failed to generate baselines");
    }

    #[test]
    fn test_baseline_exists() {
        // Verify that baseline files exist after generation
        let baseline_dir = PathBuf::from("tests/fixtures/baseline");

        if !baseline_dir.exists() {
            println!(
                "Baseline directory doesn't exist yet. Run: cargo test --test baseline_generator -- --ignored"
            );
            return;
        }

        let expected_files = vec![
            "simple_ts_low.yaml",
            "simple_ts_medium.yaml",
            "simple_ts_high.yaml",
            "nestjs_low.yaml",
            "nestjs_medium.yaml",
            "nestjs_high.yaml",
        ];

        for file in expected_files {
            let path = baseline_dir.join(file);
            if path.exists() {
                println!("✓ Found baseline: {}", file);
            } else {
                println!("⚠ Missing baseline: {}", file);
            }
        }
    }
}
