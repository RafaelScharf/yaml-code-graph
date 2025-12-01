// crates/ycg_core/src/framework_filter.rs

use crate::model::{ScipSymbolKind, SymbolNode, YcgGraph};
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

/// Framework noise filter for removing boilerplate patterns
/// Targets framework-specific patterns like NestJS and TypeORM
pub struct FrameworkNoiseFilter;

impl FrameworkNoiseFilter {
    /// Apply framework-specific noise reduction to the graph
    ///
    /// This removes:
    /// - Dependency injection only constructors
    /// - Decorator metadata from DTO properties
    ///
    /// This preserves:
    /// - Property names and types in DTOs
    /// - All non-boilerplate code elements
    pub fn filter_graph(graph: &mut YcgGraph) {
        let mut nodes_to_remove = HashSet::new();

        for node in &mut graph.definitions {
            // Check if this is a DI-only constructor
            if node.kind == ScipSymbolKind::Method
                && node.name == "constructor"
                && Self::is_di_only_constructor(node)
            {
                nodes_to_remove.insert(node.id.clone());
                continue;
            }

            // Simplify DTO properties
            if node.parent_id.is_some() {
                // Check if parent is in a DTO file by looking at the node's signature or documentation
                // We'll use a heuristic: if the signature contains decorators, it's likely a DTO property
                if node.signature.is_some() {
                    Self::simplify_dto_property(node);
                }
            }
        }

        // Remove DI-only constructors
        graph
            .definitions
            .retain(|node| !nodes_to_remove.contains(&node.id));

        // Remove edges that reference removed nodes
        graph.references.retain(|edge| {
            !nodes_to_remove.contains(&edge.from) && !nodes_to_remove.contains(&edge.to)
        });
    }

    /// Detect if a constructor only performs dependency injection assignments
    ///
    /// A DI-only constructor has a signature that:
    /// - Contains only parameter declarations with access modifiers (private, public, protected)
    /// - Has no body logic beyond assignments
    /// - Matches pattern: constructor(private x: Type, public y: Type)
    fn is_di_only_constructor(node: &SymbolNode) -> bool {
        if node.name != "constructor" {
            return false;
        }

        let signature = match &node.signature {
            Some(sig) => sig,
            None => return false,
        };

        // Pattern for DI constructor: constructor with access modifiers in parameters
        // Example: constructor(private userService: UserService, public config: Config)
        lazy_static::lazy_static! {
            static ref DI_CONSTRUCTOR_PATTERN: Regex = Regex::new(
                r"constructor\s*\([^)]*\b(private|public|protected|readonly)\s+\w+"
            ).unwrap();
        }

        // Check if signature matches DI pattern
        if !DI_CONSTRUCTOR_PATTERN.is_match(signature) {
            return false;
        }

        // Additional check: if there's a body with only simple assignments, it's DI-only
        // Look for patterns like "this.x = x;" in the signature or documentation
        // If the signature is just the parameter list with access modifiers, it's DI-only

        // Check if signature contains only parameter declarations (no complex logic)
        // A DI-only constructor typically has a very simple signature
        let has_complex_logic = signature.contains('{')
            && (signature.contains("if ")
                || signature.contains("for ")
                || signature.contains("while ")
                || signature.contains("switch ")
                || signature.contains("return ")
                || signature.contains("throw ")
                || signature.contains("await ")
                || signature.contains("=>"));

        !has_complex_logic
    }

    /// Detect if a file is a DTO based on its path
    ///
    /// A file is considered a DTO if:
    /// - Path contains "/dto/"
    /// - Filename ends with ".dto.ts"
    pub fn is_dto_file(file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();

        // Check if path contains /dto/
        if path_str.contains("/dto/") {
            return true;
        }

        // Check if filename ends with .dto.ts
        if let Some(filename) = file_path.file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.ends_with(".dto.ts") {
                return true;
            }
        }

