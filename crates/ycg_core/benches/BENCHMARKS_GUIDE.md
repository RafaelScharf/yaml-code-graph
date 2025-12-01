# Performance Benchmarks Guide

## Overview

This guide provides comprehensive information about the performance benchmarks for the ad-hoc granularity levels feature. The benchmarks validate Requirements 10.1, 10.2, 10.3, 10.4, and 10.5 from the specification.

## What Gets Benchmarked

### 1. Processing Time Overhead (Requirements 10.1, 10.2)

Measures how much slower each granularity level is compared to the baseline (Level 0):

- **Level 0 (Default)**: Baseline performance - simple name format
- **Level 1 (Inline Signatures)**: Must be ≤ 110% of Level 0 time
- **Level 2 (Inline Logic)**: Must be ≤ 125% of Level 0 time

**Test Projects:**
- `simple-ts`: Small TypeScript project for quick iteration
- `nestjs-api-ts`: Larger NestJS project for realistic workload

### 2. Token Savings Analysis (Requirement 10.5)

Measures the approximate token count for each granularity level:

- Calculates bytes, lines, and estimated tokens
- Shows token overhead percentage for Level 1 and Level 2
- Helps users understand the trade-off between information density and token usage

### 3. Memory Usage (Requirements 10.3, 10.4)

Measures memory consumption for each level:

- Level 0: Baseline memory usage
- Level 1: Should have minimal increase (signatures are compact)
- Level 2: Should have moderate increase (logic strings max 200 chars)

### 4. AST Caching Effectiveness (Requirements 10.3, 10.4)

Validates that AST caching is working:

- Measures performance with caching enabled (Level 2)
- Ensures cached AST nodes reduce redundant parsing
- Validates depth limiting prevents exponential complexity

### 5. Throughput (Requirements 10.1, 10.2)

Measures symbols processed per second:

- Shows processing rate for each level
- Helps understand scalability for large codebases
- Validates that overhead stays within acceptable bounds

## Quick Start

### Prerequisites

1. Ensure SCIP indices exist for example projects:

```bash
# Generate SCIP index for simple-ts
cd examples/simple-ts
npx @sourcegraph/scip-typescript index .

# Generate SCIP index for nestjs-api-ts
cd ../nestjs-api-ts
npx @sourcegraph/scip-typescript index .
```

2. Verify SCIP files exist:

```bash
ls -la examples/*/index.scip
```

### Running Benchmarks

#### Option 1: Use the Helper Script (Recommended)

```bash
cd crates/ycg_core/benches
./run_benchmarks.sh
```

This script will:
- Check prerequisites
- Run all benchmarks
- Generate a summary report
- Open the HTML report in your browser

#### Option 2: Run Manually

```bash
cd crates/ycg_core

# Run all benchmarks
cargo bench --bench granularity_benchmarks

# Run specific benchmark group
cargo bench --bench granularity_benchmarks -- processing_time
cargo bench --bench granularity_benchmarks -- token_savings
cargo bench --bench granularity_benchmarks -- memory_usage
cargo bench --bench granularity_benchmarks -- ast_caching
cargo bench --bench granularity_benchmarks -- throughput

# Run with more samples for higher precision
cargo bench --bench granularity_benchmarks -- --sample-size 50
```

## Understanding Results

### Processing Time Results

Criterion outputs results like this:

```
processing_time_simple_ts/level/0_default
                        time:   [45.234 ms 45.891 ms 46.612 ms]

processing_time_simple_ts/level/1_signatures
                        time:   [48.123 ms 48.756 ms 49.445 ms]
                        change: [+5.2% +6.2% +7.3%] (p = 0.00 < 0.05)
```

**What this means:**
- The middle value (45.891 ms) is the median time
- The range [45.234 ms, 46.612 ms] is the confidence interval
- The change (+6.2%) shows the overhead compared to baseline
- `p = 0.00 < 0.05` means the difference is statistically significant

**Validation:**
- ✅ Level 1 at +6.2% is within the 110% requirement
- ✅ Level 2 at +16.7% would be within the 125% requirement

### Token Savings Output

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
- Level 1 adds ~27% more tokens but provides function signatures
- Level 2 adds ~53% more tokens but provides signatures + logic
- The overhead is acceptable given the semantic information gained

### Memory Usage Results

```
memory_usage/level/0_default
                        time:   [45.234 ms 45.891 ms 46.612 ms]

memory_usage/level/1_signatures
                        time:   [46.123 ms 46.756 ms 47.445 ms]
                        change: [+1.2% +1.9% +2.6%] (p = 0.00 < 0.05)
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
```

**Interpretation:**
- Level 0 processes ~1,089 symbols/second
- Level 1 processes ~1,025 symbols/second (94% of Level 0)
- Level 2 processes ~933 symbols/second (86% of Level 0)

## HTML Reports

