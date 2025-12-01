// crates/ycg_core/src/validators.rs
//! Output validation for YCG graphs in both YAML and Ad-Hoc formats.
//!
//! This module provides validators to ensure:
//! - YAML output conforms to YAML 1.2 specification
//! - Ad-Hoc format has correct structure (3 pipe-separated fields)
//! - Graph edges maintain referential integrity (all IDs exist)

use crate::model::{AdHocGranularity, YcgGraph, YcgGraphAdHoc, YcgGraphOptimized};
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

/// Validate ad-hoc format output based on granularity level
///
/// This function validates that ad-hoc definitions conform to the expected
/// structure for the specified granularity level:
/// - Level 0 (Default): 3 fields (ID|Name|Type)
/// - Level 1 (InlineSignatures): 3 fields (ID|Signature|Type)
/// - Level 2 (InlineLogic): 3 or 4 fields (ID|Signature|Type|logic:steps)
///
/// Additionally validates:
/// - Logic field format (must start with "logic:")
/// - Logic keywords are valid (check, action, return, match, get)
///
/// # Arguments
/// * `graph` - The YcgGraphAdHoc to validate
/// * `granularity` - The granularity level to validate against
///
/// # Returns
/// * `Ok(())` if valid
/// * `Err` with descriptive message if invalid
///
/// **Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5, 9.6**
pub fn validate_adhoc_granularity(
    graph: &YcgGraphAdHoc,
    granularity: AdHocGranularity,
) -> Result<()> {
    for (idx, def) in graph.definitions.iter().enumerate() {
        let field_count = count_unescaped_pipes(def) + 1;

        // Validate field count based on granularity level
        match granularity {
            AdHocGranularity::Default | AdHocGranularity::InlineSignatures => {
                // Level 0 and Level 1: exactly 3 fields
                if field_count != 3 {
                    return Err(anyhow!(
                        "Invalid ad-hoc format at definition {} for granularity level '{}': \
                         expected 3 fields, got {}. Definition: '{}'",
                        idx,
                        granularity.to_str(),
                        field_count,
                        def
                    ));
                }
            }
            AdHocGranularity::InlineLogic => {
                // Level 2: 3 or 4 fields (logic field is optional)
                if field_count != 3 && field_count != 4 {
                    return Err(anyhow!(
                        "Invalid ad-hoc format at definition {} for granularity level 'logic': \
                         expected 3 or 4 fields, got {}. Definition: '{}'",
                        idx,
                        field_count,
                        def
                    ));
                }

                // If 4 fields, validate the logic field
                if field_count == 4 {
                    let parts: Vec<&str> = split_unescaped_pipes(def);
                    let logic_field = parts[3];

                    // Validate logic field starts with "logic:"
                    if !logic_field.starts_with("logic:") {
                        return Err(anyhow!(
                            "Invalid logic field at definition {}: \
                             logic field must start with 'logic:', got '{}'. Definition: '{}'",
                            idx,
                            logic_field,
                            def
                        ));
                    }

                    // Extract logic content after "logic:" prefix
                    let logic_content = &logic_field[6..]; // Skip "logic:"

                    // Validate logic keywords
                    validate_logic_keywords(logic_content, idx, def)?;
                }
            }
        }
    }

    Ok(())
}

/// Validate that logic content uses only valid keywords
///
/// Valid keywords: check, action, return, match, get
/// Logic steps are separated by semicolons
///
/// # Arguments
/// * `logic_content` - The logic content after "logic:" prefix
/// * `idx` - Definition index for error reporting
/// * `def` - Full definition string for error reporting
///
/// # Returns
/// * `Ok(())` if all keywords are valid
/// * `Err` with details about invalid keywords
fn validate_logic_keywords(logic_content: &str, idx: usize, def: &str) -> Result<()> {
    const VALID_LOGIC_KEYWORDS: &[&str] = &["check", "action", "return", "match", "get"];

    // Split by semicolons to get individual logic steps
    let steps: Vec<&str> = logic_content.split(';').map(|s| s.trim()).collect();

    for step in steps {
        if step.is_empty() {
            continue;
        }

        // Extract keyword (everything before the opening parenthesis)
        let keyword = if let Some(paren_pos) = step.find('(') {
            &step[..paren_pos]
        } else {
            // No parenthesis found - this might be a truncated logic (ending with "...")
            // or malformed logic
            if step.ends_with("...") {
                // Truncated logic is acceptable
                continue;
            }
            step
        };

        // Check if keyword is valid
        if !VALID_LOGIC_KEYWORDS.contains(&keyword) {
            return Err(anyhow!(
                "Invalid logic keyword at definition {}: \
                 found '{}', valid keywords are: {}. Definition: '{}'",
                idx,
                keyword,
                VALID_LOGIC_KEYWORDS.join(", "),
                def
            ));
        }
    }

    Ok(())
}

/// Split a string by unescaped pipes
///
/// Returns a vector of string slices, properly handling escaped pipes.
fn split_unescaped_pipes(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut current_start = 0;
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '|' {
            // Skip escaped pipe
            i += 2;
        } else if chars[i] == '|' {
            // Found unescaped pipe - extract substring
            parts.push(&s[current_start..i]);
            current_start = i + 1;
            i += 1;
        } else {
            i += 1;
        }
    }

    // Add the last part
    if current_start < s.len() {
        parts.push(&s[current_start..]);
    }

    parts
}

/// Validator for Ad-Hoc format output
pub struct AdHocValidator;

impl AdHocValidator {
    /// Validate that Ad-Hoc format output has correct structure (Level 0 - Default)
    ///
    /// Checks that:
    /// - Each definition has exactly 3 pipe-separated fields (id|name|type)
    /// - All graph edge references point to valid symbol IDs
    ///
    /// This method validates Level 0 (Default) format. For granularity-aware
    /// validation, use `validate_with_granularity()`.
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
        // Validate field count for each definition (Level 0: exactly 3 fields)
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

