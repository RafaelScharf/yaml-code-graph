// crates/ycg_core/src/semantic_filter.rs

use crate::model::{ScipSymbolKind, SymbolNode, YcgGraph};
use regex::Regex;
use std::collections::HashSet;

/// Semantic filter for graph compaction
/// Removes low-significance nodes (local variables, anonymous blocks)
/// while preserving high-level architectural information
pub struct SemanticFilter;

impl SemanticFilter {
    /// Filter graph nodes based on semantic significance
    ///
    /// This removes:
    /// - Local variables (pattern: local_\d+_[a-f0-9]+)
    /// - Anonymous blocks
    /// - Private implementation details
    ///
    /// This preserves:
    /// - Exported symbols
    /// - Public methods
    /// - Interfaces
    /// - Exported functions
    /// - Classes
    pub fn filter_graph(graph: &mut YcgGraph) {
        // Collect IDs of nodes to remove
        let mut nodes_to_remove = HashSet::new();

        for node in &graph.definitions {
            if !Self::is_significant_symbol(node) {
                nodes_to_remove.insert(node.id.clone());
            }
        }

        // Remove insignificant nodes
        graph
            .definitions
            .retain(|node| !nodes_to_remove.contains(&node.id));

        // Remove edges that reference removed nodes
        graph.references.retain(|edge| {
            !nodes_to_remove.contains(&edge.from) && !nodes_to_remove.contains(&edge.to)
        });
    }

    /// Determine if a symbol is semantically significant
    ///
    /// A symbol is significant if it's:
    /// - A class, interface, or function
    /// - A method (public or exported)
    /// - Not a local variable
    /// - Not an anonymous block
    fn is_significant_symbol(node: &SymbolNode) -> bool {
        // Check if it's a local variable
        if Self::is_local_variable(&node.id, &node.name) {
            return false;
        }

        // Check if it's an anonymous block
        if Self::is_anonymous_block(&node.name) {
            return false;
        }

        // Keep significant types
        match node.kind {
            ScipSymbolKind::Class => true,
            ScipSymbolKind::Interface => true,
            ScipSymbolKind::Function => true,
            ScipSymbolKind::Method => true,
            ScipSymbolKind::Module => true,
            ScipSymbolKind::File => true,
            ScipSymbolKind::Variable => {
                // Keep variables that appear to be exported/module-level
                // Filter out local variables
                !Self::is_local_variable(&node.id, &node.name)
            }
        }
    }

    /// Check if a symbol is a local variable
    ///
    /// Local variables are detected by:
    /// - ID pattern: local_\d+_[a-f0-9]+
    /// - Name patterns indicating local scope
    fn is_local_variable(symbol_id: &str, symbol_name: &str) -> bool {
        // Pattern for local variables: local_\d+_[a-f0-9]+
        lazy_static::lazy_static! {
            static ref LOCAL_VAR_PATTERN: Regex = Regex::new(r"local_\d+_[a-f0-9]+").unwrap();
        }

        if LOCAL_VAR_PATTERN.is_match(symbol_id) {
            return true;
        }

        // Additional heuristics for local variables
        // Variables with names like "local_11_6d84" or similar patterns
        if symbol_name.starts_with("local_") {
            return true;
        }

        // Parameters in function signatures (contains "().(")
        if symbol_id.contains("().(") {
            return true;
        }

        false
    }

