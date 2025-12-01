// crates/ycg_core/src/signature_extractor.rs
//! Signature extraction and compaction for ad-hoc format
//!
//! This module extracts function/method signatures and formats them compactly
//! with abbreviated types for token efficiency.
//!
//! **Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8**

use crate::ast_cache::AstCache;
use crate::model::SymbolNode;
use crate::type_abbreviator::TypeAbbreviator;

/// Signature extractor for methods and functions
pub struct SignatureExtractor;

impl SignatureExtractor {
    /// Extract compact signature from a SymbolNode
    ///
    /// Tries to use the signature from the enricher if available,
    /// otherwise falls back to the simple name.
    ///
    /// # Arguments
    /// * `node` - The symbol node to extract signature from
    ///
    /// # Returns
    /// * `Some(String)` - Compact signature in format "name(param1:type1,param2:type2):ReturnType"
    /// * `None` - If no signature can be extracted (falls back to simple name)
    ///
    /// # Examples
    /// ```
    /// // Input signature: "async findOne(user: string): Promise<InternalUser | null>"
    /// // Output: "findOne(user:str):Promise<InternalUser>?"
    /// ```
    ///
    /// **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8**
    pub fn extract_signature(node: &SymbolNode) -> Option<String> {
        // If node has a signature from enricher, use it
        if let Some(ref sig) = node.signature {
            return Some(Self::compact_signature(sig, &node.name));
        }

        // No signature available, return None to fall back to simple name
        None
    }

    /// Extract compact signature from a SymbolNode with AST caching
    ///
    /// This version accepts an AstCache to enable AST reuse when extracting
    /// signatures from multiple symbols in the same file.
    ///
    /// # Arguments
    /// * `node` - The symbol node to extract signature from
    /// * `file_path` - Path to the source file (for cache lookup)
    /// * `cache` - AST cache for reusing parsed ASTs
    ///
    /// # Returns
    /// * `Some(String)` - Compact signature
    /// * `None` - If no signature can be extracted
    ///
    /// **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 10.3, 10.4**
    pub fn extract_signature_with_cache(
        node: &SymbolNode,
        file_path: &str,
        cache: &mut AstCache,
    ) -> Option<String> {
        // If node has a signature from enricher, use it
        if let Some(ref sig) = node.signature {
            return Some(Self::compact_signature(sig, &node.name));
        }

        // Try to extract from cached AST
        // When tree-sitter integration is complete, this will use the cached AST
        let _ast = cache.get(file_path)?;

        // Placeholder: tree-sitter extraction would happen here
        // For now, return None to fall back to simple name
        None
    }

    /// Compact a signature by abbreviating types and removing unnecessary keywords
    ///
    /// Transformations:
    /// - Remove async/export/public keywords
    /// - Abbreviate parameter types
    /// - Abbreviate return type
    /// - Remove whitespace
    /// - Handle optional return types (| null, | undefined)
    ///
    /// **Validates: Requirements 2.2, 2.3, 2.4, 2.5**
    fn compact_signature(sig: &str, method_name: &str) -> String {
        // Remove leading keywords (async, export, public, etc.)
        let cleaned = sig
            .replace("async ", "")
            .replace("export ", "")
            .replace("public ", "")
            .replace("private ", "")
            .replace("protected ", "")
            .replace("static ", "");

        // Try to parse the signature
        if let Some((name, params, return_type)) = Self::parse_signature(&cleaned, method_name) {
            Self::format_compact_signature(&name, &params, &return_type)
        } else {
            // Fallback: return the method name
            method_name.to_string()
        }
    }

    /// Parse a signature string into components
    ///
    /// Returns: (method_name, [(param_name, param_type)], return_type)
    fn parse_signature(
        sig: &str,
        fallback_name: &str,
    ) -> Option<(String, Vec<(String, String)>, String)> {
        // Pattern: name(params): return_type
        // or: name(params)

        // Find the method name (before opening paren)
        let paren_start = sig.find('(')?;
        let name = sig[..paren_start].trim();
        let name = if name.is_empty() { fallback_name } else { name };

        // Find matching closing paren
        let paren_end = Self::find_matching_paren(sig, paren_start)?;

        // Extract parameters
        let params_str = &sig[paren_start + 1..paren_end];
        let params = Self::parse_parameters(params_str);

        // Extract return type (after colon)
        let return_type = if let Some(colon_pos) = sig[paren_end..].find(':') {
            let return_start = paren_end + colon_pos + 1;
            sig[return_start..].trim().to_string()
        } else {
            String::new()
        };

        Some((name.to_string(), params, return_type))
    }

