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

#[derive(Debug, Serialize, Deserialize, Default)]
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
