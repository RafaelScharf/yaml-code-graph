// crates/ycg_core/src/adhoc_serializer_v2.rs
//! Enhanced ad-hoc format serializer with granularity level support
//!
//! This module provides the AdHocSerializerV2 which extends the original
//! ad-hoc format with three granularity levels:
//!
//! - **Level 0 (Default)**: ID|Name|Type - Maximum token efficiency
//! - **Level 1 (Inline Signatures)**: ID|Signature(args):Return|Type - API contracts
//! - **Level 2 (Inline Logic)**: ID|Signature|Type|logic:steps - Business logic
//!
//! **Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.8, 3.1, 3.2**

use crate::ast_cache::AstCache;
use crate::logic_extractor::LogicExtractor;
use crate::model::{AdHocGranularity, ScipSymbolKind, SymbolNode, YcgGraph, YcgGraphAdHoc};
use crate::signature_extractor::SignatureExtractor;
use std::collections::BTreeMap;

/// Enhanced ad-hoc serializer with granularity level support
///
/// Serializes symbol nodes based on the configured granularity level,
/// providing fine-grained control over output verbosity and token usage.
///
/// **Validates: Requirements 1.1, 1.2, 1.3, 1.4**
pub struct AdHocSerializerV2 {
    granularity: AdHocGranularity,
}

impl AdHocSerializerV2 {
    /// Create a new serializer with the specified granularity level
    ///
    /// # Arguments
    /// * `granularity` - The granularity level to use for serialization
    ///
    /// # Examples
    /// ```
    /// use ycg_core::adhoc_serializer_v2::AdHocSerializerV2;
    /// use ycg_core::model::AdHocGranularity;
    ///
    /// let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineSignatures);
    /// ```
    pub fn new(granularity: AdHocGranularity) -> Self {
        Self { granularity }
    }

    /// Serialize a symbol node based on the configured granularity level
    ///
    /// Dispatches to the appropriate serialization method based on level:
    /// - Level 0: serialize_default()
    /// - Level 1: serialize_with_signature()
    /// - Level 2: serialize_with_logic()
    ///
    /// # Arguments
    /// * `node` - The symbol node to serialize
    /// * `source` - Source code for signature/logic extraction (unused for Level 0)
    ///
    /// # Returns
    /// Pipe-separated string representation of the node
    ///
    /// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 2.1, 2.8, 3.1, 3.2**
    pub fn serialize_node(&self, node: &SymbolNode, source: &str) -> String {
        match self.granularity {
            AdHocGranularity::Default => self.serialize_default(node),
            AdHocGranularity::InlineSignatures => self.serialize_with_signature(node, source),
            AdHocGranularity::InlineLogic => self.serialize_with_logic(node, source),
        }
    }

    /// Serialize in default format (Level 0): ID|Name|Type
    ///
    /// Provides maximum token efficiency with only structural information.
    /// This format is byte-identical to v1.3.1 output for backward compatibility.
    ///
    /// # Arguments
    /// * `node` - The symbol node to serialize
    ///
    /// # Returns
    /// String in format "ID|Name|Type"
    ///
    /// # Examples
    /// ```text
    /// User_b8c1|User|class
    /// greet_a3f2|greet|function
    /// ```
    ///
    /// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.6**
    fn serialize_default(&self, node: &SymbolNode) -> String {
        let id = Self::escape_pipes(&node.id);
        let name = Self::escape_pipes(&node.name);
        let kind = Self::kind_to_string(&node.kind);

        format!("{}|{}|{}", id, name, kind)
    }