    /// Find matching closing parenthesis
    fn find_matching_paren(s: &str, start: usize) -> Option<usize> {
        let mut depth = 0;
        for (i, ch) in s[start..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(start + i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Parse parameters from parameter string
    ///
    /// Handles:
    /// - Simple params: "user: string"
    /// - Optional params: "user?: string"
    /// - Default values: "limit: number = 10"
    /// - Multiple params: "user: string, limit: number"
    ///
    /// Returns: Vec<(param_name, param_type)>
    fn parse_parameters(params_str: &str) -> Vec<(String, String)> {
        if params_str.trim().is_empty() {
            return Vec::new();
        }

        let mut params = Vec::new();
        let parts = Self::split_parameters(params_str);

        for part in parts {
            if let Some((name, type_str)) = Self::parse_single_parameter(&part) {
                params.push((name, type_str));
            }
        }

        params
    }

    /// Split parameters by comma, respecting nested generics and parentheses
    fn split_parameters(params_str: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut depth = 0;

        for ch in params_str.chars() {
            match ch {
                '<' | '(' | '[' => {
                    depth += 1;
                    current.push(ch);
                }
                '>' | ')' | ']' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        parts
    }

    /// Parse a single parameter into (name, type)
    ///
    /// Handles:
    /// - "user: string" → ("user", "string")
    /// - "user?: string" → ("user", "string?")
    /// - "limit: number = 10" → ("limit", "number")
    fn parse_single_parameter(param: &str) -> Option<(String, String)> {
        // Remove default value if present (everything after =)
        let param = if let Some(eq_pos) = param.find('=') {
            &param[..eq_pos]
        } else {
            param
        };

        // Split by colon
        let colon_pos = param.find(':')?;
        let name_part = param[..colon_pos].trim();
        let type_part = param[colon_pos + 1..].trim();

        // Handle optional parameters (name?)
        let (name, is_optional) = if name_part.ends_with('?') {
            (&name_part[..name_part.len() - 1], true)
        } else {
            (name_part, false)
        };

        // Add ? to type if parameter is optional
        let type_str = if is_optional && !type_part.ends_with('?') {
            format!("{}?", type_part)
        } else {
            type_part.to_string()
        };

        Some((name.to_string(), type_str))
    }

    /// Format a compact signature with abbreviated types
    ///
    /// Format: methodName(param1:type1,param2:type2):ReturnType
    ///
    /// **Validates: Requirements 2.2, 2.3, 2.4, 2.5**
    fn format_compact_signature(
        name: &str,
        params: &[(String, String)],
        return_type: &str,
    ) -> String {
        // Abbreviate parameter types
        let compact_params = params
            .iter()
            .map(|(param_name, param_type)| {
                // Handle union types with null/undefined (convert to optional)
                let normalized_type = Self::normalize_optional_type(param_type);
                let abbrev_type = TypeAbbreviator::abbreviate(&normalized_type);
                format!("{}:{}", param_name, abbrev_type)
            })
            .collect::<Vec<_>>()
            .join(",");

        // Abbreviate return type
        let normalized_return = Self::normalize_optional_type(return_type);
        let compact_return = TypeAbbreviator::abbreviate(&normalized_return);

        // Format: methodName(param1:type1,param2:type2):ReturnType
        // Requirement 2.8: Omit return type if void or empty
        if compact_return.is_empty() || compact_return == "void" {
            format!("{}({})", name, compact_params)
        } else {
            format!("{}({}):{}", name, compact_params, compact_return)
        }
    }

    /// Normalize optional types from union notation to ? notation
    ///
    /// Converts:
    /// - "User | null" → "User?"
    /// - "string | undefined" → "str?"
    /// - "User | null | undefined" → "User?"
    /// - "boolean | Promise<boolean> | Observable<boolean>" → "bool" (takes first type for simplicity)
    ///
    /// **Validates: Requirement 2.4**
    fn normalize_optional_type(type_str: &str) -> String {
        let trimmed = type_str.trim();

        // Check if it's a union with null or undefined
        if trimmed.contains('|') {
            // Split by | and filter out null/undefined
            let parts: Vec<&str> = trimmed
                .split('|')
                .map(|s| s.trim())
                .filter(|s| *s != "null" && *s != "undefined")
                .collect();

            if parts.len() == 1 {
                // Single type with null/undefined → make it optional
                return format!("{}?", parts[0]);
            } else if parts.len() > 1 {
                // Multiple types in union (e.g., boolean | Promise<boolean> | Observable<boolean>)
                // For compactness, take the first type only
                // This is a simplification for token efficiency
                return parts[0].to_string();
            }
        }

        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ScipSymbolKind, SymbolNode};

    fn create_test_node(name: &str, signature: Option<String>) -> SymbolNode {
        SymbolNode {
            id: "test_id".to_string(),
            name: name.to_string(),
            kind: ScipSymbolKind::Method,
            parent_id: None,
            documentation: None,
            signature,
            logic: None,
        }
    }

    #[test]
    fn test_extract_signature_with_enricher_data() {
        // Requirement 2.1: Use signature from enricher if available
        let node = create_test_node(
            "findOne",
            Some("async findOne(user: string): Promise<InternalUser>".to_string()),
        );

        let result = SignatureExtractor::extract_signature(&node);
        assert!(result.is_some());
        let sig = result.unwrap();

        // Should contain method name and abbreviated types
        assert!(sig.contains("findOne"));
        assert!(sig.contains("str")); // string abbreviated
    }

    #[test]
    fn test_extract_signature_without_enricher_data() {
        // Requirement 2.7: Fall back to simple name if no signature
        let node = create_test_node("findOne", None);

        let result = SignatureExtractor::extract_signature(&node);
        assert!(result.is_none()); // Should return None for fallback
    }

    #[test]
    fn test_compact_signature_simple() {
        // Requirement 2.2: Format as name(params):return
        let sig = "findOne(id: string): User";
        let result = SignatureExtractor::compact_signature(sig, "findOne");

        assert_eq!(result, "findOne(id:str):User");
    }

    #[test]
    fn test_compact_signature_multiple_params() {
        // Requirement 2.2: Handle multiple parameters
        let sig = "create(name: string, age: number, active: boolean): User";
        let result = SignatureExtractor::compact_signature(sig, "create");

        assert_eq!(result, "create(name:str,age:num,active:bool):User");
    }

    #[test]
    fn test_compact_signature_optional_params() {
        // Requirement 2.5: Handle optional parameters
        let sig = "findOne(id: string, options?: QueryOptions): User";
        let result = SignatureExtractor::compact_signature(sig, "findOne");

        assert!(result.contains("options:QueryOptions?"));
    }

    #[test]
    fn test_compact_signature_with_generics() {
        // Requirement 2.3: Abbreviate types in generics
        let sig = "findAll(query: string): Promise<User[]>";
        let result = SignatureExtractor::compact_signature(sig, "findAll");

        assert_eq!(result, "findAll(query:str):Promise<User[]>");
    }

    #[test]
    fn test_compact_signature_optional_return() {
        // Requirement 2.4: Handle optional return types
        let sig = "findOne(id: string): User | null";
        let result = SignatureExtractor::compact_signature(sig, "findOne");

        assert_eq!(result, "findOne(id:str):User?");
    }

    #[test]
    fn test_compact_signature_void_return() {
        // Requirement 2.8: Omit void return type
        let sig = "deleteUser(id: string): void";
        let result = SignatureExtractor::compact_signature(sig, "deleteUser");

        assert_eq!(result, "deleteUser(id:str)");
    }

    #[test]
    fn test_compact_signature_no_return() {
        // Requirement 2.8: Omit return type if not specified
        let sig = "constructor(service: UserService)";
        let result = SignatureExtractor::compact_signature(sig, "constructor");

        assert_eq!(result, "constructor(service:UserService)");
    }

    #[test]
    fn test_compact_signature_removes_keywords() {
        // Should remove async, export, public, etc.
        let sig = "async public findOne(id: string): Promise<User>";
        let result = SignatureExtractor::compact_signature(sig, "findOne");

        assert!(!result.contains("async"));
        assert!(!result.contains("public"));
        assert_eq!(result, "findOne(id:str):Promise<User>");
    }

    #[test]
    fn test_compact_signature_with_default_values() {
        // Should handle default values
        let sig = "create(name: string, limit: number = 10): User";
        let result = SignatureExtractor::compact_signature(sig, "create");

        assert_eq!(result, "create(name:str,limit:num):User");
    }

    #[test]
    fn test_normalize_optional_type_with_null() {
        let result = SignatureExtractor::normalize_optional_type("User | null");
        assert_eq!(result, "User?");
    }

    #[test]
    fn test_normalize_optional_type_with_undefined() {
        let result = SignatureExtractor::normalize_optional_type("string | undefined");
        assert_eq!(result, "string?");
    }

    #[test]
    fn test_normalize_optional_type_with_both() {
        let result = SignatureExtractor::normalize_optional_type("User | null | undefined");
        assert_eq!(result, "User?");
    }

    #[test]
    fn test_normalize_optional_type_union() {
        // Multiple types with null - for compactness, take first type only
        let result = SignatureExtractor::normalize_optional_type("User | Admin | null");
        assert_eq!(result, "User");
    }

    #[test]
    fn test_parse_parameters_empty() {
        let params = SignatureExtractor::parse_parameters("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_parameters_single() {
        let params = SignatureExtractor::parse_parameters("user: string");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ("user".to_string(), "string".to_string()));
    }

    #[test]
    fn test_parse_parameters_multiple() {
        let params = SignatureExtractor::parse_parameters("name: string, age: number");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], ("name".to_string(), "string".to_string()));
        assert_eq!(params[1], ("age".to_string(), "number".to_string()));
    }

    #[test]
    fn test_parse_parameters_with_generics() {
        let params = SignatureExtractor::parse_parameters("data: Promise<User>, id: string");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], ("data".to_string(), "Promise<User>".to_string()));
        assert_eq!(params[1], ("id".to_string(), "string".to_string()));
    }

    #[test]
    fn test_parse_parameters_optional() {
        let params = SignatureExtractor::parse_parameters("user?: string");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ("user".to_string(), "string?".to_string()));
    }

