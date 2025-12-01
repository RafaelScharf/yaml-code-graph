// crates/ycg_core/src/ast_cache.rs
//! AST caching for performance optimization
//!
//! This module provides caching for parsed AST nodes to avoid redundant parsing
//! when extracting signatures and logic from multiple symbols in the same file.
//!
//! **Requirements: 10.3, 10.4**
//!
//! ## Performance Benefits
//!
//! Without caching, each symbol in a file would trigger a full parse:
//! - File with 10 methods = 10 parses of the same file
//! - File with 50 methods = 50 parses of the same file
//!
//! With caching:
//! - File with 10 methods = 1 parse, 9 cache hits
//! - File with 50 methods = 1 parse, 49 cache hits
//!
//! ## Usage
//!
//! ```rust
//! use ycg_core::ast_cache::AstCache;
//!
//! let mut cache = AstCache::new();
//! let source_code = "function hello() {}";
//!
//! // First access parses the file
//! let ast1 = cache.get_or_parse("file.ts", source_code);
//!
//! // Subsequent accesses use cached AST
//! let ast2 = cache.get_or_parse("file.ts", source_code);
//! ```

use std::collections::HashMap;

/// Placeholder for tree-sitter Tree type
///
/// When tree-sitter integration is complete, this will be replaced with
/// the actual tree_sitter::Tree type.
///
/// For now, we use a simple struct to represent the parsed AST.
#[derive(Clone, Debug)]
pub struct ParsedAst {
    /// File path this AST was parsed from
    pub file_path: String,
    /// Source code content
    pub source: String,
    // Future: tree_sitter::Tree will be added here
}

/// AST cache for parsed source files
///
/// Caches parsed AST nodes per file to avoid redundant parsing when
/// extracting signatures and logic from multiple symbols in the same file.
///
/// **Validates: Requirements 10.3, 10.4**
pub struct AstCache {
    /// Map from file path to parsed AST
    cache: HashMap<String, ParsedAst>,
    /// Statistics for cache performance monitoring
    stats: CacheStats,
}

/// Cache performance statistics
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// Number of cache hits (AST found in cache)
    pub hits: usize,
    /// Number of cache misses (AST had to be parsed)
    pub misses: usize,
    /// Number of files currently cached
    pub cached_files: usize,
}

impl CacheStats {
    /// Calculate cache hit rate as a percentage
    ///
    /// Returns 0.0 if no accesses have been made yet.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Get total number of cache accesses
    pub fn total_accesses(&self) -> usize {
        self.hits + self.misses
    }
}

