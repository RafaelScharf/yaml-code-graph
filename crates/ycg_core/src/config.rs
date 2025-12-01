// crates/ycg_core/src/config.rs

use crate::model::{FileFilterConfig, OutputFormat, YcgConfigFile};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Configuration loader for YCG
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a file
    /// Returns None if the file doesn't exist
    /// Returns an error if the file exists but is malformed
    pub fn load_from_file(path: &Path) -> Result<Option<YcgConfigFile>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: YcgConfigFile = serde_json::from_str(&content).with_context(|| {
            format!(
                "Failed to parse config file as JSON: {:?}. Please check the syntax.",
                path
            )
        })?;

        Ok(Some(config))
    }

    /// Merge file configuration with CLI arguments
    /// CLI arguments take precedence over file configuration
    pub fn merge_with_cli(
        file_config: Option<YcgConfigFile>,
        cli_compact: Option<bool>,
        cli_output_format: Option<String>,
        cli_ignore_framework_noise: Option<bool>,
        cli_include_patterns: Vec<String>,
        cli_exclude_patterns: Vec<String>,
        cli_no_gitignore: bool,
    ) -> Result<MergedConfig> {
        let mut merged = MergedConfig::default();

        // Start with file config if available
        if let Some(file_cfg) = file_config {
            // Output settings
            if let Some(compact) = file_cfg.output.compact {
                merged.compact = compact;
            }
            if let Some(format_str) = file_cfg.output.format {
                merged.output_format = Self::parse_output_format(&format_str)?;
            }
            if let Some(ignore_noise) = file_cfg.output.ignore_framework_noise {
                merged.ignore_framework_noise = ignore_noise;
            }

            // File filter settings
            merged.file_filter.include_patterns = file_cfg.include;
            if let Some(custom_patterns) = file_cfg.ignore.custom_patterns {
                merged.file_filter.exclude_patterns = custom_patterns;
            }
            if let Some(use_gitignore) = file_cfg.ignore.use_gitignore {
                merged.file_filter.use_gitignore = use_gitignore;
            }
        }

        // CLI overrides file config
        if let Some(compact) = cli_compact {
            merged.compact = compact;
        }
        if let Some(format_str) = cli_output_format {
            merged.output_format = Self::parse_output_format(&format_str)?;
        }
        if let Some(ignore_noise) = cli_ignore_framework_noise {
            merged.ignore_framework_noise = ignore_noise;
        }

        // CLI patterns override file patterns
        if !cli_include_patterns.is_empty() {
            merged.file_filter.include_patterns = cli_include_patterns;
        }
        if !cli_exclude_patterns.is_empty() {
            merged.file_filter.exclude_patterns = cli_exclude_patterns;
        }
        if cli_no_gitignore {
            merged.file_filter.use_gitignore = false;
        }

        Ok(merged)
    }

    /// Validate the merged configuration for conflicts
    pub fn validate(config: &MergedConfig) -> Result<()> {
        // Check for conflicting patterns
        for include in &config.file_filter.include_patterns {
            for exclude in &config.file_filter.exclude_patterns {
                if include == exclude {
                    anyhow::bail!(
                        "Conflicting patterns: '{}' appears in both include and exclude patterns",
                        include
                    );
                }
            }
        }

        Ok(())
    }

    /// Parse output format string
    fn parse_output_format(format_str: &str) -> Result<OutputFormat> {
        match format_str.to_lowercase().as_str() {
            "yaml" => Ok(OutputFormat::Yaml),
            "adhoc" => Ok(OutputFormat::AdHoc),
            _ => anyhow::bail!(
                "Invalid output format: '{}'. Valid options are: 'yaml', 'adhoc'",
                format_str
            ),
        }
    }
}

/// Merged configuration from file and CLI
#[derive(Debug, Clone)]
pub struct MergedConfig {
    pub compact: bool,
    pub output_format: OutputFormat,
    pub ignore_framework_noise: bool,
    pub file_filter: FileFilterConfig,
}

impl Default for MergedConfig {
    fn default() -> Self {
        Self {
            compact: false,
            output_format: OutputFormat::default(),
            ignore_framework_noise: false,
            file_filter: FileFilterConfig {
                include_patterns: Vec::new(),
                exclude_patterns: Vec::new(),
                use_gitignore: true, // Default to respecting gitignore
            },
        }
    }
}