    #[test]
    fn test_parse_parameters_with_default() {
        let params = SignatureExtractor::parse_parameters("limit: number = 10");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ("limit".to_string(), "number".to_string()));
    }

    #[test]
    fn test_format_compact_signature_full() {
        let params = vec![
            ("user".to_string(), "string".to_string()),
            ("id".to_string(), "number".to_string()),
        ];
        let result =
            SignatureExtractor::format_compact_signature("findOne", &params, "Promise<User>");

        assert_eq!(result, "findOne(user:str,id:num):Promise<User>");
    }

    #[test]
    fn test_format_compact_signature_no_params() {
        let params = vec![];
        let result = SignatureExtractor::format_compact_signature("getAll", &params, "User[]");

        assert_eq!(result, "getAll():User[]");
    }

    #[test]
    fn test_format_compact_signature_void_return() {
        let params = vec![("id".to_string(), "string".to_string())];
        let result = SignatureExtractor::format_compact_signature("delete", &params, "void");

        assert_eq!(result, "delete(id:str)");
    }

    #[test]
    fn test_constructor_with_dependency_injection() {
        // Requirement 2.6: Constructors with dependency injection
        let sig = "constructor(userService: UserService, configService: ConfigService)";
        let result = SignatureExtractor::compact_signature(sig, "constructor");

        assert_eq!(
            result,
            "constructor(userService:UserService,configService:ConfigService)"
        );
    }

    #[test]
    fn test_constructor_no_params() {
        // Test constructor with no parameters
        let sig = "constructor()";
        let result = SignatureExtractor::compact_signature(sig, "constructor");

        assert_eq!(result, "constructor()");
    }

    #[test]
    fn test_method_with_array_params() {
        // Requirement 2.5: Handle array types in parameters
        let sig = "processUsers(users: User[], options: string[]): boolean";
        let result = SignatureExtractor::compact_signature(sig, "processUsers");

        assert_eq!(result, "processUsers(users:User[],options:str[]):bool");
    }

    #[test]
    fn test_method_with_nested_generics() {
        // Test nested generic types
        let sig = "transform(data: Promise<Result<User>>): Promise<Response<User>>";
        let result = SignatureExtractor::compact_signature(sig, "transform");

        assert_eq!(
            result,
            "transform(data:Promise<Result<User>>):Promise<Response<User>>"
        );
    }

    #[test]
    fn test_method_with_complex_optional() {
        // Test complex optional parameters
        let sig = "findUser(id: string, options?: { limit: number, offset: number }): User | null";
        let result = SignatureExtractor::compact_signature(sig, "findUser");

        // Should handle complex optional type
        assert!(result.contains("findUser"));
        assert!(result.contains("id:str"));
        assert!(result.contains("User?"));
    }

    #[test]
    fn test_fallback_on_malformed_signature() {
        // Requirement 2.7: Fallback to simple name on parse failure
        let sig = "this is not a valid signature";
        let result = SignatureExtractor::compact_signature(sig, "methodName");

        assert_eq!(result, "methodName");
    }

    #[test]
    fn test_method_with_rest_parameters() {
        // Test rest/spread parameters
        let sig = "combine(first: string, ...rest: string[]): string";
        let result = SignatureExtractor::compact_signature(sig, "combine");

        assert!(result.contains("combine"));
        assert!(result.contains("first:str"));
        assert!(result.contains("str"));
    }

    #[test]
    fn test_method_with_union_types() {
        // Test union types in parameters
        let sig = "process(value: string | number): boolean";
        let result = SignatureExtractor::compact_signature(sig, "process");

        assert!(result.contains("process"));
        assert!(result.contains("bool"));
    }

    #[test]
    fn test_simple_method_no_types() {
        // Test method with no type annotations (edge case)
        let sig = "simpleMethod()";
        let result = SignatureExtractor::compact_signature(sig, "simpleMethod");

        assert_eq!(result, "simpleMethod()");
    }

    #[test]
    fn test_method_with_multiple_optional_params() {
        // Test multiple optional parameters
        let sig = "search(query: string, limit?: number, offset?: number): User[]";
        let result = SignatureExtractor::compact_signature(sig, "search");

        assert_eq!(result, "search(query:str,limit:num?,offset:num?):User[]");
    }
}
