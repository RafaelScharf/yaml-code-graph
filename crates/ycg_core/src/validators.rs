// crates/ycg_core/src/validators.rs
//! Output validation for YCG graphs in both YAML and Ad-Hoc formats.
//!
//! This module provides validators to ensure:
//! - YAML output conforms to YAML 1.2 specification
//! - Ad-Hoc format has correct structure (3 pipe-separated fields)
//! - Graph edges maintain referential integrity (all IDs exist)

use crate::model::{YcgGraph, YcgGraphAdHoc, YcgGraphOptimized};
use anyhow::{Context, Result, anyhow};
use std::collections::HashSet;

/// Validator for YAML format output
pub struct YamlValidator;

impl YamlValidator {
    /// Validate that YAML output conforms to YAML 1.2 specification
    ///
    /// Uses serde_yaml to parse and validate the structure. This ensures:
    /// - Valid YAML syntax
    /// - Correct structure for YcgGraph or YcgGraphOptimized
    /// - All required fields are present
    ///
    /// # Arguments
    /// * `output` - The YAML string to validate
    ///
    /// # Returns
    /// * `Ok(())` if valid
    /// * `Err` with descriptive message if invalid
    ///
    /// **Validates: Requirements 8.1**
    pub fn validate(output: &str) -> Result<()> {
        // Try parsing as standard YcgGraph first
        if let Ok(_graph) = serde_yaml::from_str::<YcgGraph>(output) {
            return Ok(());
        }

        // Try parsing as optimized YcgGraphOptimized
        if let Ok(_graph) = serde_yaml::from_str::<YcgGraphOptimized>(output) {
            return Ok(());
        }

        // If both fail, provide a helpful error message
        Err(anyhow!(
            "Invalid YAML output: does not conform to YcgGraph or YcgGraphOptimized structure"
        ))
        .context("YAML validation failed")
    }
}

/// Validator for Ad-Hoc format output
pub struct AdHocValidator;

impl AdHocValidator {
    /// Validate that Ad-Hoc format output has correct structure
    ///
    /// Checks that:
    /// - Each definition has exactly 3 pipe-separated fields (id|name|type)
    /// - All graph edge references point to valid symbol IDs
    ///
    /// # Arguments
    /// * `graph` - The YcgGraphAdHoc to validate
    ///
    /// # Returns
    /// * `Ok(())` if valid
    /// * `Err` with descriptive message if invalid
    ///
    /// **Validates: Requirements 8.2, 8.3**
    pub fn validate(graph: &YcgGraphAdHoc) -> Result<()> {
        // Validate field count for each definition
        for (idx, def) in graph.definitions.iter().enumerate() {
            let field_count = count_unescaped_pipes(def) + 1;
            if field_count != 3 {
                return Err(anyhow!(
                    "Invalid ad-hoc format at definition {}: expected 3 fields, got {}. Definition: '{}'",
                    idx,
                    field_count,
                    def
                ));
            }
        }

        // Validate referential integrity
        validate_graph_integrity_adhoc(graph)
            .context("Ad-hoc format referential integrity check failed")?;

        Ok(())
    }
}