impl AstCache {
    /// Create a new empty AST cache
    ///
    /// # Examples
    /// ```
    /// use ycg_core::ast_cache::AstCache;
    ///
    /// let cache = AstCache::new();
    /// ```
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            stats: CacheStats::default(),
        }
    }

    /// Create a new AST cache with pre-allocated capacity
    ///
    /// Use this when you know approximately how many files will be processed
    /// to avoid reallocation overhead.
    ///
    /// # Arguments
    /// * `capacity` - Expected number of files to cache
    ///
    /// # Examples
    /// ```
    /// use ycg_core::ast_cache::AstCache;
    ///
    /// // Pre-allocate for 100 files
    /// let cache = AstCache::with_capacity(100);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            stats: CacheStats::default(),
        }
    }

    /// Get or parse AST for a file
    ///
    /// If the AST is already cached, returns a reference to the cached AST.
    /// Otherwise, parses the source code and caches the result.
    ///
    /// # Arguments
    /// * `file_path` - Path to the source file (used as cache key)
    /// * `source` - Source code content
    ///
    /// # Returns
    /// Reference to the parsed AST (either from cache or newly parsed)
    ///
    /// # Examples
    /// ```
    /// use ycg_core::ast_cache::AstCache;
    ///
    /// let mut cache = AstCache::new();
    /// let source = "function hello() { return 'world'; }";
    ///
    /// // First call parses the source
    /// let ast1 = cache.get_or_parse("hello.ts", source);
    ///
    /// // Second call uses cached AST
    /// let ast2 = cache.get_or_parse("hello.ts", source);
    /// ```
    ///
    /// **Validates: Requirements 10.3, 10.4**
    pub fn get_or_parse(&mut self, file_path: &str, source: &str) -> &ParsedAst {
        // Check if already cached
        if self.cache.contains_key(file_path) {
            self.stats.hits += 1;
            return self.cache.get(file_path).unwrap();
        }

        // Cache miss - parse the source
        self.stats.misses += 1;

        let ast = Self::parse_source(file_path, source);
        self.cache.insert(file_path.to_string(), ast);
        self.stats.cached_files = self.cache.len();

        self.cache.get(file_path).unwrap()
    }

    /// Check if a file's AST is already cached
    ///
    /// # Arguments
    /// * `file_path` - Path to the source file
    ///
    /// # Returns
    /// `true` if the AST is cached, `false` otherwise
    pub fn contains(&self, file_path: &str) -> bool {
        self.cache.contains_key(file_path)
    }

    /// Get cached AST without parsing
    ///
    /// Returns `None` if the AST is not cached.
    ///
    /// # Arguments
    /// * `file_path` - Path to the source file
    ///
    /// # Returns
    /// `Some(&ParsedAst)` if cached, `None` otherwise
    pub fn get(&self, file_path: &str) -> Option<&ParsedAst> {
        self.cache.get(file_path)
    }

    /// Clear the cache
    ///
    /// Removes all cached ASTs and resets statistics.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.stats = CacheStats::default();
    }

    /// Get cache statistics
    ///
    /// Returns statistics about cache performance including hits, misses,
    /// and hit rate.
    ///
    /// # Examples
    /// ```
    /// use ycg_core::ast_cache::AstCache;
    ///
    /// let mut cache = AstCache::new();
    /// let source = "function test() {}";
    ///
    /// cache.get_or_parse("test.ts", source);
    /// cache.get_or_parse("test.ts", source);
    ///
    /// let stats = cache.stats();
    /// assert_eq!(stats.hits, 1);
    /// assert_eq!(stats.misses, 1);
    /// assert_eq!(stats.hit_rate(), 50.0);
    /// ```
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get the number of files currently cached
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Parse source code into AST
    ///
    /// This is a placeholder implementation. When tree-sitter integration
    /// is complete, this will use tree-sitter to parse the source code.
    ///
    /// # Arguments
    /// * `file_path` - Path to the source file
    /// * `source` - Source code content
    ///
    /// # Returns
    /// Parsed AST representation
    fn parse_source(file_path: &str, source: &str) -> ParsedAst {
        // Placeholder: In the future, this will use tree-sitter
        // For now, just store the source code
        ParsedAst {
            file_path: file_path.to_string(),
            source: source.to_string(),
        }
    }
}

impl Default for AstCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache_is_empty() {
        let cache = AstCache::new();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let cache = AstCache::with_capacity(10);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_get_or_parse_first_access_is_miss() {
        let mut cache = AstCache::new();
        let source = "function hello() {}";

        let ast = cache.get_or_parse("test.ts", source);

        assert_eq!(ast.file_path, "test.ts");
        assert_eq!(ast.source, source);
        assert_eq!(cache.stats().misses, 1);
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_get_or_parse_second_access_is_hit() {
        let mut cache = AstCache::new();
        let source = "function hello() {}";

        // First access - miss
        cache.get_or_parse("test.ts", source);

        // Second access - hit
        let ast = cache.get_or_parse("test.ts", source);

        assert_eq!(ast.file_path, "test.ts");
        assert_eq!(cache.stats().misses, 1);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_get_or_parse_multiple_files() {
        let mut cache = AstCache::new();

        cache.get_or_parse("file1.ts", "code1");
        cache.get_or_parse("file2.ts", "code2");
        cache.get_or_parse("file3.ts", "code3");

        assert_eq!(cache.len(), 3);
        assert_eq!(cache.stats().misses, 3);
        assert_eq!(cache.stats().hits, 0);
    }

    #[test]
    fn test_get_or_parse_multiple_accesses_same_file() {
        let mut cache = AstCache::new();
        let source = "function test() {}";

        // Access same file 5 times
        for _ in 0..5 {
            cache.get_or_parse("test.ts", source);
        }

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.stats().misses, 1); // First access
        assert_eq!(cache.stats().hits, 4); // Subsequent 4 accesses
    }

    #[test]
    fn test_contains() {
        let mut cache = AstCache::new();

        assert!(!cache.contains("test.ts"));

        cache.get_or_parse("test.ts", "code");

        assert!(cache.contains("test.ts"));
        assert!(!cache.contains("other.ts"));
    }

    #[test]
    fn test_get_without_parsing() {
        let mut cache = AstCache::new();

        // Not cached yet
        assert!(cache.get("test.ts").is_none());

        // Cache it
        cache.get_or_parse("test.ts", "code");

        // Now it's cached
        let ast = cache.get("test.ts");
        assert!(ast.is_some());
        assert_eq!(ast.unwrap().file_path, "test.ts");
    }

    #[test]
    fn test_clear() {
        let mut cache = AstCache::new();

        cache.get_or_parse("file1.ts", "code1");
        cache.get_or_parse("file2.ts", "code2");
        cache.get_or_parse("file1.ts", "code1"); // Hit

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 2);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut cache = AstCache::new();

        // No accesses yet
        assert_eq!(cache.stats().hit_rate(), 0.0);

        // 1 miss, 0 hits = 0% hit rate
        cache.get_or_parse("test.ts", "code");
        assert_eq!(cache.stats().hit_rate(), 0.0);

        // 1 miss, 1 hit = 50% hit rate
        cache.get_or_parse("test.ts", "code");
        assert_eq!(cache.stats().hit_rate(), 50.0);

        // 1 miss, 2 hits = 66.67% hit rate
        cache.get_or_parse("test.ts", "code");
        assert!((cache.stats().hit_rate() - 66.67).abs() < 0.1);

        // 1 miss, 3 hits = 75% hit rate
        cache.get_or_parse("test.ts", "code");
        assert_eq!(cache.stats().hit_rate(), 75.0);
    }

