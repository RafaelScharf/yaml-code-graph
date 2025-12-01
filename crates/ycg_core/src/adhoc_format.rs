// crates/ycg_core/src/adhoc_format.rs
//! Ad-Hoc format serialization and parsing for ultra-compact graph representation.
//!
//! This module provides bidirectional transformation between YcgGraph and a pipe-separated
//! format optimized for minimal token usage. Format: `id|name|type`

use crate::model::{ScipSymbolKind, SymbolNode, YcgGraph, YcgGraphAdHoc};
use anyhow::{Context, Result, anyhow};

/// Serializer for converting graph nodes to ad-hoc pipe-separated format
pub struct AdHocSerializer;

impl AdHocSerializer {
    /// Serialize a single SymbolNode to pipe-separated format: "id|name|type"
    ///
    /// Escapes pipe characters in values using backslash: `|` becomes `\|`
    ///
    /// # Example
    /// ```ignore
    /// let node = SymbolNode {
    ///     id: "User_b8c1".to_string(),
    ///     name: "User".to_string(),
    ///     kind: ScipSymbolKind::Class,
    ///     // ... other fields
    /// };
    /// let serialized = AdHocSerializer::serialize_node(&node);
    /// assert_eq!(serialized, "User_b8c1|User|class");
    /// ```
    pub fn serialize_node(node: &SymbolNode) -> String {
        let id = escape_pipes(&node.id);
        let name = escape_pipes(&node.name);
        let kind = kind_to_string(&node.kind);

        format!("{}|{}|{}", id, name, kind)
    }

    /// Serialize entire YcgGraph to YcgGraphAdHoc format
    ///
    /// Converts all definitions to pipe-separated strings while preserving
    /// metadata and adjacency structure.
    pub fn serialize_graph(graph: &YcgGraph) -> YcgGraphAdHoc {
        let definitions: Vec<String> = graph
            .definitions
            .iter()
            .map(|node| Self::serialize_node(node))
            .collect();

        // Convert flat references to adjacency list
        let mut adjacency = std::collections::BTreeMap::new();

        for edge in &graph.references {
            let node_edges = adjacency
                .entry(edge.from.clone())
                .or_insert_with(std::collections::BTreeMap::new);

            let targets = node_edges.entry(edge.edge_type).or_insert_with(Vec::new);

            targets.push(edge.to.clone());
        }

        // Sort targets for determinism
        for inner_map in adjacency.values_mut() {
            for targets in inner_map.values_mut() {
                targets.sort();
            }
        }

        YcgGraphAdHoc {
            metadata: graph.metadata.clone(),
            definitions,
            adjacency,
        }
    }
}

/// Parser for reconstructing graph nodes from ad-hoc format
pub struct AdHocParser;

impl AdHocParser {
    /// Parse a pipe-separated string back into a SymbolNode
    ///
    /// Expected format: "id|name|type"
    /// Handles escaped pipes: `\|` becomes `|`
    ///
    /// # Errors
    /// Returns error if:
    /// - String doesn't have exactly 3 pipe-separated fields
    /// - Type field is not a valid ScipSymbolKind
    pub fn parse_node(raw: &str) -> Result<SymbolNode> {
        let parts = split_escaped(raw)?;

        if parts.len() != 3 {
            return Err(anyhow!(
                "Invalid ad-hoc format: expected 3 fields, got {}. Input: '{}'",
                parts.len(),
                raw
            ));
        }

        let id = unescape_pipes(&parts[0]);
        let name = unescape_pipes(&parts[1]);
        let kind = string_to_kind(&parts[2])
            .with_context(|| format!("Invalid symbol kind: '{}'", parts[2]))?;

        Ok(SymbolNode {
            id,
            name,
            kind,
            parent_id: None,
            documentation: None,
            signature: None,
            logic: None,
        })
    }

    /// Parse complete YcgGraphAdHoc back to YcgGraph
    ///
    /// Reconstructs the full graph structure from ad-hoc format.
    /// Note: This is primarily for testing round-trip consistency.
    pub fn parse_graph(adhoc: &YcgGraphAdHoc) -> Result<YcgGraph> {
        let definitions: Result<Vec<SymbolNode>> = adhoc
            .definitions
            .iter()
            .map(|s| Self::parse_node(s))
            .collect();

        let definitions = definitions?;

        // Convert adjacency list back to flat references
        let mut references = Vec::new();

        for (from, edges) in &adhoc.adjacency {
            for (edge_type, targets) in edges {
                for to in targets {
                    references.push(crate::model::ReferenceEdge {
                        from: from.clone(),
                        to: to.clone(),
                        edge_type: *edge_type,
                    });
                }
            }
        }

        references.sort();

        Ok(YcgGraph {
            metadata: adhoc.metadata.clone(),
            definitions,
            references,
        })
    }
}