/// Validate graph edge referential integrity for standard YcgGraph
///
/// Ensures all graph edges reference valid symbol identifiers that exist
/// in the definitions section.
///
/// # Arguments
/// * `graph` - The YcgGraph to validate
///
/// # Returns
/// * `Ok(())` if all edges reference valid symbols
/// * `Err` with details about invalid references
///
/// **Validates: Requirements 8.3**
pub fn validate_graph_integrity(graph: &YcgGraph) -> Result<()> {
    // Build set of all valid symbol IDs
    let valid_ids: HashSet<String> = graph
        .definitions
        .iter()
        .map(|node| node.id.clone())
        .collect();

    // Check each edge
    let mut invalid_edges = Vec::new();

    for edge in &graph.references {
        let from_valid = valid_ids.contains(&edge.from);
        let to_valid = valid_ids.contains(&edge.to);

        if !from_valid || !to_valid {
            invalid_edges.push((edge.from.clone(), edge.to.clone(), !from_valid, !to_valid));
        }
    }

    if !invalid_edges.is_empty() {
        let mut error_msg = format!(
            "Graph referential integrity violation: {} invalid edge(s) found\n",
            invalid_edges.len()
        );

        for (from, to, from_invalid, to_invalid) in invalid_edges.iter().take(5) {
            if *from_invalid && *to_invalid {
                error_msg.push_str(&format!(
                    "  - Edge from '{}' to '{}': both IDs not found in definitions\n",
                    from, to
                ));
            } else if *from_invalid {
                error_msg.push_str(&format!(
                    "  - Edge from '{}' to '{}': source ID not found in definitions\n",
                    from, to
                ));
            } else {
                error_msg.push_str(&format!(
                    "  - Edge from '{}' to '{}': target ID not found in definitions\n",
                    from, to
                ));
            }
        }

        if invalid_edges.len() > 5 {
            error_msg.push_str(&format!("  ... and {} more\n", invalid_edges.len() - 5));
        }

        return Err(anyhow!(error_msg));
    }

    Ok(())
}

/// Validate graph edge referential integrity for optimized YcgGraphOptimized
///
/// Ensures all graph edges in the adjacency list reference valid symbol
/// identifiers that exist in the definitions section.
///
/// # Arguments
/// * `graph` - The YcgGraphOptimized to validate
///
/// # Returns
/// * `Ok(())` if all edges reference valid symbols
/// * `Err` with details about invalid references
///
/// **Validates: Requirements 8.3**
pub fn validate_graph_integrity_optimized(graph: &YcgGraphOptimized) -> Result<()> {
    // Build set of all valid symbol IDs
    let valid_ids: HashSet<String> = graph
        .definitions
        .iter()
        .map(|node| node.id.clone())
        .collect();

    // Check each edge in adjacency list
    let mut invalid_edges = Vec::new();

    for (from, edges) in &graph.adjacency {
        let from_valid = valid_ids.contains(from);

        for (_edge_type, targets) in edges {
            for to in targets {
                let to_valid = valid_ids.contains(to);

                if !from_valid || !to_valid {
                    invalid_edges.push((from.clone(), to.clone(), !from_valid, !to_valid));
                }
            }
        }
    }

    if !invalid_edges.is_empty() {
        let mut error_msg = format!(
            "Graph referential integrity violation: {} invalid edge(s) found\n",
            invalid_edges.len()
        );

        for (from, to, from_invalid, to_invalid) in invalid_edges.iter().take(5) {
            if *from_invalid && *to_invalid {
                error_msg.push_str(&format!(
                    "  - Edge from '{}' to '{}': both IDs not found in definitions\n",
                    from, to
                ));
            } else if *from_invalid {
                error_msg.push_str(&format!(
                    "  - Edge from '{}' to '{}': source ID not found in definitions\n",
                    from, to
                ));
            } else {
                error_msg.push_str(&format!(
                    "  - Edge from '{}' to '{}': target ID not found in definitions\n",
                    from, to
                ));
            }
        }

        if invalid_edges.len() > 5 {
            error_msg.push_str(&format!("  ... and {} more\n", invalid_edges.len() - 5));
        }

        return Err(anyhow!(error_msg));
    }

    Ok(())
}