    #[test]
    fn test_cache_stats_total_accesses() {
        let mut cache = AstCache::new();

        assert_eq!(cache.stats().total_accesses(), 0);

        cache.get_or_parse("test.ts", "code");
        assert_eq!(cache.stats().total_accesses(), 1);

        cache.get_or_parse("test.ts", "code");
        cache.get_or_parse("test.ts", "code");
        assert_eq!(cache.stats().total_accesses(), 3);
    }

    #[test]
    fn test_cache_stats_cached_files() {
        let mut cache = AstCache::new();

        assert_eq!(cache.stats().cached_files, 0);

        cache.get_or_parse("file1.ts", "code1");
        assert_eq!(cache.stats().cached_files, 1);

        cache.get_or_parse("file2.ts", "code2");
        assert_eq!(cache.stats().cached_files, 2);

        // Accessing same file doesn't increase count
        cache.get_or_parse("file1.ts", "code1");
        assert_eq!(cache.stats().cached_files, 2);
    }

    #[test]
    fn test_realistic_usage_pattern() {
        // Simulate processing a file with 10 methods
        let mut cache = AstCache::new();
        let source = "class MyClass { /* 10 methods */ }";
        let file_path = "my_class.ts";

        // Extract signatures/logic for 10 methods in the same file
        for _ in 0..10 {
            cache.get_or_parse(file_path, source);
        }

        // Should have 1 parse (miss) and 9 cache hits
        assert_eq!(cache.stats().misses, 1);
        assert_eq!(cache.stats().hits, 9);
        assert_eq!(cache.stats().hit_rate(), 90.0);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_multiple_files_realistic() {
        // Simulate processing 3 files with varying numbers of symbols
        let mut cache = AstCache::new();

        // File 1: 5 symbols
        for _ in 0..5 {
            cache.get_or_parse("file1.ts", "code1");
        }

        // File 2: 10 symbols
        for _ in 0..10 {
            cache.get_or_parse("file2.ts", "code2");
        }

        // File 3: 3 symbols
        for _ in 0..3 {
            cache.get_or_parse("file3.ts", "code3");
        }

        // Total: 18 accesses, 3 misses (one per file), 15 hits
        assert_eq!(cache.stats().total_accesses(), 18);
        assert_eq!(cache.stats().misses, 3);
        assert_eq!(cache.stats().hits, 15);
        assert_eq!(cache.len(), 3);

        // Hit rate should be 15/18 = 83.33%
        let expected_hit_rate = (15.0 / 18.0) * 100.0;
        assert!((cache.stats().hit_rate() - expected_hit_rate).abs() < 0.1);
    }

    #[test]
    fn test_default_trait() {
        let cache = AstCache::default();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_parsed_ast_clone() {
        let ast = ParsedAst {
            file_path: "test.ts".to_string(),
            source: "code".to_string(),
        };

        let cloned = ast.clone();
        assert_eq!(cloned.file_path, "test.ts");
        assert_eq!(cloned.source, "code");
    }
}
