# AST Caching Implementation

## Overview

The AST caching system provides performance optimization for signature and logic extraction by caching parsed Abstract Syntax Trees (ASTs) per file. This prevents redundant parsing when processing multiple symbols from the same source file.

## Performance Benefits

### Without Caching
- File with 10 methods = 10 full parses of the same file
- File with 50 methods = 50 full parses of the same file
- Large codebase with 1000 symbols across 100 files = 1000 parses

### With Caching
- File with 10 methods = 1 parse + 9 cache hits (90% hit rate)
- File with 50 methods = 1 parse + 49 cache hits (98% hit rate)
- Large codebase with 1000 symbols across 100 files = 100 parses + 900 cache hits (90% hit rate)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│              AdHocSerializerV2                          │
│                                                         │
│  ┌───────────────────────────────────────────────┐    │
│  │  serialize_graph_with_cache()                 │    │
│  │  - Creates AstCache                           │    │
│  │  - Pre-populates cache for each file          │    │
│  │  - Passes cache to extractors                 │    │
│  └───────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                    AstCache                             │
│                                                         │
│  ┌───────────────────────────────────────────────┐    │
│  │  HashMap<FilePath, ParsedAst>                 │    │
│  │  - Stores parsed ASTs by file path            │    │
│  │  - Tracks cache statistics (hits/misses)      │    │
│  │  - Provides get_or_parse() interface          │    │
│  └───────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          ▼                             ▼
┌──────────────────────┐    ┌──────────────────────┐
│ SignatureExtractor   │    │  LogicExtractor      │
│                      │    │                      │
│ extract_signature_   │    │ extract_logic_       │
│ with_cache()         │    │ with_cache()         │
│ - Uses cached AST    │    │ - Uses cached AST    │
│ - No redundant parse │    │ - No redundant parse │
└──────────────────────┘    └──────────────────────┘
```

## Usage

### Basic Usage

```rust
use ycg_core::ast_cache::AstCache;

let mut cache = AstCache::new();

// First access parses the file
let ast1 = cache.get_or_parse("file.ts", source_code);

// Subsequent accesses use cached AST
let ast2 = cache.get_or_parse("file.ts", source_code);
```

### With Serializer

```rust
use ycg_core::adhoc_serializer_v2::AdHocSerializerV2;
use ycg_core::ast_cache::AstCache;
use ycg_core::model::AdHocGranularity;

let serializer = AdHocSerializerV2::new(AdHocGranularity::InlineLogic);
let mut cache = AstCache::new();

// Serialize graph with caching enabled
let adhoc_graph = serializer.serialize_graph_with_cache(
    &graph,
    &sources,
    &mut cache
);

// Check cache statistics
let stats = cache.stats();
println!("Cache hit rate: {:.2}%", stats.hit_rate());
println!("Total accesses: {}", stats.total_accesses());
```

### Pre-allocating Cache

For better performance when you know the approximate number of files:

```rust
// Pre-allocate for 100 files
let mut cache = AstCache::with_capacity(100);
```

## Cache Statistics

The cache tracks performance metrics:

- **Hits**: Number of times AST was found in cache
- **Misses**: Number of times AST had to be parsed
- **Hit Rate**: Percentage of cache hits (hits / total accesses)
- **Cached Files**: Number of unique files currently cached

```rust
let stats = cache.stats();
println!("Hits: {}", stats.hits);
println!("Misses: {}", stats.misses);
println!("Hit Rate: {:.2}%", stats.hit_rate());
println!("Cached Files: {}", stats.cached_files);
```

## Implementation Details

### Current State (Placeholder)

The current implementation is a placeholder that stores source code without actual tree-sitter parsing. This allows the caching infrastructure to be in place and tested before tree-sitter integration is complete.

```rust
pub struct ParsedAst {
    pub file_path: String,
    pub source: String,
    // Future: tree_sitter::Tree will be added here
}
```

### Future State (With Tree-Sitter)

When tree-sitter integration is complete, the `ParsedAst` struct will include the actual parsed tree:

```rust
pub struct ParsedAst {
    pub file_path: String,
    pub source: String,
    pub tree: tree_sitter::Tree,  // Actual parsed AST
}
```

The extractors will then use the cached tree for signature and logic extraction:

```rust
pub fn extract_signature_with_cache(
    node: &SymbolNode,
    file_path: &str,
    cache: &mut AstCache,
) -> Option<String> {
    // Get cached AST
    let ast = cache.get_or_parse(file_path, source)?;
    
    // Use tree-sitter to extract signature from AST
    let signature = extract_from_tree(&ast.tree, node)?;
    
    Some(compact_signature(&signature))
}
```

## Performance Characteristics

### Time Complexity
- `get_or_parse()`: O(1) for cache hit, O(n) for cache miss (where n is file size)
- `get()`: O(1)
- `contains()`: O(1)
- `clear()`: O(m) (where m is number of cached files)

### Space Complexity
- O(m * k) where:
  - m = number of unique files
  - k = average AST size per file

### Expected Performance Impact

Based on Requirements 10.1 and 10.2:

- **Level 1 (Signatures)**: ≤ 110% of Level 0 time
- **Level 2 (Logic)**: ≤ 125% of Level 0 time

The caching system helps achieve these targets by:
1. Eliminating redundant parsing (major cost)
2. Amortizing parse cost across all symbols in a file
3. Providing O(1) lookup for cached ASTs

## Testing

The cache includes comprehensive unit tests covering:

- Basic cache operations (get, contains, clear)
- Cache statistics tracking
- Hit/miss counting
- Realistic usage patterns
- Multiple file scenarios

Run tests with:

```bash
cargo test --package ycg_core --lib ast_cache
```

## Requirements Validation

This implementation validates:

- **Requirement 10.3**: Cache parsed AST nodes per file ✓
- **Requirement 10.4**: Reuse cached AST for multiple symbols ✓

## Future Enhancements

1. **LRU Eviction**: Add cache size limits with LRU eviction for very large codebases
2. **Persistent Cache**: Save parsed ASTs to disk for cross-run caching
3. **Parallel Parsing**: Parse multiple files in parallel using rayon
4. **Incremental Parsing**: Use tree-sitter's incremental parsing for file updates
5. **Memory Profiling**: Add memory usage tracking to cache statistics

## Related Files

- `crates/ycg_core/src/ast_cache.rs` - Cache implementation
- `crates/ycg_core/src/adhoc_serializer_v2.rs` - Serializer using cache
- `crates/ycg_core/src/signature_extractor.rs` - Signature extraction with cache
- `crates/ycg_core/src/logic_extractor.rs` - Logic extraction with cache