        false
    }

    /// Remove decorators from a signature string
    ///
    /// Removes patterns like:
    /// - @ApiProperty()
    /// - @IsString()
    /// - @IsOptional()
    /// - @Column({ type: 'varchar' })
    /// - Any @Decorator(...) pattern
    pub fn strip_decorators(signature: &str) -> String {
        lazy_static::lazy_static! {
            // Pattern to match decorators: @DecoratorName(...) or @DecoratorName
            static ref DECORATOR_PATTERN: Regex = Regex::new(
                r"@\w+(\([^)]*\))?\s*"
            ).unwrap();
        }

        DECORATOR_PATTERN
            .replace_all(signature, "")
            .trim()
            .to_string()
    }

    /// Simplify DTO property definitions by removing decorator metadata
    ///
    /// This preserves:
    /// - Property name
    /// - Property type
    ///
    /// This removes:
    /// - All decorator metadata (@ApiProperty, @IsString, etc.)
    pub fn simplify_dto_property(node: &mut SymbolNode) {
        if let Some(signature) = &node.signature {
            // Check if signature contains decorators
            if signature.contains('@') {
                let simplified = Self::strip_decorators(signature);

                // Only update if we actually removed something
                if simplified != *signature {
                    node.signature = Some(simplified);
                }
            }
        }

        // Also strip decorators from documentation if present
        if let Some(doc) = &node.documentation {
            if doc.contains('@') {
                let simplified = Self::strip_decorators(doc);
                if simplified != *doc {
                    node.documentation = Some(simplified);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EdgeType, ProjectMetadata, ReferenceEdge};
    use std::path::PathBuf;

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

    #[test]
    fn test_is_dto_file() {
        // Test path with /dto/
        assert!(FrameworkNoiseFilter::is_dto_file(&PathBuf::from(
            "src/users/dto/user.dto.ts"
        )));
        assert!(FrameworkNoiseFilter::is_dto_file(&PathBuf::from(
            "src/dto/create-user.ts"
        )));

        // Test filename ending with .dto.ts
        assert!(FrameworkNoiseFilter::is_dto_file(&PathBuf::from(
            "src/users/user.dto.ts"
        )));

        // Test non-DTO files
        assert!(!FrameworkNoiseFilter::is_dto_file(&PathBuf::from(
            "src/users/user.service.ts"
        )));
        assert!(!FrameworkNoiseFilter::is_dto_file(&PathBuf::from(
            "src/users/user.controller.ts"
        )));
    }

    #[test]
    fn test_strip_decorators() {
        // Test single decorator
        let input = "@ApiProperty() name: string";
        let expected = "name: string";
        assert_eq!(FrameworkNoiseFilter::strip_decorators(input), expected);

        // Test multiple decorators
        let input = "@IsString() @IsOptional() email?: string";
        let expected = "email?: string";
        assert_eq!(FrameworkNoiseFilter::strip_decorators(input), expected);

        // Test decorator with arguments
        let input = "@Column({ type: 'varchar', length: 255 }) username: string";
        let expected = "username: string";
        assert_eq!(FrameworkNoiseFilter::strip_decorators(input), expected);

        // Test multiple decorators on multiple lines
        let input = "@ApiProperty()\n@IsString()\nname: string";
        let expected = "name: string";
        assert_eq!(FrameworkNoiseFilter::strip_decorators(input), expected);

        // Test no decorators
        let input = "name: string";
        assert_eq!(FrameworkNoiseFilter::strip_decorators(input), input);
    }

    #[test]
    fn test_is_di_only_constructor() {
        // Test DI-only constructor with private
        let node = create_test_node(
            "ctor_1",
            "constructor",
            ScipSymbolKind::Method,
            Some("constructor(private userService: UserService)".to_string()),
        );
        assert!(FrameworkNoiseFilter::is_di_only_constructor(&node));

        // Test DI-only constructor with multiple parameters
        let node = create_test_node(
            "ctor_2",
            "constructor",
            ScipSymbolKind::Method,
            Some(
                "constructor(private userService: UserService, public config: Config)".to_string(),
            ),
        );
        assert!(FrameworkNoiseFilter::is_di_only_constructor(&node));

        // Test DI-only constructor with readonly
        let node = create_test_node(
            "ctor_3",
            "constructor",
            ScipSymbolKind::Method,
            Some("constructor(private readonly logger: Logger)".to_string()),
        );
        assert!(FrameworkNoiseFilter::is_di_only_constructor(&node));

        // Test constructor with complex logic (should not be DI-only)
        let node = create_test_node(
            "ctor_4",
            "constructor",
            ScipSymbolKind::Method,
            Some(
                "constructor(private service: Service) { if (condition) { doSomething(); } }"
                    .to_string(),
            ),
        );
        assert!(!FrameworkNoiseFilter::is_di_only_constructor(&node));

        // Test regular constructor without access modifiers
        let node = create_test_node(
            "ctor_5",
            "constructor",
            ScipSymbolKind::Method,
            Some("constructor(name: string, age: number)".to_string()),
        );
        assert!(!FrameworkNoiseFilter::is_di_only_constructor(&node));

        // Test non-constructor method
        let node = create_test_node(
            "method_1",
            "save",
            ScipSymbolKind::Method,
            Some("save(data: any)".to_string()),
        );
        assert!(!FrameworkNoiseFilter::is_di_only_constructor(&node));
    }

    #[test]
    fn test_simplify_dto_property() {
        let mut node = create_test_node(
            "prop_1",
            "name",
            ScipSymbolKind::Variable,
            Some("@ApiProperty() @IsString() name: string".to_string()),
        );

        FrameworkNoiseFilter::simplify_dto_property(&mut node);

        assert_eq!(node.signature, Some("name: string".to_string()));
    }

    #[test]
    fn test_filter_graph_removes_di_constructors() {
        let mut graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                create_test_node("User_01a2", "User", ScipSymbolKind::Class, None),
                create_test_node(
                    "ctor_1",
                    "constructor",
                    ScipSymbolKind::Method,
                    Some("constructor(private userService: UserService)".to_string()),
                ),
                create_test_node(
                    "save_03c4",
                    "save",
                    ScipSymbolKind::Method,
                    Some("save(data: any)".to_string()),
                ),
            ],
            references: vec![
                ReferenceEdge {
                    from: "User_01a2".to_string(),
                    to: "ctor_1".to_string(),
                    edge_type: EdgeType::Calls,
                },
                ReferenceEdge {
                    from: "User_01a2".to_string(),
                    to: "save_03c4".to_string(),
                    edge_type: EdgeType::Calls,
                },
            ],
        };

        FrameworkNoiseFilter::filter_graph(&mut graph);

        // Should remove DI-only constructor
        assert_eq!(graph.definitions.len(), 2);
        assert!(graph.definitions.iter().any(|n| n.id == "User_01a2"));
        assert!(graph.definitions.iter().any(|n| n.id == "save_03c4"));
        assert!(!graph.definitions.iter().any(|n| n.id == "ctor_1"));

        // Should remove edges referencing removed constructor
        assert_eq!(graph.references.len(), 1);
        assert_eq!(graph.references[0].from, "User_01a2");
        assert_eq!(graph.references[0].to, "save_03c4");
    }

    #[test]
    fn test_filter_graph_simplifies_dto_properties() {
        let mut graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                create_test_node("UserDto_01a2", "UserDto", ScipSymbolKind::Class, None),
                create_test_node(
                    "name_prop",
                    "name",
                    ScipSymbolKind::Variable,
                    Some("@ApiProperty() @IsString() name: string".to_string()),
                ),
            ],
            references: vec![],
        };

        // Set parent_id for the property
        graph.definitions[1].parent_id = Some("UserDto_01a2".to_string());

        FrameworkNoiseFilter::filter_graph(&mut graph);

        // Should simplify DTO property signature
        let name_prop = graph
            .definitions
            .iter()
            .find(|n| n.id == "name_prop")
            .unwrap();
        assert_eq!(name_prop.signature, Some("name: string".to_string()));
    }

    #[test]
    fn test_filter_graph_preserves_non_boilerplate() {
        let mut graph = YcgGraph {
            metadata: ProjectMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            definitions: vec![
                create_test_node("User_01a2", "User", ScipSymbolKind::Class, None),
                create_test_node(
                    "ctor_complex",
                    "constructor",
                    ScipSymbolKind::Method,
                    Some(
                        "constructor(name: string) { this.name = name.toUpperCase(); }".to_string(),
                    ),
                ),
                create_test_node(
                    "save_03c4",
                    "save",
                    ScipSymbolKind::Method,
                    Some("save(data: any)".to_string()),
                ),
            ],
            references: vec![],
        };

        FrameworkNoiseFilter::filter_graph(&mut graph);

        // Should preserve constructor with complex logic
        assert_eq!(graph.definitions.len(), 3);
        assert!(graph.definitions.iter().any(|n| n.id == "ctor_complex"));
    }
}
