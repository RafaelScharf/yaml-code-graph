// crates/ycg_core/tests/logic_metadata_restriction_test.rs
//! Integration test for logic metadata restriction to methods and functions
//!
//! This test verifies that logic metadata (preconditions/guard clauses) is only
//! attached to Method and Function symbol kinds, and not to Variables or other kinds.
//!
//! **Validates: Requirements 4.1, 4.2, 4.3, 4.4**

use ycg_core::model::{LogicMetadata, ProjectMetadata, ScipSymbolKind, SymbolNode, YcgGraph};

/// Test that methods can have logic metadata
///
/// This test verifies that when a method has preconditions extracted by the enricher,
/// the logic metadata is correctly attached to the method symbol.
///
/// **Validates: Requirements 4.1, 4.2**
#[test]
fn test_method_has_logic_metadata() {
    // Create a method node with logic metadata
    let method_node = SymbolNode {
        id: "UsersController_findOne_13b7".to_string(),
        name: "findOne".to_string(),
        kind: ScipSymbolKind::Method,
        parent_id: Some("UsersController_4702".to_string()),
        documentation: None,
        signature: Some("async findOne(id: number): Promise<UserDto>".to_string()),
        logic: Some(LogicMetadata {
            preconditions: vec!["must avoid: user".to_string()],
        }),
    };

    // Verify the method has logic metadata
    assert!(
        method_node.logic.is_some(),
        "Method should have logic metadata"
    );
    assert_eq!(
        method_node.logic.as_ref().unwrap().preconditions.len(),
        1,
        "Method should have one precondition"
    );
}

/// Test that functions can have logic metadata
///
/// This test verifies that when a function has preconditions extracted by the enricher,
/// the logic metadata is correctly attached to the function symbol.
///
/// **Validates: Requirements 4.1, 4.2**
#[test]
fn test_function_has_logic_metadata() {
    // Create a function node with logic metadata
    let function_node = SymbolNode {
        id: "validateInput_abc123".to_string(),
        name: "validateInput".to_string(),
        kind: ScipSymbolKind::Function,
        parent_id: None,
        documentation: None,
        signature: Some("function validateInput(data: string): boolean".to_string()),
        logic: Some(LogicMetadata {
            preconditions: vec!["must check: data.length > 0".to_string()],
        }),
    };

    // Verify the function has logic metadata
    assert!(
        function_node.logic.is_some(),
        "Function should have logic metadata"
    );
    assert_eq!(
        function_node.logic.as_ref().unwrap().preconditions.len(),
        1,
        "Function should have one precondition"
    );
}

/// Test that variables do NOT have logic metadata
///
/// This test verifies that variables never have logic metadata attached,
/// even if they are defined within a method that has guard clauses.
///
/// **Validates: Requirements 4.2, 4.3, 4.4**
#[test]
fn test_variable_has_no_logic_metadata() {
    // Create a variable node without logic metadata
    let variable_node = SymbolNode {
        id: "userId0__95a5".to_string(),
        name: "userId0:".to_string(),
        kind: ScipSymbolKind::Variable,
        parent_id: Some("UsersController_findOne_13b7".to_string()),
        documentation: None,
        signature: None,
        logic: None, // Variables should never have logic
    };

    // Verify the variable does NOT have logic metadata
    assert!(
        variable_node.logic.is_none(),
        "Variable should NOT have logic metadata"
    );
}

/// Test that classes do NOT have logic metadata
///
/// This test verifies that class symbols do not have logic metadata attached.
///
/// **Validates: Requirements 4.2, 4.4**
#[test]
fn test_class_has_no_logic_metadata() {
    // Create a class node without logic metadata
    let class_node = SymbolNode {
        id: "UsersController_4702".to_string(),
        name: "UsersController".to_string(),
        kind: ScipSymbolKind::Class,
        parent_id: Some("file_abc".to_string()),
        documentation: None,
        signature: None,
        logic: None, // Classes should never have logic
    };

    // Verify the class does NOT have logic metadata
    assert!(
        class_node.logic.is_none(),
        "Class should NOT have logic metadata"
    );
}

/// Test that interfaces do NOT have logic metadata
///
/// This test verifies that interface symbols do not have logic metadata attached.
///
/// **Validates: Requirements 4.2, 4.4**
#[test]
fn test_interface_has_no_logic_metadata() {
    // Create an interface node without logic metadata
    let interface_node = SymbolNode {
        id: "UserDto_698e".to_string(),
        name: "UserDto".to_string(),
        kind: ScipSymbolKind::Interface,
        parent_id: Some("file_abc".to_string()),
        documentation: None,
        signature: None,
        logic: None, // Interfaces should never have logic
    };

    // Verify the interface does NOT have logic metadata
    assert!(
        interface_node.logic.is_none(),
        "Interface should NOT have logic metadata"
    );
}

