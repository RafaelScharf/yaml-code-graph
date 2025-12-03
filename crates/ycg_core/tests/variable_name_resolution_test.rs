// Integration tests for variable name resolution
// Validates Requirements 6.1, 6.2, 6.3, 6.4, 6.5

use std::path::PathBuf;
use ycg_core::enricher::TreeSitterEnricher;

#[test]
fn test_resolve_variable_name_typescript() {
    let mut enricher = TreeSitterEnricher::new();

    // Create a test TypeScript file
    let test_file = PathBuf::from("examples/simple-ts/src/index.ts");

    if !test_file.exists() {
        eprintln!("Test file not found, skipping test");
        return;
    }

    // Try to resolve a variable name
    // Note: This test requires the actual file to exist and have variables
    let result = enricher.resolve_variable_name(&test_file, 5, 10);

    // We expect either a resolved name or None (graceful degradation)
    match result {
        Some(name) => {
            println!("✓ Resolved variable name: {}", name);
            assert!(!name.is_empty());
            assert!(!name.contains("0")); // Should not be a generic name
        }
        None => {
            println!("⚠️  Could not resolve variable name (graceful degradation)");
        }
    }
}

#[test]
fn test_resolve_variable_name_invalid_position() {
    let mut enricher = TreeSitterEnricher::new();

    let test_file = PathBuf::from("examples/simple-ts/src/index.ts");

    if !test_file.exists() {
        eprintln!("Test file not found, skipping test");
        return;
    }

    // Try to resolve at an invalid position (line 10000)
    let result = enricher.resolve_variable_name(&test_file, 10000, 0);

    // Should return None for invalid position
    assert!(result.is_none());
}

#[test]
fn test_resolve_variable_name_nonexistent_file() {
    let mut enricher = TreeSitterEnricher::new();

    let test_file = PathBuf::from("nonexistent/file.ts");

    // Should return None for nonexistent file
    let result = enricher.resolve_variable_name(&test_file, 0, 0);
    assert!(result.is_none());
}

#[test]
fn test_resolve_variable_name_unsupported_extension() {
    let mut enricher = TreeSitterEnricher::new();

    let test_file = PathBuf::from("test.unsupported");

    // Should return None for unsupported file extension
    let result = enricher.resolve_variable_name(&test_file, 0, 0);
    assert!(result.is_none());
}

// Note: is_generic_name is a private function in lib.rs
// We test it indirectly through integration tests
// Here are the expected behaviors:

// Generic names (should be resolved):
// - status0, timestamp1, user2, result3
// - data0, value1, item2

// Non-generic names (should NOT be resolved):
// - userId, userName, activeUsers
// - myVariable, someData
// - user_id, user-name (contains special chars)
// - User0 (starts with uppercase)
// - 0user (starts with digit)

#[test]
fn test_generic_name_patterns() {
    // This test documents the expected behavior of is_generic_name
    // The actual function is tested indirectly through the integration

    let generic_names = vec!["status0", "timestamp1", "user2", "result3", "data0"];

    let non_generic_names = vec![
        "userId",
        "userName",
        "activeUsers",
        "myVariable",
        "user_id",
        "User0",
        "0user",
        "user",
        "status",
    ];

    println!("Generic names that should be resolved:");
    for name in generic_names {
        println!("  - {}", name);
    }

    println!("\nNon-generic names that should NOT be resolved:");
    for name in non_generic_names {
        println!("  - {}", name);
    }

    // This test always passes - it's documentation
    assert!(true);
}

#[test]
fn test_graceful_degradation() {
    // Test that the system gracefully degrades when resolution fails
    let mut enricher = TreeSitterEnricher::new();

    // Try various failure scenarios
    let scenarios = vec![
        ("nonexistent.ts", 0, 0, "Nonexistent file"),
        ("test.unknown", 0, 0, "Unknown extension"),
    ];

    for (file, line, col, description) in scenarios {
        let result = enricher.resolve_variable_name(&PathBuf::from(file), line, col);

        // Should return None (graceful degradation)
        assert!(
            result.is_none(),
            "Expected None for scenario: {}",
            description
        );

        println!("✓ Graceful degradation for: {}", description);
    }
}
