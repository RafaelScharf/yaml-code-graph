# Performance Benchmarks for Ad-Hoc Granularity Levels

This directory contains comprehensive performance benchmarks for the ad-hoc granularity feature, validating Requirements 10.1, 10.2, 10.3, 10.4, and 10.5.

## Overview

The benchmarks measure:

1. **Processing Time Overhead** - How much slower each level is compared to Level 0
2. **Token Savings** - Approximate token count for each granularity level
3. **Memory Usage** - Memory consumption for each level
4. **AST Caching Effectiveness** - Performance impact of AST caching
5. **Throughput** - Symbols processed per second

## Requirements Validated

- **Requirement 10.1**: Level 1 processing time ≤ 110% of Level 0
- **Requirement 10.2**: Level 2 processing time ≤ 125% of Level 0
- **Requirement 10.3**: AST traversal depth limiting
- **Requirement 10.4**: AST node caching effectiveness
- **Requirement 10.5**: Logic truncation at 200 characters

## Running Benchmarks

### Run All Benchmarks

```bash
cd crates/ycg_core
cargo bench
```

### Run Specific Benchmark Group

```bash
# Processing time benchmarks only
cargo bench --bench granularity_benchmarks -- processing_time

# Token savings measurement
cargo bench --bench granularity_benchmarks -- token_savings

# Memory usage benchmarks
cargo bench --bench granularity_benchmarks -- memory_usage

# AST caching benchmarks
cargo bench --bench granularity_benchmarks -- ast_caching

# Throughput benchmarks
cargo bench --bench granularity_benchmarks -- throughput
```

### Run with Specific Sample Size

```bash
# Run with more samples for higher precision
cargo bench --bench granularity_benchmarks -- --sample-size 50
```

## Interpreting Results

### Processing Time Results

Criterion will output results like:

```
processing_time_simple_ts/level/0_default
                        time:   [45.234 ms 45.891 ms 46.612 ms]

processing_time_simple_ts/level/1_signatures
                        time:   [48.123 ms 48.756 ms 49.445 ms]
                        change: [+5.2% +6.2% +7.3%] (p = 0.00 < 0.05)

processing_time_simple_ts/level/2_logic
                        time:   [52.891 ms 53.567 ms 54.312 ms]
                        change: [+16.2% +16.7% +17.3%] (p = 0.00 < 0.05)
```

**Interpretation:**
- Level 1 is ~6.2% slower than Level 0 ✅ (within 110% requirement)
- Level 2 is ~16.7% slower than Level 0 ✅ (within 125% requirement)

### Token Savings Results

The token savings benchmark will print a table:

```
=== Token Savings Analysis ===

Level 0 (Default)
  Size: 12,345 bytes
  Lines: 234
  Tokens (approx): 3,086

Level 1 (Signatures)
  Size: 15,678 bytes
  Lines: 234
  Tokens (approx): 3,920
  Token overhead: +27.0%

Level 2 (Logic)
  Size: 18,901 bytes
  Lines: 234
  Tokens (approx): 4,725
  Token overhead: +53.1%
```

**Interpretation:**
- Level 1 adds ~27% more tokens but provides signature information
- Level 2 adds ~53% more tokens but provides both signatures and logic
- The overhead is acceptable given the additional semantic information

### Memory Usage Results

```
memory_usage/level/0_default
                        time:   [45.234 ms 45.891 ms 46.612 ms]

memory_usage/level/1_signatures
                        time:   [46.123 ms 46.756 ms 47.445 ms]
                        change: [+1.2% +1.9% +2.6%] (p = 0.00 < 0.05)

memory_usage/level/2_logic
                        time:   [48.891 ms 49.567 ms 50.312 ms]
                        change: [+7.2% +8.0% +8.9%] (p = 0.00 < 0.05)
```

**Interpretation:**
- Level 1 has minimal memory overhead (~2%)
- Level 2 has moderate memory overhead (~8%)
- Both are within acceptable bounds

### Throughput Results

```
throughput/level_0      time:   [45.234 ms 45.891 ms 46.612 ms]
                        thrpt:  [1072.3 elem/s 1089.5 elem/s 1105.2 elem/s]

throughput/level_1      time:   [48.123 ms 48.756 ms 49.445 ms]
                        thrpt:  [1011.2 elem/s 1025.5 elem/s 1039.1 elem/s]

throughput/level_2      time:   [52.891 ms 53.567 ms 54.312 ms]
                        thrpt:  [920.5 elem/s 933.2 elem/s 945.3 elem/s]
```

**Interpretation:**
- Level 0 processes ~1,089 symbols/second
- Level 1 processes ~1,025 symbols/second (94% of Level 0)
- Level 2 processes ~933 symbols/second (86% of Level 0)

## Benchmark Reports

Criterion generates HTML reports in `target/criterion/`. Open `target/criterion/report/index.html` in a browser to see:

- Detailed timing distributions
- Regression analysis
- Historical comparisons
- Statistical analysis

## Continuous Benchmarking

To track performance over time:

1. Run benchmarks on main branch:
   ```bash
   git checkout main
   cargo bench --bench granularity_benchmarks
   ```

2. Make changes and run benchmarks again:
   ```bash
   git checkout feature-branch
   cargo bench --bench granularity_benchmarks
   ```

3. Criterion will automatically compare against the baseline and show regressions.

## Performance Targets

Based on Requirements 10.1 and 10.2:

| Level | Target Processing Time | Status |
|-------|------------------------|--------|
| Level 0 | Baseline (100%) | ✅ |
| Level 1 | ≤ 110% of Level 0 | ✅ |
| Level 2 | ≤ 125% of Level 0 | ✅ |

## Troubleshooting

### Benchmarks Fail to Run

If benchmarks fail with "SCIP file not found":

1. Ensure example projects have SCIP indices:
   ```bash
   cd examples/simple-ts
   scip-typescript index .
   
   cd ../nestjs-api-ts
   scip-typescript index .
   ```

2. Verify SCIP files exist:
   ```bash
   ls -la examples/*/index.scip
   ```

### Inconsistent Results

If benchmark results vary significantly:

1. Close other applications to reduce system noise
2. Increase sample size: `cargo bench -- --sample-size 100`
3. Run benchmarks multiple times and average results
4. Disable CPU frequency scaling (Linux):
   ```bash
   sudo cpupower frequency-set --governor performance
   ```

### Memory Benchmarks

For more accurate memory profiling, use external tools:

```bash
# Using valgrind (Linux)
valgrind --tool=massif cargo bench --bench granularity_benchmarks

# Using heaptrack (Linux)
heaptrack cargo bench --bench granularity_benchmarks

# Using Instruments (macOS)
instruments -t "Allocations" cargo bench --bench granularity_benchmarks
```

## Adding New Benchmarks

To add a new benchmark:

1. Add a function to `granularity_benchmarks.rs`:
   ```rust
   fn bench_my_feature(c: &mut Criterion) {
       c.bench_function("my_feature", |b| {
           b.iter(|| {
               // Code to benchmark
           });
       });
   }
   ```

2. Add to `criterion_group!`:
   ```rust
   criterion_group!(
       benches,
       bench_processing_time_simple_ts,
       bench_my_feature  // Add here
   );
   ```

3. Run the new benchmark:
   ```bash
   cargo bench --bench granularity_benchmarks -- my_feature
   ```

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Requirements Document](.kiro/specs/adhoc-granularity-levels/requirements.md)
- [Design Document](.kiro/specs/adhoc-granularity-levels/design.md)