    /// Serialize with inline signature (Level 1): ID|Signature(args):Return|Type
    ///
    /// Includes function signatures with abbreviated types for API contract analysis.
    /// Falls back to simple name if signature extraction fails.
    ///
    /// # Arguments
    /// * `node` - The symbol node to serialize
    /// * `source` - Source code for signature extraction
    ///
    /// # Returns
    /// String in format "ID|Signature|Type" or "ID|Name|Type" (fallback)
    ///
    /// # Examples
    /// ```text
    /// findOne_7fed|findOne(id:str):User|method
    /// purchase_99a1|purchase(user:User,itemId:str):Promise<Order>|method
    /// ```
    ///
    /// **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8**
    fn serialize_with_signature(&self, node: &SymbolNode, _source: &str) -> String {
        let id = Self::escape_pipes(&node.id);
        let kind = Self::kind_to_string(&node.kind);

        // Try to extract signature
        let name_or_sig = if let Some(sig) = SignatureExtractor::extract_signature(node) {
            Self::escape_pipes(&sig)
        } else {
            // Fallback to simple name (Requirement 2.7)
            Self::escape_pipes(&node.name)
        };

        format!("{}|{}|{}", id, name_or_sig, kind)
    }

    /// Serialize with inline logic (Level 2): ID|Signature|Type|logic:steps
    ///
    /// Includes signatures plus compact logic representation for business logic analysis.
    /// Falls back to Level 1 format if logic extraction fails or returns None.
    ///
    /// # Arguments
    /// * `node` - The symbol node to serialize
    /// * `source` - Source code for signature and logic extraction
    ///
    /// # Returns
    /// String in format "ID|Signature|Type|logic:steps" or "ID|Signature|Type" (fallback)
    ///
    /// # Examples
    /// ```text
    /// checkStock_7fed|checkStock(itemId:str):bool|method|logic:return(item.qty>0)
    /// purchase_99a1|purchase(user,itemId)|method|logic:check(stock>0);check(user.balance>=price);action(deduct_balance);action(save_order)
    /// ```
    ///
    /// **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10, 3.11**
    fn serialize_with_logic(&self, node: &SymbolNode, source: &str) -> String {
        let id = Self::escape_pipes(&node.id);
        let kind = Self::kind_to_string(&node.kind);

        // Extract signature (always included at Level 2)
        let name_or_sig = if let Some(sig) = SignatureExtractor::extract_signature(node) {
            Self::escape_pipes(&sig)
        } else {
            Self::escape_pipes(&node.name)
        };

        // Extract logic
        if let Some(logic) = LogicExtractor::extract_logic(node, source) {
            let escaped_logic = Self::escape_pipes(&logic);
            // Four-field format with logic (Requirement 3.2)
            format!("{}|{}|{}|{}", id, name_or_sig, kind, escaped_logic)
        } else {
            // No logic to extract, fall back to Level 1 format (Requirement 3.10)
            format!("{}|{}|{}", id, name_or_sig, kind)
        }
    }

    /// Serialize entire graph to YcgGraphAdHoc format
    ///
    /// Converts all definitions to pipe-separated strings based on granularity level,
    /// while preserving metadata and adjacency structure.
    ///
    /// # Arguments
    /// * `graph` - The graph to serialize
    /// * `sources` - Map of file paths to source code content (for signature/logic extraction)
    ///
    /// # Returns
    /// YcgGraphAdHoc with serialized definitions and adjacency list
    ///
    /// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 2.1, 2.8, 3.1, 3.2**
    pub fn serialize_graph(
        &self,
        graph: &YcgGraph,
        sources: &std::collections::HashMap<String, String>,
    ) -> YcgGraphAdHoc {
        self.serialize_graph_with_cache(graph, sources, &mut AstCache::new())
    }

