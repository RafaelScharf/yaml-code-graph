// Integration test for config file loading and CLI merging

use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_file_loading() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("ycg.config.json");

    // Write a test config file
    let config_content = r#"{
  "output": {
    "format": "yaml",
    "compact": true,
    "ignoreFrameworkNoise": false
  },
  "ignore": {
    "useGitignore": true,
    "customPatterns": [
      "**/node_modules/**"
    ]
  },
  "include": [
    "**/*.ts"
  ]
}"#;

    fs::write(&config_path, config_content).unwrap();

    // Load the config using ConfigLoader
    use ycg_core::config::ConfigLoader;
    let loaded = ConfigLoader::load_from_file(&config_path).unwrap();

    assert!(loaded.is_some());
    let config = loaded.unwrap();

    // Verify the loaded values
    assert_eq!(config.output.format, Some("yaml".to_string()));
    assert_eq!(config.output.compact, Some(true));
    assert_eq!(config.output.ignore_framework_noise, Some(false));
    assert_eq!(config.ignore.use_gitignore, Some(true));
    assert_eq!(
        config.ignore.custom_patterns,
        Some(vec!["**/node_modules/**".to_string()])
    );
    assert_eq!(config.include, vec!["**/*.ts".to_string()]);
}

#[test]
fn test_cli_precedence_over_config() {
    use ycg_core::config::ConfigLoader;
    use ycg_core::model::{IgnoreConfig, OutputConfig, YcgConfigFile};

    // Create a file config
    let file_config = YcgConfigFile {
        output: OutputConfig {
            format: Some("yaml".to_string()),
            compact: Some(false),
            ignore_framework_noise: Some(false),
            adhoc_granularity: None,
        },
        ignore: IgnoreConfig {
            use_gitignore: Some(true),
            custom_patterns: Some(vec!["**/node_modules/**".to_string()]),
        },
        include: vec!["**/*.ts".to_string()],
    };

    // Merge with CLI args that override some settings
    let merged = ConfigLoader::merge_with_cli(
        Some(file_config),
        Some(true),                       // CLI compact = true (overrides file's false)
        Some("adhoc".to_string()),        // CLI format = adhoc (overrides file's yaml)
        Some(true), // CLI ignore_framework_noise = true (overrides file's false)
        vec!["**/*.rs".to_string()], // CLI include (overrides file's include)
        vec!["**/target/**".to_string()], // CLI exclude
        false,      // CLI no_gitignore = false (keeps gitignore enabled)
        None,       // CLI adhoc_granularity (not specified)
    )
    .unwrap();

    // Verify CLI values took precedence
    assert_eq!(merged.compact, true); // CLI overrode file
    assert_eq!(merged.ignore_framework_noise, true); // CLI overrode file
    assert_eq!(
        merged.file_filter.include_patterns,
        vec!["**/*.rs".to_string()]
    ); // CLI overrode file
    assert_eq!(
        merged.file_filter.exclude_patterns,
        vec!["**/target/**".to_string()]
    );
}

#[test]
fn test_config_validation_detects_conflicts() {
    use ycg_core::config::{ConfigLoader, MergedConfig};
    use ycg_core::model::{FileFilterConfig, OutputFormat};

    // Create a config with conflicting patterns
    let config = MergedConfig {
        compact: false,
        output_format: OutputFormat::Yaml,
        ignore_framework_noise: false,
        file_filter: FileFilterConfig {
            include_patterns: vec!["**/*.ts".to_string()],
            exclude_patterns: vec!["**/*.ts".to_string()], // Same pattern in both!
            use_gitignore: true,
        },
        adhoc_granularity: ycg_core::model::AdHocGranularity::default(),
    };

    // Validation should fail
    let result = ConfigLoader::validate(&config);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Conflicting patterns")
    );
}

#[test]
fn test_nonexistent_config_file() {
    use std::path::PathBuf;
    use ycg_core::config::ConfigLoader;

    let nonexistent = PathBuf::from("/tmp/nonexistent_ycg_config_12345.json");
    let result = ConfigLoader::load_from_file(&nonexistent).unwrap();

    // Should return None, not an error
    assert!(result.is_none());
}

