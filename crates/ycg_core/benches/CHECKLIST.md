# Performance Benchmarks Checklist

Use this checklist to ensure benchmarks are run correctly and results are valid.

## Pre-Run Checklist

### Prerequisites

- [ ] Rust toolchain installed and up to date
  ```bash
  rustc --version  # Should be 1.70+
  ```

- [ ] Node.js and npm installed (for SCIP generation)
  ```bash
  node --version  # Should be 16+
  npm --version
  ```

- [ ] SCIP indices generated for example projects
  ```bash
  ls -la examples/simple-ts/index.scip
  ls -la examples/nestjs-api-ts/index.scip
  ```

- [ ] No other heavy processes running
  ```bash
  # Close browsers, IDEs, etc. for consistent results
  ```

### Generate SCIP Indices (if missing)

- [ ] Generate for simple-ts
  ```bash
  cd examples/simple-ts
  npx @sourcegraph/scip-typescript index .
  cd ../..
  ```

- [ ] Generate for nestjs-api-ts
  ```bash
  cd examples/nestjs-api-ts
  npx @sourcegraph/scip-typescript index .
  cd ../..
  ```

- [ ] Verify SCIP files exist and are not empty
  ```bash
  ls -lh examples/*/index.scip
  # Should show files > 0 bytes
  ```

## Running Benchmarks

### Option 1: Quick Test Run

- [ ] Compile benchmarks
  ```bash
  cd crates/ycg_core
  cargo bench --bench granularity_benchmarks --no-run
  ```

- [ ] Run with reduced samples (fast)
  ```bash
  cargo bench --bench granularity_benchmarks -- --sample-size 10
  ```

- [ ] Check for errors in output

### Option 2: Full Benchmark Run

- [ ] Run all benchmarks with default settings
  ```bash
  cd crates/ycg_core
  cargo bench --bench granularity_benchmarks 2>&1 | tee benchmark_results.txt
  ```

- [ ] Wait for completion (~10-15 minutes)

- [ ] Check for any errors or warnings

### Option 3: Using Helper Script

- [ ] Make script executable
  ```bash
  chmod +x crates/ycg_core/benches/run_benchmarks.sh
  ```

- [ ] Run script
  ```bash
  cd crates/ycg_core/benches
  ./run_benchmarks.sh
  ```

- [ ] Review output for any issues

## Post-Run Checklist

### Verify Results

- [ ] Check that all benchmark groups completed
  - [ ] processing_time_simple_ts
  - [ ] processing_time_nestjs
  - [ ] token_savings
  - [ ] memory_usage
  - [ ] ast_caching
  - [ ] throughput

- [ ] Verify no benchmarks were skipped
  ```bash
  grep "Skipping" benchmark_results.txt
  # Should return no results
  ```

- [ ] Check for compilation errors
  ```bash
  grep "error" benchmark_results.txt
  # Should return no results
  ```

### Analyze Results

- [ ] Run Python analysis script
  ```bash
  cd crates/ycg_core/benches
  python3 analyze_benchmarks.py benchmark_results.txt
  ```

- [ ] Verify Requirements 10.1 and 10.2 pass
  - [ ] REQ.10.1: Level 1 ≤ 110% of Level 0
  - [ ] REQ.10.2: Level 2 ≤ 125% of Level 0

- [ ] Review token savings analysis
  - [ ] Level 0 baseline established
  - [ ] Level 1 overhead calculated
  - [ ] Level 2 overhead calculated

### Review HTML Reports

- [ ] Open HTML report
  ```bash
  open target/criterion/report/index.html
  # or: xdg-open target/criterion/report/index.html
  ```

- [ ] Check each benchmark group
  - [ ] Review timing distributions
  - [ ] Check confidence intervals
  - [ ] Look for outliers

- [ ] Compare with previous runs (if available)
  - [ ] Check for regressions
  - [ ] Verify improvements

### Validate Performance Targets

- [ ] Level 1 processing time ≤ 110% of Level 0
  ```
  Expected: ~106% ✅
  Actual: ____%
  Status: [ ] PASS [ ] FAIL
  ```

- [ ] Level 2 processing time ≤ 125% of Level 0
  ```
  Expected: ~117% ✅
  Actual: ____%
  Status: [ ] PASS [ ] FAIL
  ```

- [ ] Memory usage is reasonable
  ```
  Level 1 overhead: ____%
  Level 2 overhead: ____%
  Status: [ ] PASS [ ] FAIL
  ```

- [ ] Throughput is acceptable
  ```
  Level 0: ____ symbols/sec
  Level 1: ____ symbols/sec (___% of Level 0)
  Level 2: ____ symbols/sec (___% of Level 0)
  Status: [ ] PASS [ ] FAIL
  ```

## Documentation Checklist

- [ ] Results documented in benchmark_summary.md

- [ ] Any issues or anomalies noted

- [ ] Performance targets met or explained

- [ ] Recommendations for optimization (if needed)

## Troubleshooting Checklist

If benchmarks fail or results are unexpected:

- [ ] Check SCIP files are valid
  ```bash
  file examples/*/index.scip
  # Should show "data" or "protobuf"
  ```

- [ ] Verify no other processes consuming CPU
  ```bash
  top  # or htop
  ```

- [ ] Check disk space
  ```bash
  df -h
  ```

- [ ] Try with increased sample size
  ```bash
  cargo bench -- --sample-size 50
  ```

- [ ] Run individual benchmarks
  ```bash
  cargo bench -- processing_time_simple_ts
  ```

- [ ] Check for system updates or background tasks

- [ ] Review error messages carefully

## CI/CD Integration Checklist

If integrating into CI/CD:

- [ ] Add SCIP generation step

- [ ] Add benchmark execution step

- [ ] Add result analysis step

- [ ] Configure artifact storage for reports

- [ ] Set up performance regression alerts

- [ ] Document expected performance ranges

## Final Checklist

- [ ] All benchmarks completed successfully

- [ ] All requirements validated

- [ ] Results documented

- [ ] HTML reports generated

- [ ] Analysis complete

- [ ] No regressions detected

- [ ] Ready for commit/merge

## Sign-Off

**Date:** _______________

**Benchmarks Run By:** _______________

**System:** _______________

**Results:** [ ] PASS [ ] FAIL

**Notes:**
```
_________________________________________________________________
_________________________________________________________________
_________________________________________________________________
```

## Quick Reference

| Command | Purpose |
|---------|---------|
| `cargo bench --bench granularity_benchmarks` | Run all benchmarks |
| `cargo bench -- --sample-size 10` | Quick test run |
| `./run_benchmarks.sh` | Automated run with report |
| `python3 analyze_benchmarks.py benchmark_results.txt` | Analyze results |
| `open target/criterion/report/index.html` | View HTML report |

## Support

- Full guide: [BENCHMARKS_GUIDE.md](BENCHMARKS_GUIDE.md)
- Quick reference: [QUICK_REFERENCE.md](QUICK_REFERENCE.md)
- Basic usage: [README.md](README.md)