// --- Helper Functions ---

/// Escape pipe characters in a string: `|` -> `\|`
fn escape_pipes(s: &str) -> String {
    s.replace('|', r"\|")
}

/// Unescape pipe characters: `\|` -> `|`
fn unescape_pipes(s: &str) -> String {
    s.replace(r"\|", "|")
}

/// Split a string by unescaped pipes
///
/// Handles escaped pipes correctly: "a\|b|c" -> ["a\|b", "c"]
fn split_escaped(s: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            // Check if next char is a pipe
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '|' {
                    // Escaped pipe - keep the escape sequence
                    current.push(ch);
                    current.push(chars.next().unwrap());
                } else {
                    // Not an escaped pipe, just a backslash
                    current.push(ch);
                }
            } else {
                // Backslash at end of string
                current.push(ch);
            }
        } else if ch == '|' {
            // Unescaped pipe - field separator
            parts.push(current.clone());
            current.clear();
        } else {
            current.push(ch);
        }
    }

    // Don't forget the last field
    parts.push(current);

    Ok(parts)
}

/// Convert ScipSymbolKind to lowercase string
fn kind_to_string(kind: &ScipSymbolKind) -> String {
    match kind {
        ScipSymbolKind::File => "file",
        ScipSymbolKind::Module => "module",
        ScipSymbolKind::Class => "class",
        ScipSymbolKind::Method => "method",
        ScipSymbolKind::Function => "function",
        ScipSymbolKind::Variable => "variable",
        ScipSymbolKind::Interface => "interface",
    }
    .to_string()
}