/// Validate graph edge referential integrity for ad-hoc format
///
/// Ensures all graph edges in the adjacency list reference valid symbol
/// identifiers that exist in the definitions section.
///
/// # Arguments
/// * `graph` - The YcgGraphAdHoc to validate
///
/// # Returns
/// * `Ok(())` if all edges reference valid symbols
/// * `Err` with details about invalid references
fn validate_graph_integrity_adhoc(graph: &YcgGraphAdHoc) -> Result<()> {
    // Extract IDs from pipe-separated definitions
    let mut valid_ids = HashSet::new();

    for def in &graph.definitions {
        // Split by unescaped pipes and take first field (ID)
        let parts: Vec<&str> = def.split('|').collect();
        if !parts.is_empty() {
            // Unescape the ID
            let id = parts[0].replace(r"\|", "|");
            valid_ids.insert(id);
        }
    }

    // Check each edge in adjacency list
    let mut invalid_edges = Vec::new();

    for (from, edges) in &graph.adjacency {
        let from_valid = valid_ids.contains(from);

        for (_edge_type, targets) in edges {
            for to in targets {
                let to_valid = valid_ids.contains(to);

                if !from_valid || !to_valid {
                    invalid_edges.push((from.clone(), to.clone(), !from_valid, !to_valid));
                }
            }
        }
    }

    if !invalid_edges.is_empty() {
        let mut error_msg = format!(
            "Graph referential integrity violation: {} invalid edge(s) found\n",
            invalid_edges.len()
        );

        for (from, to, from_invalid, to_invalid) in invalid_edges.iter().take(5) {
            if *from_invalid && *to_invalid {
                error_msg.push_str(&format!(
                    "  - Edge from '{}' to '{}': both IDs not found in definitions\n",
                    from, to
                ));
            } else if *from_invalid {
                error_msg.push_str(&format!(
                    "  - Edge from '{}' to '{}': source ID not found in definitions\n",
                    from, to
                ));
            } else {
                error_msg.push_str(&format!(
                    "  - Edge from '{}' to '{}': target ID not found in definitions\n",
                    from, to
                ));
            }
        }

        if invalid_edges.len() > 5 {
            error_msg.push_str(&format!("  ... and {} more\n", invalid_edges.len() - 5));
        }

        return Err(anyhow!(error_msg));
    }

    Ok(())
}

