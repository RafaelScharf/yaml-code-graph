// crates/ycg_core/src/errors.rs
//! Error types for YCG token optimization features
//!
//! This module defines comprehensive error types for all YCG operations,
//! with a focus on providing clear, actionable error messages to users.
//!
//! **Requirements: 5.9, 6.8**

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for YCG operations
///
/// This enum encompasses all possible errors that can occur during
/// YCG processing, from configuration loading to graph generation.
#[derive(Error, Debug)]
pub enum YcgError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// File system operation errors
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    /// SCIP parsing errors
    #[error("SCIP parsing error: {0}")]
    Parse(#[from] ParseError),

    /// Output validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// File filtering errors
    #[error("File filtering error: {0}")]
    FileFilter(#[from] FileFilterError),

    /// Generic errors (for compatibility with anyhow)
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Configuration-specific errors
///
/// These errors occur during configuration file loading and validation.
///
/// **Validates: Requirements 5.9, 6.8**
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Malformed JSON in configuration file
    ///
    /// **Validates: Requirement 5.9**
    #[error("Malformed JSON in config file '{path}' at line {line}, column {column}: {message}")]
    MalformedJson {
        path: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },

    /// Invalid output format specified
    ///
    /// **Validates: Requirement 3.6**
    #[error("Invalid output format: '{provided}'. Valid options are: {}", valid.join(", "))]
    InvalidOutputFormat {
        provided: String,
        valid: Vec<String>,
    },

    /// Conflicting CLI flags or configuration options
    ///
    /// **Validates: Requirement 6.8**
    #[error("Conflicting configuration: {flag1} conflicts with {flag2}. {suggestion}")]
    ConflictingFlags {
        flag1: String,
        flag2: String,
        suggestion: String,
    },

    /// Invalid glob pattern in include/exclude configuration
    #[error("Invalid glob pattern '{pattern}': {reason}")]
    InvalidGlobPattern { pattern: String, reason: String },

    /// Configuration file not found (non-fatal, can use defaults)
    #[error("Configuration file not found at '{path}'. Using default settings.")]
    ConfigFileNotFound { path: PathBuf },

    /// Failed to read configuration file
    #[error("Failed to read configuration file '{path}': {reason}")]
    ConfigFileReadError { path: PathBuf, reason: String },

    /// Missing required configuration field
    #[error("Missing required configuration field: '{field}' in '{path}'")]
    MissingRequiredField { field: String, path: PathBuf },

    /// Invalid configuration value
    #[error("Invalid value for '{field}': {value}. {reason}")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },
}

/// SCIP parsing and processing errors
#[derive(Error, Debug)]
pub enum ParseError {
    /// SCIP index file not found
    #[error(
        "SCIP index file not found: '{path}'. Please run 'scip-typescript index' or 'rust-analyzer scip .' first."
    )]
    ScipFileNotFound { path: PathBuf },

    /// Corrupted or invalid SCIP protobuf data
    #[error("Failed to decode SCIP index from '{path}': {reason}. The file may be corrupted.")]
    ScipDecodeError { path: PathBuf, reason: String },

    /// Invalid SCIP document structure
    #[error("Invalid SCIP document structure in '{path}': {reason}")]
    InvalidScipStructure { path: PathBuf, reason: String },

    /// Source file referenced in SCIP but not found
    #[error(
        "Source file '{file}' referenced in SCIP index not found in project root '{project_root}'"
    )]
    SourceFileNotFound { file: String, project_root: PathBuf },

    /// Failed to read source file for enrichment
    #[error("Failed to read source file '{path}': {reason}")]
    SourceFileReadError { path: PathBuf, reason: String },
}

