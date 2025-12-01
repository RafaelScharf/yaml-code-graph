// crates/ycg_core/src/model.rs
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// --- MODELO PADRÃO (Flat List) ---
#[derive(Debug, Serialize, Deserialize)]
pub struct YcgGraph {
    #[serde(rename = "_meta")]
    pub metadata: ProjectMetadata,
    #[serde(rename = "_defs")]
    pub definitions: Vec<SymbolNode>,
    #[serde(rename = "graph", skip_serializing_if = "Vec::is_empty", default)]
    pub references: Vec<ReferenceEdge>,
}

// --- MODELO OTIMIZADO (Adjacency List) ---
#[derive(Debug, Serialize, Deserialize)]
pub struct YcgGraphOptimized {
    #[serde(rename = "_meta")]
    pub metadata: ProjectMetadata,
    #[serde(rename = "_defs")]
    pub definitions: Vec<SymbolNode>, // Os nós continuam iguais

    // O Grafo muda: Origem -> Tipo -> Lista de Destinos
    // BTreeMap garante ordem alfabética determinística (Requirement 7.2)
    #[serde(rename = "graph")]
    pub adjacency: BTreeMap<String, BTreeMap<EdgeType, Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SymbolNode {
    pub id: String,
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "t")]
    pub kind: ScipSymbolKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "doc")]
    pub documentation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sig")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logic: Option<LogicMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogicMetadata {
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "pre")]
    pub preconditions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub struct ReferenceEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum EdgeType {
    Calls,
    References,
    Imports,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScipSymbolKind {
    File,
    Module,
    Class,
    Method,
    Function,
    Variable,
    Interface,
}

// --- CONFIGURATION MODELS FOR TOKEN OPTIMIZATION ---

/// Configuration file format for YCG
#[derive(Debug, Deserialize, Clone)]
pub struct YcgConfigFile {
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub ignore: IgnoreConfig,
    #[serde(default)]
    pub include: Vec<String>,
}

/// Output configuration settings
#[derive(Debug, Deserialize, Clone, Default)]
pub struct OutputConfig {
    pub format: Option<String>,
    pub compact: Option<bool>,
    #[serde(rename = "ignoreFrameworkNoise")]
    pub ignore_framework_noise: Option<bool>,
    #[serde(rename = "adhocGranularity")]
    pub adhoc_granularity: Option<String>,
}

/// Ignore patterns configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct IgnoreConfig {
    #[serde(rename = "useGitignore")]
    pub use_gitignore: Option<bool>,
    #[serde(rename = "customPatterns")]
    pub custom_patterns: Option<Vec<String>>,
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Yaml,
    AdHoc,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Yaml
    }
}

/// File filtering configuration
#[derive(Debug, Clone, Default)]
pub struct FileFilterConfig {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub use_gitignore: bool,
}

// --- AD-HOC GRANULARITY CONFIGURATION ---

/// Granularity levels for ad-hoc format output
///
/// Controls the amount of detail included in each symbol definition.
/// All levels are opt-in through CLI flags or configuration file.
///
/// **Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdHocGranularity {
    /// Level 0: Default format (ID|Name|Type)
    /// Maximum token efficiency, structural information only
    /// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.6**
    Default,

    /// Level 1: Inline signatures (ID|Signature(args):Return|Type)
    /// Includes function signatures with abbreviated types
    /// **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8**
    InlineSignatures,

    /// Level 2: Inline logic (ID|Signature|Type|logic:steps)
    /// Includes signatures plus compact logic representation
    /// **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10, 3.11**
    InlineLogic,
}

impl Default for AdHocGranularity {
    fn default() -> Self {
        AdHocGranularity::Default
    }
}

impl AdHocGranularity {
    /// Parse from string (for config file)
    ///
    /// # Arguments
    /// * `s` - String value from configuration file
    ///
    /// # Returns
    /// * `Ok(AdHocGranularity)` if valid
    /// * `Err(String)` with error message if invalid
    ///
    /// # Valid Values
    /// - "default" → AdHocGranularity::Default
    /// - "signatures" → AdHocGranularity::InlineSignatures
    /// - "logic" → AdHocGranularity::InlineLogic
    ///
    /// **Validates: Requirements 7.3, 7.4, 7.5, 7.6**
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "default" => Ok(AdHocGranularity::Default),
            "signatures" => Ok(AdHocGranularity::InlineSignatures),
            "logic" => Ok(AdHocGranularity::InlineLogic),
            _ => Err(format!(
                "Invalid adhocGranularity value: '{}'. Valid values are: 'default', 'signatures', 'logic'",
                s
            )),
        }
    }

    /// Convert to string representation
    pub fn to_str(&self) -> &'static str {
        match self {
            AdHocGranularity::Default => "default",
            AdHocGranularity::InlineSignatures => "signatures",
            AdHocGranularity::InlineLogic => "logic",
        }
    }
}

// --- AD-HOC FORMAT MODEL ---

/// Ad-hoc format representation using pipe-separated strings
#[derive(Debug, Serialize)]
pub struct YcgGraphAdHoc {
    #[serde(rename = "_meta")]
    pub metadata: ProjectMetadata,

    #[serde(rename = "_defs")]
    pub definitions: Vec<String>, // Pipe-separated strings: "id|name|type"

    #[serde(rename = "graph")]
    pub adjacency: BTreeMap<String, BTreeMap<EdgeType, Vec<String>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adhoc_granularity_default() {
        let granularity = AdHocGranularity::default();
        assert_eq!(granularity, AdHocGranularity::Default);
    }

    #[test]
    fn test_adhoc_granularity_from_str_valid() {
        assert_eq!(
            AdHocGranularity::from_str("default").unwrap(),
            AdHocGranularity::Default
        );
        assert_eq!(
            AdHocGranularity::from_str("DEFAULT").unwrap(),
            AdHocGranularity::Default
        );
        assert_eq!(
            AdHocGranularity::from_str("signatures").unwrap(),
            AdHocGranularity::InlineSignatures
        );
        assert_eq!(
            AdHocGranularity::from_str("SIGNATURES").unwrap(),
            AdHocGranularity::InlineSignatures
        );
        assert_eq!(
            AdHocGranularity::from_str("logic").unwrap(),
            AdHocGranularity::InlineLogic
        );
        assert_eq!(
            AdHocGranularity::from_str("LOGIC").unwrap(),
            AdHocGranularity::InlineLogic
        );
    }

    #[test]
    fn test_adhoc_granularity_from_str_invalid() {
        let result = AdHocGranularity::from_str("invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("invalid"));
        assert!(err.contains("default"));
        assert!(err.contains("signatures"));
        assert!(err.contains("logic"));
    }

    #[test]
    fn test_adhoc_granularity_to_str() {
        assert_eq!(AdHocGranularity::Default.to_str(), "default");
        assert_eq!(AdHocGranularity::InlineSignatures.to_str(), "signatures");
        assert_eq!(AdHocGranularity::InlineLogic.to_str(), "logic");
    }

    #[test]
    fn test_adhoc_granularity_round_trip() {
        let levels = vec![
            AdHocGranularity::Default,
            AdHocGranularity::InlineSignatures,
            AdHocGranularity::InlineLogic,
        ];

        for level in levels {
            let str_repr = level.to_str();
            let parsed = AdHocGranularity::from_str(str_repr).unwrap();
            assert_eq!(level, parsed);
        }
    }
}