#[test]
fn test_malformed_config_file() {
    use tempfile::TempDir;
    use ycg_core::config::ConfigLoader;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bad_config.json");

    // Write malformed JSON
    fs::write(&config_path, "{ this is not valid json }").unwrap();

    let result = ConfigLoader::load_from_file(&config_path);

    // Should return an error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to parse"));
}

#[test]
fn test_invalid_output_format() {
    use ycg_core::config::ConfigLoader;
    use ycg_core::model::{IgnoreConfig, OutputConfig, YcgConfigFile};

    let file_config = YcgConfigFile {
        output: OutputConfig {
            format: None,
            compact: None,
            ignore_framework_noise: None,
            adhoc_granularity: None,
        },
        ignore: IgnoreConfig {
            use_gitignore: None,
            custom_patterns: None,
        },
        include: vec![],
    };

    // Try to merge with invalid output format
    let result = ConfigLoader::merge_with_cli(
        Some(file_config),
        None,
        Some("invalid_format".to_string()), // Invalid format
        None,
        vec![],
        vec![],
        false,
        None, // CLI adhoc_granularity
    );

    // Should return an error
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid output format")
    );
}

#[test]
fn test_cli_granularity_precedence_over_config() {
    // Requirement 7.2: CLI precedence over config file for granularity
    use ycg_core::config::ConfigLoader;
    use ycg_core::model::{AdHocGranularity, IgnoreConfig, OutputConfig, YcgConfigFile};

    // Create a file config with granularity = "default"
    let file_config = YcgConfigFile {
        output: OutputConfig {
            format: Some("adhoc".to_string()),
            compact: None,
            ignore_framework_noise: None,
            adhoc_granularity: Some("default".to_string()),
        },
        ignore: IgnoreConfig {
            use_gitignore: None,
            custom_patterns: None,
        },
        include: vec![],
    };

    // Merge with CLI granularity = InlineLogic (should override file's "default")
    let merged = ConfigLoader::merge_with_cli(
        Some(file_config),
        None,
        None,
        None,
        vec![],
        vec![],
        false,
        Some(AdHocGranularity::InlineLogic), // CLI overrides file
    )
    .unwrap();

    // Verify CLI value took precedence
    assert_eq!(merged.adhoc_granularity, AdHocGranularity::InlineLogic);
}

#[test]
fn test_config_file_granularity_when_no_cli() {
    // Requirement 7.1: Config file granularity used when no CLI flag
    use ycg_core::config::ConfigLoader;
    use ycg_core::model::{AdHocGranularity, IgnoreConfig, OutputConfig, YcgConfigFile};

    // Create a file config with granularity = "signatures"
    let file_config = YcgConfigFile {
        output: OutputConfig {
            format: Some("adhoc".to_string()),
            compact: None,
            ignore_framework_noise: None,
            adhoc_granularity: Some("signatures".to_string()),
        },
        ignore: IgnoreConfig {
            use_gitignore: None,
            custom_patterns: None,
        },
        include: vec![],
    };

    // Merge with no CLI granularity (None)
    let merged = ConfigLoader::merge_with_cli(
        Some(file_config),
        None,
        None,
        None,
        vec![],
        vec![],
        false,
        None, // No CLI override
    )
    .unwrap();

    // Verify file config value was used
    assert_eq!(merged.adhoc_granularity, AdHocGranularity::InlineSignatures);
}

#[test]
fn test_invalid_granularity_in_config_file() {
    // Requirement 7.6: Invalid granularity value in config file
    use ycg_core::config::ConfigLoader;
    use ycg_core::model::{IgnoreConfig, OutputConfig, YcgConfigFile};

    // Create a file config with invalid granularity
    let file_config = YcgConfigFile {
        output: OutputConfig {
            format: Some("adhoc".to_string()),
            compact: None,
            ignore_framework_noise: None,
            adhoc_granularity: Some("invalid_level".to_string()),
        },
        ignore: IgnoreConfig {
            use_gitignore: None,
            custom_patterns: None,
        },
        include: vec![],
    };

    // Merge should fail with clear error
    let result = ConfigLoader::merge_with_cli(
        Some(file_config),
        None,
        None,
        None,
        vec![],
        vec![],
        false,
        None,
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("invalid_level"));
    assert!(err_msg.contains("default"));
    assert!(err_msg.contains("signatures"));
    assert!(err_msg.contains("logic"));
}
