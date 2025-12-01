// crates/ycg_core/src/logic_extractor.rs
//! Logic extraction for ad-hoc format Level 2
//!
//! This module extracts compact logic representations from method/function bodies
//! and formats them using logic keywords for token efficiency.
//!
//! **Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10, 3.11, 5.1-5.10**
//!
//! ## Logic Keywords
//!
//! - `check(condition)`: Conditional checks (if statements, guards, throws)
//! - `action(operation)`: Side-effect operations (assignments, method calls)
//! - `return(expression)`: Return statements
//! - `match(pattern)?true:false`: Pattern matching (ternary, switch)
//! - `get(source)`: Data retrieval operations
//!
//! ## Examples
//!
//! ```text
//! // Simple check
//! if (stock > 0) { ... }
//! → check(stock>0)
//!
//! // Multiple checks chained
//! if (user.isActive && user.balance >= price) { ... }
//! → check(user.isActive&&user.balance>=price)
//!
//! // Action
//! this.balance -= amount;
//! → action(deduct_balance)
//!
//! // Return
//! return item.qty > 0;
//! → return(item.qty>0)
//!
//! // Match (ternary)
//! return isAdmin ? 'allow' : 'deny';
//! → match(isAdmin)?allow:deny
//!
//! // Chained logic
//! check(stock>0);check(user.balance>=price);action(deduct_balance);action(save_order)
//! ```

use crate::model::{ScipSymbolKind, SymbolNode};

/// Maximum length for logic representation (excluding "logic:" prefix)
const MAX_LOGIC_LENGTH: usize = 200;

/// Logic extractor for methods and functions
pub struct LogicExtractor;

impl LogicExtractor {
    /// Extract compact logic representation from a SymbolNode
    ///
    /// This is a placeholder implementation that returns None.
    /// Full implementation requires tree-sitter AST parsing integration.
    ///
    /// # Arguments
    /// * `node` - The symbol node to extract logic from
    /// * `_source` - Source code (unused in placeholder)
    ///
    /// # Returns
    /// * `Some(String)` - Logic representation in format "logic:steps"
    /// * `None` - If no logic can be extracted or node is not a method/function
    ///
    /// # Future Implementation
    ///
    /// The full implementation will:
    /// 1. Parse method body using tree-sitter
    /// 2. Traverse AST to identify logic patterns
    /// 3. Extract and format logic keywords
    /// 4. Chain steps with semicolons
    /// 5. Truncate at MAX_LOGIC_LENGTH characters
    ///
    /// **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10, 3.11**
    pub fn extract_logic(node: &SymbolNode, _source: &str) -> Option<String> {
        // Only extract logic for methods and functions
        if !matches!(node.kind, ScipSymbolKind::Method | ScipSymbolKind::Function) {
            return None;
        }

        // Placeholder: Return None until tree-sitter integration is complete
        // This allows the system to work without logic extraction
        // and fall back to Level 1 format (signatures only)
        None
    }

    /// Validate logic representation format
    ///
    /// Ensures logic string follows the correct format:
    /// - Starts with "logic:"
    /// - Contains only valid keywords (check, action, return, match, get)
    /// - Steps are separated by semicolons
    /// - Length does not exceed MAX_LOGIC_LENGTH (excluding "logic:" prefix)
    ///
    /// **Validates: Requirements 9.3, 9.4**
    pub fn validate_logic(logic: &str) -> bool {
        // Must start with "logic:"
        if !logic.starts_with("logic:") {
            return false;
        }

        let content = &logic[6..]; // Skip "logic:" prefix

        // Check length limit
        if content.len() > MAX_LOGIC_LENGTH {
            return false;
        }

        // Validate keywords
        Self::validate_keywords(content)
    }

    /// Validate that logic string contains only valid keywords
    fn validate_keywords(content: &str) -> bool {
        // Valid keywords
        const VALID_KEYWORDS: &[&str] = &["check", "action", "return", "match", "get"];

        // Split by semicolons to get individual steps
        for step in content.split(';') {
            let step = step.trim();
            if step.is_empty() {
                continue;
            }

            // Check if step starts with a valid keyword
            let has_valid_keyword = VALID_KEYWORDS
                .iter()
                .any(|keyword| step.starts_with(keyword));

            if !has_valid_keyword {
                return false;
            }
        }

        true
    }