    /// Validate Ad-Hoc format output with granularity level awareness
    ///
    /// Performs comprehensive validation including:
    /// - Field count validation based on granularity level
    /// - Logic field format validation (for Level 2)
    /// - Logic keyword validation (for Level 2)
    /// - Graph referential integrity
    ///
    /// # Arguments
    /// * `graph` - The YcgGraphAdHoc to validate
    /// * `granularity` - The granularity level to validate against
    ///
    /// # Returns
    /// * `Ok(())` if valid
    /// * `Err` with descriptive message if invalid
    ///
    /// **Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5, 9.6**
    pub fn validate_with_granularity(
        graph: &YcgGraphAdHoc,
        granularity: AdHocGranularity,
    ) -> Result<()> {
        // Validate granularity-specific structure
        validate_adhoc_granularity(graph, granularity)
            .context("Ad-hoc granularity validation failed")?;

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
    use crate::model::{
        AdHocGranularity, EdgeType, ProjectMetadata, ReferenceEdge, ScipSymbolKind, SymbolNode,
    };
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

    // --- GRANULARITY VALIDATION TESTS ---

    #[test]
    fn test_validate_adhoc_granularity_level0_valid() {
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

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::Default);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_adhoc_granularity_level0_invalid_field_count() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|methodB|method|extra_field".to_string(), // 4 fields - invalid for Level 0
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::Default);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expected 3 fields"));
        assert!(err_msg.contains("got 4"));
    }

    #[test]
    fn test_validate_adhoc_granularity_level1_valid() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|methodB(param:str):bool|method".to_string(),
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineSignatures);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_adhoc_granularity_level1_invalid_field_count() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|methodB(param:str):bool".to_string(), // Only 2 fields - invalid
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineSignatures);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expected 3 fields"));
        assert!(err_msg.contains("got 2"));
    }

    #[test]
    fn test_validate_adhoc_granularity_level2_valid_with_logic() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|methodB(param:str):bool|method|logic:check(param);return(true)".to_string(),
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineLogic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_adhoc_granularity_level2_valid_without_logic() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|methodB(param:str):bool|method".to_string(), // 3 fields - valid (logic optional)
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineLogic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_adhoc_granularity_level2_invalid_field_count() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|methodB(param:str):bool|method|logic:check(param)|extra".to_string(), // 5 fields - invalid
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineLogic);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expected 3 or 4 fields"));
        assert!(err_msg.contains("got 5"));
    }

    #[test]
    fn test_validate_logic_field_format_missing_prefix() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "B_0002|methodB(param:str):bool|method|check(param);return(true)".to_string(), // Missing "logic:" prefix
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineLogic);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("logic field must start with 'logic:'"));
    }

    #[test]
    fn test_validate_logic_keywords_valid() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "B_0002|methodB|method|logic:check(x>0);action(save);get(data);match(x)?a:b;return(result)".to_string(),
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineLogic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_logic_keywords_invalid() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "B_0002|methodB|method|logic:check(x>0);invalid_keyword(data);return(result)"
                    .to_string(),
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineLogic);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid logic keyword"));
        assert!(err_msg.contains("invalid_keyword"));
        assert!(err_msg.contains("check, action, return, match, get"));
    }

    #[test]
    fn test_validate_logic_keywords_truncated() {
        // Truncated logic (ending with "...") should be accepted
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "B_0002|methodB|method|logic:check(x>0);action(save);get(data);match(x)?a:b;return(res...".to_string(),
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineLogic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_granularity_level0() {
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

        let result = AdHocValidator::validate_with_granularity(&graph, AdHocGranularity::Default);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_granularity_level1() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|methodB(x:num):str|method".to_string(),
            ],
            adjacency: BTreeMap::new(),
        };

        let result =
            AdHocValidator::validate_with_granularity(&graph, AdHocGranularity::InlineSignatures);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_granularity_level2() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "A_0001|ClassA|class".to_string(),
                "B_0002|methodB(x:num):str|method|logic:check(x>0);return(x.toString())"
                    .to_string(),
            ],
            adjacency: BTreeMap::new(),
        };

        let result =
            AdHocValidator::validate_with_granularity(&graph, AdHocGranularity::InlineLogic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_split_unescaped_pipes() {
        let parts = split_unescaped_pipes("a|b|c");
        assert_eq!(parts, vec!["a", "b", "c"]);

        let parts = split_unescaped_pipes(r"a\|b|c");
        assert_eq!(parts, vec![r"a\|b", "c"]);

        let parts = split_unescaped_pipes(r"a\|b\|c");
        assert_eq!(parts, vec![r"a\|b\|c"]);

        let parts = split_unescaped_pipes("id|name(x:str):bool|method|logic:check(x)");
        assert_eq!(
            parts,
            vec!["id", "name(x:str):bool", "method", "logic:check(x)"]
        );
    }

    #[test]
    fn test_validate_logic_keywords_with_complex_conditions() {
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                "B_0002|methodB|method|logic:check(x>0 && y<10);action(save);return(result)"
                    .to_string(),
            ],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineLogic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_logic_keywords_empty_steps() {
        // Empty steps (e.g., from trailing semicolons) should be handled gracefully
        let graph = YcgGraphAdHoc {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec!["B_0002|methodB|method|logic:check(x>0);;return(result)".to_string()],
            adjacency: BTreeMap::new(),
        };

        let result = validate_adhoc_granularity(&graph, AdHocGranularity::InlineLogic);
        assert!(result.is_ok());
    }
}