/// Output validation errors
///
/// These errors occur when validating generated output for correctness.
///
/// **Validates: Requirements 8.1, 8.2, 8.3**
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Invalid YAML output structure
    ///
    /// **Validates: Requirement 8.1**
    #[error(
        "Invalid YAML output: {reason}. The output does not conform to YAML 1.2 specification."
    )]
    InvalidYaml { reason: String },

    /// Invalid ad-hoc format structure
    ///
    /// **Validates: Requirement 8.2**
    #[error(
        "Invalid ad-hoc format at definition {index}: {reason}. Expected format: 'id|name|type'"
    )]
    InvalidAdHocFormat { index: usize, reason: String },

    /// Graph referential integrity violation
    ///
    /// **Validates: Requirement 8.3**
    #[error("Graph referential integrity violation: {count} invalid edge(s) found.\n{details}")]
    ReferentialIntegrityViolation { count: usize, details: String },

    /// Empty graph generated (possibly due to over-filtering)
    #[error(
        "Generated graph is empty. This may be due to overly restrictive filters. Try adjusting your include/exclude patterns."
    )]
    EmptyGraph,

    /// Invalid symbol definition
    #[error("Invalid symbol definition at index {index}: {reason}")]
    InvalidSymbolDefinition { index: usize, reason: String },
}

/// File filtering errors
///
/// These errors occur during file pattern matching and filtering.
#[derive(Error, Debug)]
pub enum FileFilterError {
    /// Invalid glob pattern syntax
    #[error(
        "Invalid glob pattern '{pattern}': {reason}. See https://docs.rs/glob for pattern syntax."
    )]
    InvalidPattern { pattern: String, reason: String },

    /// Gitignore file parsing error (non-fatal)
    #[error(
        "Warning: Failed to parse .gitignore file at '{path}': {reason}. Continuing without gitignore filtering."
    )]
    GitignoreParseError { path: PathBuf, reason: String },

    /// No files match the specified patterns
    #[error("No files match the specified include patterns: {}. Check your patterns and project structure.", patterns.join(", "))]
    NoFilesMatched { patterns: Vec<String> },

    /// All files excluded by filters
    #[error("All files were excluded by filters. Include patterns: [{}], Exclude patterns: [{}]", 
            include_patterns.join(", "), exclude_patterns.join(", "))]
    AllFilesExcluded {
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
    },
}

// Conversion implementations for better error handling

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::MalformedJson {
            path: PathBuf::from("unknown"),
            line: err.line(),
            column: err.column(),
            message: err.to_string(),
        }
    }
}

