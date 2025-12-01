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
    #[serde(rename = "graph", skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<ReferenceEdge>,
}

// --- MODELO OTIMIZADO (Adjacency List) ---
#[derive(Debug, Serialize)]
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