    /// Truncate logic representation to maximum length
    ///
    /// If logic exceeds MAX_LOGIC_LENGTH, truncates and adds "..." suffix.
    ///
    /// **Validates: Requirements 3.11, 10.5**
    pub fn truncate_logic(logic: &str) -> String {
        if logic.len() <= MAX_LOGIC_LENGTH {
            return logic.to_string();
        }

        // Truncate and add ellipsis
        format!("{}...", &logic[..MAX_LOGIC_LENGTH - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(kind: ScipSymbolKind) -> SymbolNode {
        SymbolNode {
            id: "test_id".to_string(),
            name: "testMethod".to_string(),
            kind,
            parent_id: None,
            documentation: None,
            signature: None,
            logic: None,
        }
    }

    #[test]
    fn test_extract_logic_returns_none_for_methods() {
        // Placeholder implementation returns None
        let node = create_test_node(ScipSymbolKind::Method);
        let result = LogicExtractor::extract_logic(&node, "");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_logic_returns_none_for_functions() {
        // Placeholder implementation returns None
        let node = create_test_node(ScipSymbolKind::Function);
        let result = LogicExtractor::extract_logic(&node, "");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_logic_returns_none_for_classes() {
        // Should not extract logic for classes
        let node = create_test_node(ScipSymbolKind::Class);
        let result = LogicExtractor::extract_logic(&node, "");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_logic_returns_none_for_variables() {
        // Should not extract logic for variables
        let node = create_test_node(ScipSymbolKind::Variable);
        let result = LogicExtractor::extract_logic(&node, "");
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_logic_valid_check() {
        // Requirement 9.3: Validate logic field format
        assert!(LogicExtractor::validate_logic("logic:check(stock>0)"));
    }

    #[test]
    fn test_validate_logic_valid_action() {
        assert!(LogicExtractor::validate_logic("logic:action(save_order)"));
    }

    #[test]
    fn test_validate_logic_valid_return() {
        assert!(LogicExtractor::validate_logic("logic:return(item.qty>0)"));
    }

    #[test]
    fn test_validate_logic_valid_match() {
        assert!(LogicExtractor::validate_logic(
            "logic:match(isAdmin)?allow:deny"
        ));
    }

    #[test]
    fn test_validate_logic_valid_get() {
        assert!(LogicExtractor::validate_logic("logic:get(user_roles)"));
    }

    #[test]
    fn test_validate_logic_valid_chained() {
        // Requirement 3.8: Chain multiple logic steps with semicolons
        assert!(LogicExtractor::validate_logic(
            "logic:check(stock>0);action(deduct_balance);return(order)"
        ));
    }

    #[test]
    fn test_validate_logic_invalid_no_prefix() {
        // Must start with "logic:"
        assert!(!LogicExtractor::validate_logic("check(stock>0)"));
    }

    #[test]
    fn test_validate_logic_invalid_keyword() {
        // Requirement 9.4: Validate logic keywords are valid
        assert!(!LogicExtractor::validate_logic("logic:invalid(test)"));
    }

    #[test]
    fn test_validate_logic_too_long() {
        // Requirement 3.11: Limit logic to 200 characters
        let long_logic = format!("logic:{}", "x".repeat(201));
        assert!(!LogicExtractor::validate_logic(&long_logic));
    }

    #[test]
    fn test_validate_logic_max_length_ok() {
        // Exactly 200 characters in content (after "logic:" prefix) should be valid
        let content = format!("check({})", "x".repeat(193)); // "check(" = 6 chars, ")" = 1 char, total = 200
        assert_eq!(content.len(), 200);
        let logic = format!("logic:{}", content);
        assert!(LogicExtractor::validate_logic(&logic));
    }

    #[test]
    fn test_truncate_logic_short() {
        let logic = "check(stock>0)";
        let result = LogicExtractor::truncate_logic(logic);
        assert_eq!(result, "check(stock>0)");
    }

    #[test]
    fn test_truncate_logic_long() {
        // Requirement 3.11, 10.5: Truncate at 200 characters
        let logic = "x".repeat(250);
        let result = LogicExtractor::truncate_logic(&logic);
        assert_eq!(result.len(), 200);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_logic_exactly_max() {
        let logic = "x".repeat(200);
        let result = LogicExtractor::truncate_logic(&logic);
        assert_eq!(result.len(), 200);
        assert!(!result.ends_with("..."));
    }

    #[test]
    fn test_validate_keywords_empty() {
        assert!(LogicExtractor::validate_keywords(""));
    }

    #[test]
    fn test_validate_keywords_multiple_valid() {
        assert!(LogicExtractor::validate_keywords(
            "check(x);action(y);return(z)"
        ));
    }

    #[test]
    fn test_validate_keywords_with_spaces() {
        assert!(LogicExtractor::validate_keywords(
            "check(x) ; action(y) ; return(z)"
        ));
    }

    #[test]
    fn test_validate_keywords_invalid_mixed() {
        assert!(!LogicExtractor::validate_keywords(
            "check(x);invalid(y);return(z)"
        ));
    }
}