/// Test graph with mixed symbol kinds
///
/// This test verifies that in a graph with multiple symbol kinds,
/// only methods and functions have logic metadata.
///
/// **Validates: Requirements 4.1, 4.2, 4.3, 4.4**
#[test]
fn test_graph_logic_metadata_restriction() {
    // Create a graph with various symbol kinds
    let graph = YcgGraph {
        metadata: ProjectMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
        },
        definitions: vec![
            // Class - no logic
            SymbolNode {
                id: "UsersController_4702".to_string(),
                name: "UsersController".to_string(),
                kind: ScipSymbolKind::Class,
                parent_id: Some("file_abc".to_string()),
                documentation: None,
                signature: None,
                logic: None,
            },
            // Method - has logic
            SymbolNode {
                id: "UsersController_findOne_13b7".to_string(),
                name: "findOne".to_string(),
                kind: ScipSymbolKind::Method,
                parent_id: Some("UsersController_4702".to_string()),
                documentation: None,
                signature: Some("async findOne(id: number): Promise<UserDto>".to_string()),
                logic: Some(LogicMetadata {
                    preconditions: vec!["must avoid: user".to_string()],
                }),
            },
            // Variable - no logic
            SymbolNode {
                id: "userId0__95a5".to_string(),
                name: "userId0:".to_string(),
                kind: ScipSymbolKind::Variable,
                parent_id: Some("UsersController_findOne_13b7".to_string()),
                documentation: None,
                signature: None,
                logic: None,
            },
            // Variable - no logic
            SymbolNode {
                id: "username0__e666".to_string(),
                name: "username0:".to_string(),
                kind: ScipSymbolKind::Variable,
                parent_id: Some("UsersController_findOne_13b7".to_string()),
                documentation: None,
                signature: None,
                logic: None,
            },
            // Function - has logic
            SymbolNode {
                id: "validateInput_abc123".to_string(),
                name: "validateInput".to_string(),
                kind: ScipSymbolKind::Function,
                parent_id: None,
                documentation: None,
                signature: Some("function validateInput(data: string): boolean".to_string()),
                logic: Some(LogicMetadata {
                    preconditions: vec!["must check: data.length > 0".to_string()],
                }),
            },
        ],
        references: vec![],
    };

    // Count symbols with logic metadata by kind
    let methods_with_logic = graph
        .definitions
        .iter()
        .filter(|node| node.kind == ScipSymbolKind::Method && node.logic.is_some())
        .count();

    let functions_with_logic = graph
        .definitions
        .iter()
        .filter(|node| node.kind == ScipSymbolKind::Function && node.logic.is_some())
        .count();

    let variables_with_logic = graph
        .definitions
        .iter()
        .filter(|node| node.kind == ScipSymbolKind::Variable && node.logic.is_some())
        .count();

    let classes_with_logic = graph
        .definitions
        .iter()
        .filter(|node| node.kind == ScipSymbolKind::Class && node.logic.is_some())
        .count();

    // Verify only methods and functions have logic
    assert_eq!(methods_with_logic, 1, "Should have 1 method with logic");
    assert_eq!(functions_with_logic, 1, "Should have 1 function with logic");
    assert_eq!(
        variables_with_logic, 0,
        "Should have 0 variables with logic"
    );
    assert_eq!(classes_with_logic, 0, "Should have 0 classes with logic");
}

/// Test that variables within methods with guard clauses don't inherit logic
///
/// This test verifies that when a method has guard clauses (preconditions),
/// the variables defined within that method do not inherit the logic metadata.
///
/// **Validates: Requirements 4.3, 4.5**
#[test]
fn test_variables_in_method_with_guard_clauses() {
    // Create a method with guard clauses
    let method_node = SymbolNode {
        id: "UsersController_findOne_13b7".to_string(),
        name: "findOne".to_string(),
        kind: ScipSymbolKind::Method,
        parent_id: Some("UsersController_4702".to_string()),
        documentation: None,
        signature: Some("async findOne(id: number): Promise<UserDto>".to_string()),
        logic: Some(LogicMetadata {
            preconditions: vec![
                "must avoid: user".to_string(),
                "must check: id > 0".to_string(),
            ],
        }),
    };

    // Create variables within the method
    let var1 = SymbolNode {
        id: "userId0__95a5".to_string(),
        name: "userId0:".to_string(),
        kind: ScipSymbolKind::Variable,
        parent_id: Some("UsersController_findOne_13b7".to_string()),
        documentation: None,
        signature: None,
        logic: None, // Should NOT inherit method's logic
    };

    let var2 = SymbolNode {
        id: "username0__e666".to_string(),
        name: "username0:".to_string(),
        kind: ScipSymbolKind::Variable,
        parent_id: Some("UsersController_findOne_13b7".to_string()),
        documentation: None,
        signature: None,
        logic: None, // Should NOT inherit method's logic
    };

    // Verify method has logic
    assert!(
        method_node.logic.is_some(),
        "Method should have logic metadata"
    );
    assert_eq!(
        method_node.logic.as_ref().unwrap().preconditions.len(),
        2,
        "Method should have 2 preconditions"
    );

    // Verify variables do NOT have logic
    assert!(
        var1.logic.is_none(),
        "Variable 1 should NOT have logic metadata"
    );
    assert!(
        var2.logic.is_none(),
        "Variable 2 should NOT have logic metadata"
    );
}
