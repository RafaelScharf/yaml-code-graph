// Helper functions for baseline comparison in backward compatibility tests
// Used by Property 4: Backward Compatibility Without Flags

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Load a baseline output file
pub fn load_baseline(test_case_name: &str) -> Result<String> {
    let baseline_path = get_baseline_path(test_case_name);

    fs::read_to_string(&baseline_path).with_context(|| {
        format!(
            "Failed to load baseline: {:?}. Run baseline generator first with: \
             cargo test --test baseline_generator -- --ignored",
            baseline_path
        )
    })
}

/// Get the path to a baseline file
pub fn get_baseline_path(test_case_name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/baseline").join(format!("{}.yaml", test_case_name))
}

/// Check if baseline exists
pub fn baseline_exists(test_case_name: &str) -> bool {
    get_baseline_path(test_case_name).exists()
}

/// Compare two YAML outputs for semantic equivalence
/// This is more robust than byte-for-byte comparison as it handles
/// minor formatting differences while ensuring structural equivalence
pub fn compare_yaml_outputs(output1: &str, output2: &str) -> Result<bool> {
    // Parse both outputs as YAML
    let yaml1: serde_yaml::Value =
        serde_yaml::from_str(output1).context("Failed to parse first YAML output")?;
    let yaml2: serde_yaml::Value =
        serde_yaml::from_str(output2).context("Failed to parse second YAML output")?;

    // Compare the parsed structures
    Ok(yaml1 == yaml2)
}

/// Get all available baseline test cases
pub fn list_baseline_test_cases() -> Result<Vec<String>> {
    let baseline_dir = PathBuf::from("tests/fixtures/baseline");

    if !baseline_dir.exists() {
        return Ok(vec![]);
    }

    let mut test_cases = Vec::new();

    for entry in fs::read_dir(&baseline_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                test_cases.push(stem.to_string());
            }
        }
    }

    test_cases.sort();
    Ok(test_cases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_path_generation() {
        let path = get_baseline_path("simple_ts_low");
        assert!(path.to_string_lossy().contains("baseline"));
        assert!(path.to_string_lossy().contains("simple_ts_low.yaml"));
    }

    #[test]
    fn test_yaml_comparison() {
        let yaml1 = r#"
metadata:
  name: test
  version: "1.0"
definitions:
  - id: test_1
    name: TestClass
"#;

        let yaml2 = r#"
metadata:
  name: test
  version: "1.0"
definitions:
  - id: test_1
    name: TestClass
"#;

        let result = compare_yaml_outputs(yaml1, yaml2);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_yaml_comparison_different() {
        let yaml1 = r#"
metadata:
  name: test1
"#;

        let yaml2 = r#"
metadata:
  name: test2
"#;

        let result = compare_yaml_outputs(yaml1, yaml2);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