/// Convert lowercase string to ScipSymbolKind
fn string_to_kind(s: &str) -> Result<ScipSymbolKind> {
    match s.to_lowercase().as_str() {
        "file" => Ok(ScipSymbolKind::File),
        "module" => Ok(ScipSymbolKind::Module),
        "class" => Ok(ScipSymbolKind::Class),
        "method" => Ok(ScipSymbolKind::Method),
        "function" => Ok(ScipSymbolKind::Function),
        "variable" => Ok(ScipSymbolKind::Variable),
        "interface" => Ok(ScipSymbolKind::Interface),
        _ => Err(anyhow!("Unknown symbol kind: '{}'", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EdgeType, ProjectMetadata, ReferenceEdge};

    #[test]
    fn test_serialize_node_basic() {
        let node = SymbolNode {
            id: "User_b8c1".to_string(),
            name: "User".to_string(),
            kind: ScipSymbolKind::Class,
            parent_id: None,
            documentation: None,
            signature: None,
            logic: None,
        };

        let result = AdHocSerializer::serialize_node(&node);
        assert_eq!(result, "User_b8c1|User|class");
    }

    #[test]
    fn test_serialize_node_with_pipes() {
        let node = SymbolNode {
            id: "weird|id".to_string(),
            name: "name|with|pipes".to_string(),
            kind: ScipSymbolKind::Function,
            parent_id: None,
            documentation: None,
            signature: None,
            logic: None,
        };

        let result = AdHocSerializer::serialize_node(&node);
        assert_eq!(result, r"weird\|id|name\|with\|pipes|function");
    }

    #[test]
    fn test_parse_node_basic() {
        let input = "greet_a3f2|greet|function";
        let result = AdHocParser::parse_node(input).unwrap();

        assert_eq!(result.id, "greet_a3f2");
        assert_eq!(result.name, "greet");
        assert_eq!(result.kind, ScipSymbolKind::Function);
    }

    #[test]
    fn test_parse_node_with_escaped_pipes() {
        let input = r"weird\|id|name\|with\|pipes|method";
        let result = AdHocParser::parse_node(input).unwrap();

        assert_eq!(result.id, "weird|id");
        assert_eq!(result.name, "name|with|pipes");
        assert_eq!(result.kind, ScipSymbolKind::Method);
    }

    #[test]
    fn test_parse_node_invalid_field_count() {
        let input = "only|two";
        let result = AdHocParser::parse_node(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected 3 fields")
        );
    }

    #[test]
    fn test_parse_node_invalid_kind() {
        let input = "id|name|invalid_kind";
        let result = AdHocParser::parse_node(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid symbol kind")
        );
    }

    #[test]
    fn test_round_trip_single_node() {
        let original = SymbolNode {
            id: "Test_1234".to_string(),
            name: "TestClass".to_string(),
            kind: ScipSymbolKind::Class,
            parent_id: None,
            documentation: None,
            signature: None,
            logic: None,
        };

        let serialized = AdHocSerializer::serialize_node(&original);
        let parsed = AdHocParser::parse_node(&serialized).unwrap();

        assert_eq!(original.id, parsed.id);
        assert_eq!(original.name, parsed.name);
        assert_eq!(original.kind, parsed.kind);
    }

    #[test]
    fn test_serialize_graph() {
        let graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                SymbolNode {
                    id: "A_0001".to_string(),
                    name: "ClassA".to_string(),
                    kind: ScipSymbolKind::Class,
                    parent_id: None,
                    documentation: None,
                    signature: None,
                    logic: None,
                },
                SymbolNode {
                    id: "B_0002".to_string(),
                    name: "methodB".to_string(),
                    kind: ScipSymbolKind::Method,
                    parent_id: Some("A_0001".to_string()),
                    documentation: None,
                    signature: None,
                    logic: None,
                },
            ],
            references: vec![ReferenceEdge {
                from: "B_0002".to_string(),
                to: "A_0001".to_string(),
                edge_type: EdgeType::Calls,
            }],
        };

        let adhoc = AdHocSerializer::serialize_graph(&graph);

        assert_eq!(adhoc.definitions.len(), 2);
        assert_eq!(adhoc.definitions[0], "A_0001|ClassA|class");
        assert_eq!(adhoc.definitions[1], "B_0002|methodB|method");
        assert_eq!(adhoc.metadata.name, "test");
    }

    #[test]
    fn test_parse_graph_round_trip() {
        let original = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                SymbolNode {
                    id: "User_b8c1".to_string(),
                    name: "User".to_string(),
                    kind: ScipSymbolKind::Class,
                    parent_id: None,
                    documentation: None,
                    signature: None,
                    logic: None,
                },
                SymbolNode {
                    id: "greet_a3f2".to_string(),
                    name: "greet".to_string(),
                    kind: ScipSymbolKind::Function,
                    parent_id: None,
                    documentation: None,
                    signature: None,
                    logic: None,
                },
            ],
            references: vec![ReferenceEdge {
                from: "greet_a3f2".to_string(),
                to: "User_b8c1".to_string(),
                edge_type: EdgeType::References,
            }],
        };

        let adhoc = AdHocSerializer::serialize_graph(&original);
        let parsed = AdHocParser::parse_graph(&adhoc).unwrap();

        assert_eq!(original.definitions.len(), parsed.definitions.len());
        assert_eq!(original.references.len(), parsed.references.len());
        assert_eq!(original.metadata.name, parsed.metadata.name);

        // Check first definition
        assert_eq!(original.definitions[0].id, parsed.definitions[0].id);
        assert_eq!(original.definitions[0].name, parsed.definitions[0].name);
        assert_eq!(original.definitions[0].kind, parsed.definitions[0].kind);
    }

    #[test]
    fn test_split_escaped() {
        let result = split_escaped("a|b|c").unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);

        let result = split_escaped(r"a\|b|c").unwrap();
        assert_eq!(result, vec![r"a\|b", "c"]);

        let result = split_escaped(r"a\|b\|c|d").unwrap();
        assert_eq!(result, vec![r"a\|b\|c", "d"]);
    }

    #[test]
    fn test_escape_unescape_pipes() {
        let original = "test|with|pipes";
        let escaped = escape_pipes(original);
        assert_eq!(escaped, r"test\|with\|pipes");

        let unescaped = unescape_pipes(&escaped);
        assert_eq!(unescaped, original);
    }

    #[test]
    fn test_kind_conversion() {
        assert_eq!(kind_to_string(&ScipSymbolKind::Class), "class");
        assert_eq!(kind_to_string(&ScipSymbolKind::Function), "function");

        assert_eq!(string_to_kind("class").unwrap(), ScipSymbolKind::Class);
        assert_eq!(string_to_kind("CLASS").unwrap(), ScipSymbolKind::Class);
        assert_eq!(
            string_to_kind("function").unwrap(),
            ScipSymbolKind::Function
        );

        assert!(string_to_kind("invalid").is_err());
    }
}
