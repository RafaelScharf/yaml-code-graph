// crates/ycg_core/src/type_abbreviator.rs
//! Type abbreviation system for ad-hoc format
//!
//! This module provides functionality to abbreviate type names for token efficiency
//! while maintaining semantic clarity.
//!
//! **Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8**

/// Type abbreviator for converting verbose type names to compact representations
pub struct TypeAbbreviator;

impl TypeAbbreviator {
    /// Abbreviate a type string according to the standard mapping
    ///
    /// Handles:
    /// - Simple types (string → str, number → num, etc.)
    /// - Array types (User[] → User[])
    /// - Optional types (User? → User?)
    /// - Generic types (Promise<User> → Promise<User>)
    /// - Nested generics (Promise<Result<User>> → Promise<Result<User>>)
    ///
    /// # Arguments
    /// * `type_str` - The type string to abbreviate
    ///
    /// # Returns
    /// * Abbreviated type string
    ///
    /// # Examples
    /// ```
    /// use ycg_core::type_abbreviator::TypeAbbreviator;
    ///
    /// assert_eq!(TypeAbbreviator::abbreviate("string"), "str");
    /// assert_eq!(TypeAbbreviator::abbreviate("number"), "num");
    /// assert_eq!(TypeAbbreviator::abbreviate("User[]"), "User[]");
    /// assert_eq!(TypeAbbreviator::abbreviate("Promise<string>"), "Promise<str>");
    /// ```
    ///
    /// **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8**
    pub fn abbreviate(type_str: &str) -> String {
        Self::abbreviate_recursive(type_str.trim())
    }

    /// Recursively abbreviate types, handling generics and arrays
    fn abbreviate_recursive(type_str: &str) -> String {
        // Handle array types: User[] -> User[]
        // Requirement 4.5: Preserve array types
        if type_str.ends_with("[]") {
            let base = &type_str[..type_str.len() - 2];
            return format!("{}[]", Self::abbreviate_recursive(base));
        }

        // Handle optional types: User? -> User?
        // Custom extension for optional types
        if type_str.ends_with('?') {
            let base = &type_str[..type_str.len() - 1];
            return format!("{}?", Self::abbreviate_recursive(base));
        }

        // Handle generic types: Promise<User> -> Promise<User>
        // Requirement 4.7: Preserve generic type parameters
        if let Some(generic_start) = type_str.find('<') {
            if let Some(generic_end) = type_str.rfind('>') {
                let base = &type_str[..generic_start];
                let generic_params = &type_str[generic_start + 1..generic_end];

                // Abbreviate base and params
                let abbrev_base = Self::abbreviate_simple(base);
                let abbrev_params = Self::abbreviate_generic_params(generic_params);

                return format!("{}<{}>", abbrev_base, abbrev_params);
            }
        }

        // Simple type abbreviation
        Self::abbreviate_simple(type_str)
    }