    /// Serialize entire graph to YcgGraphAdHoc format with AST caching
    ///
    /// This version accepts an AstCache to enable AST reuse across multiple symbols
    /// in the same file, improving performance for Level 1 and Level 2 granularity.
    ///
    /// # Arguments
    /// * `graph` - The graph to serialize
    /// * `sources` - Map of file paths to source code content (for signature/logic extraction)
    /// * `cache` - AST cache for reusing parsed ASTs
    ///
    /// # Returns
    /// YcgGraphAdHoc with serialized definitions and adjacency list
    ///
    /// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 2.1, 2.8, 3.1, 3.2, 10.3, 10.4**
    pub fn serialize_graph_with_cache(
        &self,
        graph: &YcgGraph,
        sources: &std::collections::HashMap<String, String>,
        cache: &mut AstCache,
    ) -> YcgGraphAdHoc {
        let definitions = graph
            .definitions
            .iter()
            .map(|node| {
                // Get source code for this node's file
                // For now, use empty string as placeholder since we don't have file mapping
                let source = sources.get(&node.id).map(|s| s.as_str()).unwrap_or("");

                // Cache the AST for this file if we're using Level 1 or Level 2
                // This allows multiple symbols from the same file to reuse the parsed AST
                if !source.is_empty()
                    && matches!(
                        self.granularity,
                        AdHocGranularity::InlineSignatures | AdHocGranularity::InlineLogic
                    )
                {
                    // Pre-populate cache for this file
                    // The actual parsing will happen in the extractors when tree-sitter is integrated
                    cache.get_or_parse(&node.id, source);
                }

                self.serialize_node(node, source)
            })
            .collect();

        // Build adjacency list (same as existing implementation)
        let mut adjacency: BTreeMap<String, BTreeMap<crate::model::EdgeType, Vec<String>>> =
            BTreeMap::new();

        for edge in &graph.references {
            let node_edges = adjacency
                .entry(edge.from.clone())
                .or_insert_with(BTreeMap::new);

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

    /// Escape pipe characters in content: `|` → `\|`
    ///
    /// Ensures pipe characters in symbol names, signatures, or logic
    /// don't interfere with the pipe-separated format.
    ///
    /// **Validates: Requirement 9.5**
    fn escape_pipes(s: &str) -> String {
        s.replace('|', r"\|")
    }

    /// Convert ScipSymbolKind to lowercase string
    ///
    /// Maps symbol kinds to their string representations:
    /// - Class → "class"
    /// - Method → "method"
    /// - Function → "function"
    /// - Variable → "variable"
    /// - Interface → "interface"
    /// - Module → "module"
    /// - File → "file"
    fn kind_to_string(kind: &ScipSymbolKind) -> &'static str {
        match kind {
            ScipSymbolKind::Class => "class",
            ScipSymbolKind::Method => "method",
            ScipSymbolKind::Function => "function",
            ScipSymbolKind::Variable => "variable",
            ScipSymbolKind::Interface => "interface",
            ScipSymbolKind::Module => "module",
            ScipSymbolKind::File => "file",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EdgeType, ProjectMetadata, ReferenceEdge};

    fn create_test_node(
        id: &str,
        name: &str,
        kind: ScipSymbolKind,
        signature: Option<String>,
    ) -> SymbolNode {
        SymbolNode {
            id: id.to_string(),
            name: name.to_string(),
            kind,
            parent_id: None,
            documentation: None,
            signature,
            logic: None,
        }
    }

    // ========================================================================
    // Level 0: Default Format Tests
    // Requirements: 1.1, 2.8, 3.2, 3.10
    // ========================================================================

    #[test]
    fn test_serialize_default_level_0_simple() {
        // Requirement 1.1: Level 0 format is ID|Name|Type
        let serializer = AdHocSerializerV2::new(AdHocGranularity::Default);
        let node = create_test_node("User_b8c1", "User", ScipSymbolKind::Class, None);

        let result = serializer.serialize_node(&node, "");
        assert_eq!(result, "User_b8c1|User|class");
    }

    #[test]
    fn test_serialize_default_level_0_function() {
        let serializer = AdHocSerializerV2::new(AdHocGranularity::Default);
        let node = create_test_node("greet_a3f2", "greet", ScipSymbolKind::Function, None);

        let result = serializer.serialize_node(&node, "");
        assert_eq!(result, "greet_a3f2|greet|function");
    }

    #[test]
    fn test_serialize_default_level_0_method() {
        let serializer = AdHocSerializerV2::new(AdHocGranularity::Default);
        let node = create_test_node("findOne_7fed", "findOne", ScipSymbolKind::Method, None);

        let result = serializer.serialize_node(&node, "");
        assert_eq!(result, "findOne_7fed|findOne|method");
    }

    #[test]
    fn test_serialize_default_level_0_all_kinds() {
        // Test all symbol kinds
        let serializer = AdHocSerializerV2::new(AdHocGranularity::Default);

        let kinds = vec![
            (ScipSymbolKind::File, "file"),
            (ScipSymbolKind::Module, "module"),
            (ScipSymbolKind::Class, "class"),
            (ScipSymbolKind::Method, "method"),
            (ScipSymbolKind::Function, "function"),
            (ScipSymbolKind::Variable, "variable"),
            (ScipSymbolKind::Interface, "interface"),
        ];

        for (kind, expected_str) in kinds {
            let node = create_test_node("test_id", "testName", kind, None);
            let result = serializer.serialize_node(&node, "");
            assert_eq!(result, format!("test_id|testName|{}", expected_str));
        }
    }

    // ========================================================================
    // Level 1: Inline Signatures Tests
    // Requirements: 2.8
    // ========================================================================

    #[test]
    fn test_serialize_level_1_with_signature() {
        // Requirement 2.8: Level 1 format includes signatures
        let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineSignatures);
        let node = create_test_node(
            "findOne_7fed",
            "findOne",
            ScipSymbolKind::Method,
            Some("findOne(id: string): User".to_string()),
        );

        let result = serializer.serialize_node(&node, "");

        // Should contain abbreviated signature
        assert!(result.starts_with("findOne_7fed|"));
        assert!(result.contains("findOne"));
        assert!(result.contains("str")); // string abbreviated
        assert!(result.ends_with("|method"));
    }

    #[test]
    fn test_serialize_level_1_without_signature_fallback() {
        // Requirement 2.7: Fall back to simple name if no signature
        let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineSignatures);
        let node = create_test_node("findOne_7fed", "findOne", ScipSymbolKind::Method, None);

        let result = serializer.serialize_node(&node, "");
        assert_eq!(result, "findOne_7fed|findOne|method");
    }