/// Count unescaped pipe characters in a string
///
/// Escaped pipes (`\|`) are not counted as separators.
fn count_unescaped_pipes(s: &str) -> usize {
    let mut count = 0;
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            // Skip the next character if it's a pipe
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '|' {
                    chars.next(); // Skip the escaped pipe
                }
            }
        } else if ch == '|' {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EdgeType, ProjectMetadata, ReferenceEdge, ScipSymbolKind, SymbolNode};
    use std::collections::BTreeMap;

    #[test]
    fn test_yaml_validator_valid_graph() {
        let graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![SymbolNode {
                id: "A_0001".to_string(),
                name: "ClassA".to_string(),
                kind: ScipSymbolKind::Class,
                parent_id: None,
                documentation: None,
                signature: None,
                logic: None,
            }],
            references: vec![],
        };

        let yaml = serde_yaml::to_string(&graph).unwrap();
        let result = YamlValidator::validate(&yaml);
        if let Err(e) = &result {
            eprintln!("Validation error: {}", e);
            eprintln!("YAML content:\n{}", yaml);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_yaml_validator_invalid_yaml() {
        let invalid_yaml = "this is not: valid: yaml: structure";
        assert!(YamlValidator::validate(invalid_yaml).is_err());
    }

    #[test]
    fn test_adhoc_validator_valid_format() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|methodB|method".to_string(),
            ],
            adjacency: BTreeMap::new(),
        };

        assert!(AdHocValidator::validate(&graph).is_ok());
    }

    #[test]
    fn test_adhoc_validator_invalid_field_count() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|only_two".to_string(), // Invalid: only 2 fields
            ],
            adjacency: BTreeMap::new(),
        };

        let result = AdHocValidator::validate(&graph);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected 3 fields")
        );
    }

    #[test]
    fn test_adhoc_validator_with_escaped_pipes() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                r"A\|0001|Class\|A|class".to_string(), // Escaped pipes should not count
            ],
            adjacency: BTreeMap::new(),
        };

        assert!(AdHocValidator::validate(&graph).is_ok());
    }

    #[test]
    fn test_validate_graph_integrity_valid() {
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
                    parent_id: None,
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

        assert!(validate_graph_integrity(&graph).is_ok());
    }

    #[test]
    fn test_validate_graph_integrity_invalid_source() {
        let graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![SymbolNode {
                id: "A_0001".to_string(),
                name: "ClassA".to_string(),
                kind: ScipSymbolKind::Class,
                parent_id: None,
                documentation: None,
                signature: None,
                logic: None,
            }],
            references: vec![ReferenceEdge {
                from: "INVALID_ID".to_string(),
                to: "A_0001".to_string(),
                edge_type: EdgeType::Calls,
            }],
        };

        let result = validate_graph_integrity(&graph);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("source ID not found")
        );
    }

    #[test]
    fn test_validate_graph_integrity_invalid_target() {
        let graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![SymbolNode {
                id: "A_0001".to_string(),
                name: "ClassA".to_string(),
                kind: ScipSymbolKind::Class,
                parent_id: None,
                documentation: None,
                signature: None,
                logic: None,
            }],
            references: vec![ReferenceEdge {
                from: "A_0001".to_string(),
                to: "INVALID_TARGET".to_string(),
                edge_type: EdgeType::Calls,
            }],
        };

        let result = validate_graph_integrity(&graph);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("target ID not found")
        );
    }

    #[test]
    fn test_validate_graph_integrity_optimized_valid() {
        let mut adjacency = BTreeMap::new();
        let mut edges = BTreeMap::new();
        edges.insert(EdgeType::Calls, vec!["A_0001".to_string()]);
        adjacency.insert("B_0002".to_string(), edges);

        let graph = YcgGraphOptimized {
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
                    parent_id: None,
                    documentation: None,
                    signature: None,
                    logic: None,
                },
            ],
            adjacency,
        };

        assert!(validate_graph_integrity_optimized(&graph).is_ok());
    }

    #[test]
    fn test_validate_graph_integrity_optimized_invalid() {
        let mut adjacency = BTreeMap::new();
        let mut edges = BTreeMap::new();
        edges.insert(EdgeType::Calls, vec!["INVALID_ID".to_string()]);
        adjacency.insert("B_0002".to_string(), edges);

        let graph = YcgGraphOptimized {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![SymbolNode {
                id: "B_0002".to_string(),
                name: "methodB".to_string(),
                kind: ScipSymbolKind::Method,
                parent_id: None,
                documentation: None,
                signature: None,
                logic: None,
            }],
            adjacency,
        };

        let result = validate_graph_integrity_optimized(&graph);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("target ID not found")
        );
    }

    #[test]
    fn test_validate_graph_integrity_adhoc_valid() {
        let mut adjacency = BTreeMap::new();
        let mut edges = BTreeMap::new();
        edges.insert(EdgeType::Calls, vec!["A_0001".to_string()]);
        adjacency.insert("B_0002".to_string(), edges);

        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|methodB|method".to_string(),
            ],
            adjacency,
        };

        assert!(validate_graph_integrity_adhoc(&graph).is_ok());
    }

    #[test]
    fn test_validate_graph_integrity_adhoc_invalid() {
        let mut adjacency = BTreeMap::new();
        let mut edges = BTreeMap::new();
        edges.insert(EdgeType::Calls, vec!["INVALID_ID".to_string()]);
        adjacency.insert("B_0002".to_string(), edges);

        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec!["B_0002|methodB|method".to_string()],
            adjacency,
        };

        let result = validate_graph_integrity_adhoc(&graph);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("target ID not found")
        );
    }

    #[test]
    fn test_count_unescaped_pipes() {
        assert_eq!(count_unescaped_pipes("a|b|c"), 2);
        assert_eq!(count_unescaped_pipes(r"a\|b|c"), 1);
        assert_eq!(count_unescaped_pipes(r"a\|b\|c"), 0);
        assert_eq!(count_unescaped_pipes("no pipes here"), 0);
        assert_eq!(count_unescaped_pipes(r"one\|escaped|one|not"), 2);
    }

    #[test]
    fn test_adhoc_validator_referential_integrity() {
        let mut adjacency = BTreeMap::new();
        let mut edges = BTreeMap::new();
        edges.insert(EdgeType::Calls, vec!["NONEXISTENT".to_string()]);
        adjacency.insert("A_0001".to_string(), edges);

        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec!["A_0001|ClassA|class".to_string()],
            adjacency,
        };

        let result = AdHocValidator::validate(&graph);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("referential integrity")
        );
    }
}