    /// Abbreviate generic parameters (comma-separated types)
    fn abbreviate_generic_params(params: &str) -> String {
        // Split by comma, but be careful with nested generics
        let parts = Self::split_generic_params(params);
        parts
            .iter()
            .map(|p| Self::abbreviate_recursive(p.trim()))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Split generic parameters by comma, respecting nested generics
    fn split_generic_params(params: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut depth = 0;

        for ch in params.chars() {
            match ch {
                '<' => {
                    depth += 1;
                    current.push(ch);
                }
                '>' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    if !current.is_empty() {
                        parts.push(current.trim().to_string());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            parts.push(current.trim().to_string());
        }

        parts
    }

    /// Abbreviate a simple (non-generic, non-array) type
    ///
    /// **Standard Abbreviation Table:**
    /// - string → str (Requirement 4.1)
    /// - number → num (Requirement 4.2)
    /// - boolean → bool (Requirement 4.3)
    /// - any → any (Requirement 4.4)
    /// - void → void (Requirement 4.5)
    /// - Custom types → preserved (Requirement 4.6)
    fn abbreviate_simple(type_str: &str) -> String {
        match type_str.trim() {
            "string" => "str".to_string(),
            "number" => "num".to_string(),
            "boolean" => "bool".to_string(),
            "any" => "any".to_string(),
            "void" => "void".to_string(),
            other => other.to_string(), // Preserve custom types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abbreviate_simple_types() {
        // Requirement 4.1: string → str
        assert_eq!(TypeAbbreviator::abbreviate("string"), "str");

        // Requirement 4.2: number → num
        assert_eq!(TypeAbbreviator::abbreviate("number"), "num");

        // Requirement 4.3: boolean → bool
        assert_eq!(TypeAbbreviator::abbreviate("boolean"), "bool");

        // Requirement 4.4: any → any
        assert_eq!(TypeAbbreviator::abbreviate("any"), "any");

        // Requirement 4.5: void → void
        assert_eq!(TypeAbbreviator::abbreviate("void"), "void");
    }

    #[test]
    fn test_preserve_custom_types() {
        // Requirement 4.6: Preserve custom type names
        assert_eq!(TypeAbbreviator::abbreviate("User"), "User");
        assert_eq!(TypeAbbreviator::abbreviate("InternalUser"), "InternalUser");
        assert_eq!(TypeAbbreviator::abbreviate("MyCustomType"), "MyCustomType");
    }

    #[test]
    fn test_array_types() {
        // Requirement 4.5: Preserve array types
        assert_eq!(TypeAbbreviator::abbreviate("string[]"), "str[]");
        assert_eq!(TypeAbbreviator::abbreviate("number[]"), "num[]");
        assert_eq!(TypeAbbreviator::abbreviate("User[]"), "User[]");
    }

    #[test]
    fn test_optional_types() {
        // Custom extension: optional types
        assert_eq!(TypeAbbreviator::abbreviate("string?"), "str?");
        assert_eq!(TypeAbbreviator::abbreviate("User?"), "User?");
        assert_eq!(TypeAbbreviator::abbreviate("number?"), "num?");
    }

    #[test]
    fn test_generic_types() {
        // Requirement 4.7: Preserve generic type parameters
        assert_eq!(
            TypeAbbreviator::abbreviate("Promise<string>"),
            "Promise<str>"
        );
        assert_eq!(
            TypeAbbreviator::abbreviate("Promise<number>"),
            "Promise<num>"
        );
        assert_eq!(
            TypeAbbreviator::abbreviate("Promise<User>"),
            "Promise<User>"
        );
        assert_eq!(TypeAbbreviator::abbreviate("Array<string>"), "Array<str>");
    }

    #[test]
    fn test_nested_generics() {
        // Requirement 4.8: Apply abbreviations recursively to nested types
        assert_eq!(
            TypeAbbreviator::abbreviate("Promise<Result<string>>"),
            "Promise<Result<str>>"
        );
        assert_eq!(
            TypeAbbreviator::abbreviate("Promise<Array<number>>"),
            "Promise<Array<num>>"
        );
        assert_eq!(
            TypeAbbreviator::abbreviate("Result<Promise<User>>"),
            "Result<Promise<User>>"
        );
    }

    #[test]
    fn test_multiple_generic_params() {
        // Multiple type parameters
        assert_eq!(
            TypeAbbreviator::abbreviate("Map<string,number>"),
            "Map<str,num>"
        );
        assert_eq!(
            TypeAbbreviator::abbreviate("Map<string, number>"),
            "Map<str,num>"
        );
        assert_eq!(
            TypeAbbreviator::abbreviate("Either<string,User>"),
            "Either<str,User>"
        );
    }

    #[test]
    fn test_complex_nested_generics() {
        // Complex nested generics with multiple parameters
        assert_eq!(
            TypeAbbreviator::abbreviate("Promise<Map<string,number>>"),
            "Promise<Map<str,num>>"
        );
        assert_eq!(
            TypeAbbreviator::abbreviate("Result<Either<string,number>>"),
            "Result<Either<str,num>>"
        );
    }

    #[test]
    fn test_array_of_generics() {
        // Array of generic types
        assert_eq!(
            TypeAbbreviator::abbreviate("Promise<string>[]"),
            "Promise<str>[]"
        );
        assert_eq!(
            TypeAbbreviator::abbreviate("Array<User>[]"),
            "Array<User>[]"
        );
    }

    #[test]
    fn test_optional_generics() {
        // Optional generic types
        assert_eq!(
            TypeAbbreviator::abbreviate("Promise<string>?"),
            "Promise<str>?"
        );
        assert_eq!(TypeAbbreviator::abbreviate("User?"), "User?");
    }

    #[test]
    fn test_whitespace_handling() {
        // Should handle whitespace
        assert_eq!(TypeAbbreviator::abbreviate(" string "), "str");
        assert_eq!(TypeAbbreviator::abbreviate("  number  "), "num");
        assert_eq!(
            TypeAbbreviator::abbreviate("Promise< string >"),
            "Promise<str>"
        );
    }

    #[test]
    fn test_case_sensitivity() {
        // Should be case-sensitive (only lowercase matches)
        assert_eq!(TypeAbbreviator::abbreviate("String"), "String"); // Not abbreviated
        assert_eq!(TypeAbbreviator::abbreviate("NUMBER"), "NUMBER"); // Not abbreviated
        assert_eq!(TypeAbbreviator::abbreviate("string"), "str"); // Abbreviated
    }

    #[test]
    fn test_round_trip_preservation() {
        // Verify that abbreviation is consistent
        let types = vec![
            "string",
            "number[]",
            "Promise<User>",
            "Map<string,number>",
            "User?",
        ];

        for type_str in types {
            let abbreviated = TypeAbbreviator::abbreviate(type_str);
            // Abbreviating again should give same result
            let re_abbreviated = TypeAbbreviator::abbreviate(&abbreviated);
            assert_eq!(abbreviated, re_abbreviated);
        }
    }
}