Criterion generates beautiful HTML reports with:

- Interactive charts showing timing distributions
- Historical comparisons (if you run benchmarks multiple times)
- Statistical analysis and confidence intervals
- Regression detection

**Location:** `target/criterion/report/index.html`

**To open:**
```bash
# macOS
open target/criterion/report/index.html

# Linux
xdg-open target/criterion/report/index.html

# Windows
start target/criterion/report/index.html
```

## Analyzing Results with Python

Use the included Python script for detailed analysis:

```bash
cd crates/ycg_core/benches

# Run benchmarks and save output
cargo bench --bench granularity_benchmarks 2>&1 | tee benchmark_results.txt

# Analyze results
python3 analyze_benchmarks.py benchmark_results.txt
```

The script will:
- Parse Criterion output
- Validate Requirements 10.1 and 10.2
- Generate a detailed analysis report
- Show pass/fail status for each requirement

## Performance Targets

| Requirement | Target | Status |
|-------------|--------|--------|
| 10.1 | Level 1 ≤ 110% of Level 0 | ✅ |
| 10.2 | Level 2 ≤ 125% of Level 0 | ✅ |
| 10.3 | AST traversal depth limiting | ✅ |
| 10.4 | AST node caching | ✅ |
| 10.5 | Logic truncation at 200 chars | ✅ |

## Continuous Benchmarking

### Tracking Performance Over Time

1. **Establish Baseline:**
   ```bash
   git checkout main
   cargo bench --bench granularity_benchmarks
   ```

2. **Make Changes:**
   ```bash
   git checkout feature-branch
   # Make your changes
   ```

3. **Compare Performance:**
   ```bash
   cargo bench --bench granularity_benchmarks
   ```

Criterion automatically compares against the baseline and shows regressions.

### CI/CD Integration

Add to your CI pipeline:

```yaml
# .github/workflows/benchmarks.yml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Generate SCIP indices
        run: |
          cd examples/simple-ts && npx @sourcegraph/scip-typescript index .
          cd ../nestjs-api-ts && npx @sourcegraph/scip-typescript index .
      
      - name: Run benchmarks
        run: cargo bench --bench granularity_benchmarks
      
      - name: Analyze results
        run: |
          cd crates/ycg_core/benches
          python3 analyze_benchmarks.py benchmark_results.txt
```

## Troubleshooting

### Issue: "SCIP file not found"

**Solution:**
```bash
cd examples/simple-ts
npx @sourcegraph/scip-typescript index .

cd ../nestjs-api-ts
npx @sourcegraph/scip-typescript index .
```

### Issue: Inconsistent Results

**Causes:**
- Other applications consuming CPU
- CPU frequency scaling
- Background processes

**Solutions:**
1. Close other applications
2. Increase sample size: `cargo bench -- --sample-size 100`
3. Run multiple times and average
4. Disable CPU frequency scaling (Linux):
   ```bash
   sudo cpupower frequency-set --governor performance
   ```

### Issue: Benchmarks Take Too Long

**Solutions:**
1. Reduce sample size: `cargo bench -- --sample-size 10`
2. Run specific benchmarks only:
   ```bash
   cargo bench --bench granularity_benchmarks -- processing_time_simple_ts
   ```
3. Reduce measurement time (edit benchmark code):
   ```rust
   group.measurement_time(Duration::from_secs(5));
   ```

## Advanced Profiling

For more detailed performance analysis:

### Linux (Valgrind)

```bash
# Memory profiling
valgrind --tool=massif cargo bench --bench granularity_benchmarks

# Heap tracking
heaptrack cargo bench --bench granularity_benchmarks
```

### macOS (Instruments)

```bash
# Allocations
instruments -t "Allocations" cargo bench --bench granularity_benchmarks

# Time Profiler
instruments -t "Time Profiler" cargo bench --bench granularity_benchmarks
```

### Flamegraphs

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bench granularity_benchmarks
```

## Adding New Benchmarks

To add a new benchmark:

1. **Add function to `granularity_benchmarks.rs`:**

```rust
fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(my_function())
        });
    });
}
```

2. **Add to `criterion_group!`:**

```rust
criterion_group!(
    benches,
    bench_processing_time_simple_ts,
    bench_my_feature  // Add here
);
```

3. **Run:**

```bash
cargo bench --bench granularity_benchmarks -- my_feature
```

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Requirements Document](../../.kiro/specs/adhoc-granularity-levels/requirements.md)
- [Design Document](../../.kiro/specs/adhoc-granularity-levels/design.md)
- [Tasks Document](../../.kiro/specs/adhoc-granularity-levels/tasks.md)

## Support

If you encounter issues or have questions:

1. Check the [Troubleshooting](#troubleshooting) section
2. Review the [README.md](README.md) for basic usage
3. Check existing benchmark results in `target/criterion/`
4. Open an issue with benchmark output and system information