impl From<glob::PatternError> for FileFilterError {
    fn from(err: glob::PatternError) -> Self {
        FileFilterError::InvalidPattern {
            pattern: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl ConfigError {
    /// Create a MalformedJson error with path context
    pub fn malformed_json_with_path(path: PathBuf, err: serde_json::Error) -> Self {
        ConfigError::MalformedJson {
            path,
            line: err.line(),
            column: err.column(),
            message: err.to_string(),
        }
    }

    /// Create an InvalidOutputFormat error with standard valid options
    pub fn invalid_output_format(provided: String) -> Self {
        ConfigError::InvalidOutputFormat {
            provided,
            valid: vec!["yaml".to_string(), "adhoc".to_string()],
        }
    }

    /// Create a ConflictingFlags error with a helpful suggestion
    pub fn conflicting_flags_with_suggestion(
        flag1: String,
        flag2: String,
        suggestion: String,
    ) -> Self {
        ConfigError::ConflictingFlags {
            flag1,
            flag2,
            suggestion,
        }
    }
}

impl ParseError {
    /// Create a ScipFileNotFound error
    pub fn scip_not_found(path: PathBuf) -> Self {
        ParseError::ScipFileNotFound { path }
    }

    /// Create a ScipDecodeError with context
    pub fn scip_decode_error(path: PathBuf, reason: String) -> Self {
        ParseError::ScipDecodeError { path, reason }
    }
}

impl ValidationError {
    /// Create a ReferentialIntegrityViolation with formatted details
    pub fn referential_integrity(invalid_edges: Vec<(String, String, bool, bool)>) -> Self {
        let count = invalid_edges.len();
        let mut details = String::new();

        for (from, to, from_invalid, to_invalid) in invalid_edges.iter().take(5) {
            if *from_invalid && *to_invalid {
                details.push_str(&format!(
                    "  - Edge from '{}' to '{}': both IDs not found in definitions\n",
                    from, to
                ));
            } else if *from_invalid {
                details.push_str(&format!(
                    "  - Edge from '{}' to '{}': source ID not found in definitions\n",
                    from, to
                ));
            } else {
                details.push_str(&format!(
                    "  - Edge from '{}' to '{}': target ID not found in definitions\n",
                    from, to
                ));
            }
        }

        if count > 5 {
            details.push_str(&format!("  ... and {} more\n", count - 5));
        }

        ValidationError::ReferentialIntegrityViolation { count, details }
    }

    /// Create an InvalidAdHocFormat error
    pub fn invalid_adhoc_format(index: usize, reason: String) -> Self {
        ValidationError::InvalidAdHocFormat { index, reason }
    }

    /// Create an InvalidYaml error
    pub fn invalid_yaml(reason: String) -> Self {
        ValidationError::InvalidYaml { reason }
    }
}

impl FileFilterError {
    /// Create an InvalidPattern error with context
    pub fn invalid_pattern(pattern: String, reason: String) -> Self {
        FileFilterError::InvalidPattern { pattern, reason }
    }

    /// Create a NoFilesMatched error
    pub fn no_files_matched(patterns: Vec<String>) -> Self {
        FileFilterError::NoFilesMatched { patterns }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::MalformedJson {
            path: PathBuf::from("ycg.config.json"),
            line: 5,
            column: 12,
            message: "expected comma".to_string(),
        };

        let display = format!("{}", err);
        assert!(display.contains("ycg.config.json"));
        assert!(display.contains("line 5"));
        assert!(display.contains("column 12"));
    }

    #[test]
    fn test_invalid_output_format_error() {
        let err = ConfigError::invalid_output_format("xml".to_string());
        let display = format!("{}", err);
        assert!(display.contains("xml"));
        assert!(display.contains("yaml"));
        assert!(display.contains("adhoc"));
    }

    #[test]
    fn test_conflicting_flags_error() {
        let err = ConfigError::conflicting_flags_with_suggestion(
            "--compact".to_string(),
            "--no-compact".to_string(),
            "Use only one of these flags".to_string(),
        );

        let display = format!("{}", err);
        assert!(display.contains("--compact"));
        assert!(display.contains("--no-compact"));
        assert!(display.contains("Use only one"));
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::scip_not_found(PathBuf::from("index.scip"));
        let display = format!("{}", err);
        assert!(display.contains("index.scip"));
        assert!(display.contains("not found"));
    }

    #[test]
    fn test_validation_error_referential_integrity() {
        let invalid_edges = vec![
            ("A".to_string(), "B".to_string(), false, true),
            ("C".to_string(), "D".to_string(), true, false),
        ];

        let err = ValidationError::referential_integrity(invalid_edges);
        let display = format!("{}", err);
        assert!(display.contains("2 invalid edge"));
        assert!(display.contains("target ID not found"));
        assert!(display.contains("source ID not found"));
    }

    #[test]
    fn test_file_filter_error_display() {
        let err =
            FileFilterError::no_files_matched(vec!["**/*.rs".to_string(), "**/*.ts".to_string()]);

        let display = format!("{}", err);
        assert!(display.contains("**/*.rs"));
        assert!(display.contains("**/*.ts"));
    }

    #[test]
    fn test_ycg_error_from_config_error() {
        let config_err = ConfigError::invalid_output_format("json".to_string());
        let ycg_err: YcgError = config_err.into();

        let display = format!("{}", ycg_err);
        assert!(display.contains("Configuration error"));
        assert!(display.contains("json"));
    }

    #[test]
    fn test_validation_error_invalid_adhoc() {
        let err = ValidationError::invalid_adhoc_format(3, "expected 3 fields, got 2".to_string());

        let display = format!("{}", err);
        assert!(display.contains("definition 3"));
        assert!(display.contains("expected 3 fields"));
        assert!(display.contains("id|name|type"));
    }

    #[test]
    fn test_empty_graph_error() {
        let err = ValidationError::EmptyGraph;
        let display = format!("{}", err);
        assert!(display.contains("empty"));
        assert!(display.contains("filters"));
    }

    #[test]
    fn test_all_files_excluded_error() {
        let err = FileFilterError::AllFilesExcluded {
            include_patterns: vec!["**/*.rs".to_string()],
            exclude_patterns: vec!["**/*".to_string()],
        };

        let display = format!("{}", err);
        assert!(display.contains("All files were excluded"));
        assert!(display.contains("**/*.rs"));
        assert!(display.contains("**/*"));
    }
}