    /// Check if a symbol is an anonymous block
    ///
    /// Anonymous blocks are detected by:
    /// - Names like "unknown", "anonymous", or empty
    /// - Special markers in the ID
    fn is_anonymous_block(symbol_name: &str) -> bool {
        if symbol_name.is_empty() {
            return true;
        }

        // Check for anonymous/unknown markers
        let name_lower = symbol_name.to_lowercase();
        if name_lower == "unknown" || name_lower == "anonymous" {
            return true;
        }

        // Check for block markers
        if name_lower.starts_with("block_") || name_lower.starts_with("anon_") {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EdgeType, ProjectMetadata, ReferenceEdge};

    fn create_test_node(id: &str, name: &str, kind: ScipSymbolKind) -> SymbolNode {
        SymbolNode {
            id: id.to_string(),
            name: name.to_string(),
            kind,
            parent_id: None,
            documentation: None,
            signature: None,
            logic: None,
        }
    }

    #[test]
    fn test_is_local_variable() {
        // Test local variable pattern
        assert!(SemanticFilter::is_local_variable(
            "local_11_6d84",
            "local_11_6d84"
        ));
        assert!(SemanticFilter::is_local_variable("local_5_abc3", "temp"));

        // Test parameter pattern
        assert!(SemanticFilter::is_local_variable("func().(param)", "param"));

        // Test non-local variables
        assert!(!SemanticFilter::is_local_variable("User_01a2", "User"));
        assert!(!SemanticFilter::is_local_variable(
            "myFunction_3f4a",
            "myFunction"
        ));
    }

    #[test]
    fn test_is_anonymous_block() {
        assert!(SemanticFilter::is_anonymous_block(""));
        assert!(SemanticFilter::is_anonymous_block("unknown"));
        assert!(SemanticFilter::is_anonymous_block("anonymous"));
        assert!(SemanticFilter::is_anonymous_block("block_123"));
        assert!(SemanticFilter::is_anonymous_block("anon_456"));

        assert!(!SemanticFilter::is_anonymous_block("User"));
        assert!(!SemanticFilter::is_anonymous_block("myFunction"));
    }

    #[test]
    fn test_is_significant_symbol() {
        // Classes should be kept
        let class_node = create_test_node("User_01a2", "User", ScipSymbolKind::Class);
        assert!(SemanticFilter::is_significant_symbol(&class_node));

        // Interfaces should be kept
        let interface_node = create_test_node("IUser_02b3", "IUser", ScipSymbolKind::Interface);
        assert!(SemanticFilter::is_significant_symbol(&interface_node));

        // Functions should be kept
        let func_node = create_test_node("getUser_03c4", "getUser", ScipSymbolKind::Function);
        assert!(SemanticFilter::is_significant_symbol(&func_node));

        // Methods should be kept
        let method_node = create_test_node("save_04d5", "save", ScipSymbolKind::Method);
        assert!(SemanticFilter::is_significant_symbol(&method_node));

        // Local variables should be removed
        let local_var = create_test_node("local_11_6d84", "temp", ScipSymbolKind::Variable);
        assert!(!SemanticFilter::is_significant_symbol(&local_var));

        // Anonymous blocks should be removed
        let anon_block = create_test_node("block_123", "unknown", ScipSymbolKind::Variable);
        assert!(!SemanticFilter::is_significant_symbol(&anon_block));
    }

    #[test]
    fn test_filter_graph() {
        let mut graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                create_test_node("User_01a2", "User", ScipSymbolKind::Class),
                create_test_node("local_11_6d84", "temp", ScipSymbolKind::Variable),
                create_test_node("getUser_03c4", "getUser", ScipSymbolKind::Function),
                create_test_node("unknown", "unknown", ScipSymbolKind::Variable),
            ],
            references: vec![
                ReferenceEdge {
                    from: "User_01a2".to_string(),
                    to: "getUser_03c4".to_string(),
                    edge_type: EdgeType::Calls,
                },
                ReferenceEdge {
                    from: "getUser_03c4".to_string(),
                    to: "local_11_6d84".to_string(),
                    edge_type: EdgeType::Calls,
                },
                ReferenceEdge {
                    from: "local_11_6d84".to_string(),
                    to: "unknown".to_string(),
                    edge_type: EdgeType::Calls,
                },
            ],
        };

        SemanticFilter::filter_graph(&mut graph);

        // Should keep only significant nodes
        assert_eq!(graph.definitions.len(), 2);
        assert!(graph.definitions.iter().any(|n| n.id == "User_01a2"));
        assert!(graph.definitions.iter().any(|n| n.id == "getUser_03c4"));

        // Should remove edges referencing removed nodes
        assert_eq!(graph.references.len(), 1);
        assert_eq!(graph.references[0].from, "User_01a2");
        assert_eq!(graph.references[0].to, "getUser_03c4");
    }

    #[test]
    fn test_filter_preserves_module_level_variables() {
        let module_var = create_test_node("CONFIG_5a6b", "CONFIG", ScipSymbolKind::Variable);
        assert!(SemanticFilter::is_significant_symbol(&module_var));
    }
}