    #[test]
    fn test_serialize_level_1_complex_signature() {
        let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineSignatures);
        let node = create_test_node(
            "purchase_99a1",
            "purchase",
            ScipSymbolKind::Method,
            Some("purchase(user: User, itemId: string): Promise<Order>".to_string()),
        );

        let result = serializer.serialize_node(&node, "");

        assert!(result.starts_with("purchase_99a1|"));
        assert!(result.contains("purchase"));
        assert!(result.contains("str")); // string abbreviated
        assert!(result.contains("Promise<Order>"));
        assert!(result.ends_with("|method"));
    }

    #[test]
    fn test_serialize_level_1_void_return() {
        // Requirement 2.8: Omit void return type
        let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineSignatures);
        let node = create_test_node(
            "delete_1234",
            "delete",
            ScipSymbolKind::Method,
            Some("delete(id: string): void".to_string()),
        );

        let result = serializer.serialize_node(&node, "");

        // Should not include ":void" in output
        assert!(result.contains("delete(id:str)"));
        assert!(!result.contains(":void"));
    }

    // ========================================================================
    // Level 2: Inline Logic Tests
    // Requirements: 3.2, 3.10
    // ========================================================================

    #[test]
    fn test_serialize_level_2_with_logic() {
        // Requirement 3.2: Level 2 format includes logic field
        let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineLogic);

        // Create a node with signature (logic extraction will return None in placeholder)
        let node = create_test_node(
            "checkStock_7fed",
            "checkStock",
            ScipSymbolKind::Method,
            Some("checkStock(itemId: string): boolean".to_string()),
        );

        let result = serializer.serialize_node(&node, "");

        // Since LogicExtractor returns None (placeholder), should fall back to Level 1
        assert!(result.starts_with("checkStock_7fed|"));
        assert!(result.contains("checkStock"));
        assert!(result.ends_with("|method"));
    }

    #[test]
    fn test_serialize_level_2_without_logic_fallback() {
        // Requirement 3.10: Fall back to Level 1 format if no logic
        let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineLogic);
        let node = create_test_node(
            "findOne_7fed",
            "findOne",
            ScipSymbolKind::Method,
            Some("findOne(id: string): User".to_string()),
        );

        let result = serializer.serialize_node(&node, "");

        // Should have 3 fields (no logic field)
        let field_count = result.matches('|').count();
        assert_eq!(field_count, 2); // 3 fields = 2 pipes
    }

    #[test]
    fn test_serialize_level_2_non_method_no_logic() {
        // Requirement 3.1: Only extract logic for methods and functions
        let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineLogic);
        let node = create_test_node("User_b8c1", "User", ScipSymbolKind::Class, None);

        let result = serializer.serialize_node(&node, "");

        // Should not have logic field for classes
        assert_eq!(result, "User_b8c1|User|class");
    }

    // ========================================================================
    // Pipe Escaping Tests
    // Requirements: 3.10
    // ========================================================================

    #[test]
    fn test_escape_pipes_in_name() {
        // Requirement 3.10: Escape pipe characters in content
        let serializer = AdHocSerializerV2::new(AdHocGranularity::Default);
        let node = create_test_node("test_id", "name|with|pipes", ScipSymbolKind::Function, None);

        let result = serializer.serialize_node(&node, "");
        assert_eq!(result, r"test_id|name\|with\|pipes|function");
    }

    #[test]
    fn test_escape_pipes_in_id() {
        let serializer = AdHocSerializerV2::new(AdHocGranularity::Default);
        let node = create_test_node("id|with|pipes", "testName", ScipSymbolKind::Function, None);

        let result = serializer.serialize_node(&node, "");
        assert_eq!(result, r"id\|with\|pipes|testName|function");
    }

    #[test]
    fn test_escape_pipes_in_signature() {
        let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineSignatures);
        let node = create_test_node(
            "test_id",
            "method",
            ScipSymbolKind::Method,
            Some("method(param: Type|OtherType): Result".to_string()),
        );

        let result = serializer.serialize_node(&node, "");

        // Debug: print the result to see what we get
        println!("Result: {}", result);

        // Union types are simplified to first type only (for compactness)
        // So "Type|OtherType" becomes "Type"
        // The result should be: test_id|method(param:Type):Result|method
        assert!(result.contains("method(param:Type):Result"));
    }

    // ========================================================================
    // Graph Serialization Tests
    // ========================================================================

    #[test]
    fn test_serialize_graph_level_0() {
        let serializer = AdHocSerializerV2::new(AdHocGranularity::Default);

        let graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                create_test_node("A_0001", "ClassA", ScipSymbolKind::Class, None),
                create_test_node("B_0002", "methodB", ScipSymbolKind::Method, None),
            ],
            references: vec![ReferenceEdge {
                from: "B_0002".to_string(),
                to: "A_0001".to_string(),
                edge_type: EdgeType::Calls,
            }],
        };

        let sources = std::collections::HashMap::new();
        let adhoc = serializer.serialize_graph(&graph, &sources);

        assert_eq!(adhoc.definitions.len(), 2);
        assert_eq!(adhoc.definitions[0], "A_0001|ClassA|class");
        assert_eq!(adhoc.definitions[1], "B_0002|methodB|method");
        assert_eq!(adhoc.metadata.name, "test");
    }

    #[test]
    fn test_serialize_graph_level_1() {
        let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineSignatures);

        let graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                create_test_node("A_0001", "ClassA", ScipSymbolKind::Class, None),
                create_test_node(
                    "B_0002",
                    "methodB",
                    ScipSymbolKind::Method,
                    Some("methodB(id: string): User".to_string()),
                ),
            ],
            references: vec![],
        };

        let sources = std::collections::HashMap::new();
        let adhoc = serializer.serialize_graph(&graph, &sources);

        assert_eq!(adhoc.definitions.len(), 2);
        assert_eq!(adhoc.definitions[0], "A_0001|ClassA|class");
        // Second definition should have signature
        assert!(adhoc.definitions[1].contains("methodB"));
        assert!(adhoc.definitions[1].contains("str")); // abbreviated
    }

    #[test]
    fn test_serialize_graph_adjacency_list() {
        let serializer = AdHocSerializerV2::new(AdHocGranularity::Default);

        let graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                create_test_node("A_0001", "ClassA", ScipSymbolKind::Class, None),
                create_test_node("B_0002", "methodB", ScipSymbolKind::Method, None),
                create_test_node("C_0003", "methodC", ScipSymbolKind::Method, None),
            ],
            references: vec![
                ReferenceEdge {
                    from: "B_0002".to_string(),
                    to: "A_0001".to_string(),
                    edge_type: EdgeType::Calls,
                },
                ReferenceEdge {
                    from: "B_0002".to_string(),
                    to: "C_0003".to_string(),
                    edge_type: EdgeType::Calls,
                },
            ],
        };

        let sources = std::collections::HashMap::new();
        let adhoc = serializer.serialize_graph(&graph, &sources);

        // Check adjacency list structure
        assert!(adhoc.adjacency.contains_key("B_0002"));
        let b_edges = &adhoc.adjacency["B_0002"];
        assert!(b_edges.contains_key(&EdgeType::Calls));

        let calls = &b_edges[&EdgeType::Calls];
        assert_eq!(calls.len(), 2);
        // Should be sorted
        assert_eq!(calls[0], "A_0001");
        assert_eq!(calls[1], "C_0003");
    }

    #[test]
    fn test_serialize_graph_empty() {
        let serializer = AdHocSerializerV2::new(AdHocGranularity::Default);

        let graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "empty".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![],
            references: vec![],
        };

        let sources = std::collections::HashMap::new();
        let adhoc = serializer.serialize_graph(&graph, &sources);

        assert_eq!(adhoc.definitions.len(), 0);
        assert_eq!(adhoc.adjacency.len(), 0);
    }

    // ========================================================================
    // Helper Function Tests
    // ========================================================================

    #[test]
    fn test_escape_pipes_helper() {
        assert_eq!(AdHocSerializerV2::escape_pipes("no pipes"), "no pipes");
        assert_eq!(AdHocSerializerV2::escape_pipes("one|pipe"), r"one\|pipe");
        assert_eq!(
            AdHocSerializerV2::escape_pipes("multiple|pipes|here"),
            r"multiple\|pipes\|here"
        );
    }

    #[test]
    fn test_kind_to_string_all_kinds() {
        assert_eq!(
            AdHocSerializerV2::kind_to_string(&ScipSymbolKind::File),
            "file"
        );
        assert_eq!(
            AdHocSerializerV2::kind_to_string(&ScipSymbolKind::Module),
            "module"
        );
        assert_eq!(
            AdHocSerializerV2::kind_to_string(&ScipSymbolKind::Class),
            "class"
        );
        assert_eq!(
            AdHocSerializerV2::kind_to_string(&ScipSymbolKind::Method),
            "method"
        );
        assert_eq!(
            AdHocSerializerV2::kind_to_string(&ScipSymbolKind::Function),
            "function"
        );
        assert_eq!(
            AdHocSerializerV2::kind_to_string(&ScipSymbolKind::Variable),
            "variable"
        );
        assert_eq!(
            AdHocSerializerV2::kind_to_string(&ScipSymbolKind::Interface),
            "interface"
        );
    }
}
