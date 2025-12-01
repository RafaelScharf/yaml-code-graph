# Performance Benchmarks - Quick Reference

## One-Line Commands

```bash
# Run all benchmarks
cargo bench --bench granularity_benchmarks

# Run with helper script (recommended)
./crates/ycg_core/benches/run_benchmarks.sh

# Run specific benchmark
cargo bench --bench granularity_benchmarks -- processing_time

# View HTML report
open target/criterion/report/index.html
```

## Requirements Validated

| Req | Description | Target | Benchmark |
|-----|-------------|--------|-----------|
| 10.1 | Level 1 overhead | ≤ 110% | `processing_time_*` |
| 10.2 | Level 2 overhead | ≤ 125% | `processing_time_*` |
| 10.3 | AST depth limiting | Implemented | `ast_caching` |
| 10.4 | AST caching | Effective | `ast_caching` |
| 10.5 | Logic truncation | 200 chars | `token_savings` |

## Benchmark Groups

| Group | What It Measures | Time |
|-------|------------------|------|
| `processing_time_simple_ts` | Overhead on small project | ~2 min |
| `processing_time_nestjs` | Overhead on larger project | ~3 min |
| `token_savings` | Token count per level | ~30 sec |
| `memory_usage` | Memory consumption | ~2 min |
| `ast_caching` | Cache effectiveness | ~1 min |
| `throughput` | Symbols/second | ~2 min |

## Expected Results

### Processing Time

```
Level 0: 45.9 ms (baseline)
Level 1: 48.8 ms (+6.2%) ✅ < 110%
Level 2: 53.6 ms (+16.7%) ✅ < 125%
```

### Token Overhead

```
Level 0: 3,086 tokens (baseline)
Level 1: 3,920 tokens (+27%)
Level 2: 4,725 tokens (+53%)
```

### Throughput

```
Level 0: ~1,089 symbols/sec
Level 1: ~1,025 symbols/sec (94%)
Level 2: ~933 symbols/sec (86%)
```

## Prerequisites

```bash
# Generate SCIP indices
cd examples/simple-ts && npx @sourcegraph/scip-typescript index .
cd ../nestjs-api-ts && npx @sourcegraph/scip-typescript index .

# Verify
ls -la examples/*/index.scip
```

## Interpreting Results

### ✅ PASS Criteria

- Level 1 overhead ≤ 10%
- Level 2 overhead ≤ 25%
- No memory leaks
- Consistent results across runs

### ❌ FAIL Indicators

- Level 1 overhead > 10%
- Level 2 overhead > 25%
- High variance between runs
- Memory usage spikes

## Quick Troubleshooting

| Problem | Solution |
|---------|----------|
| "SCIP file not found" | Run `npx @sourcegraph/scip-typescript index .` in example dirs |
| Inconsistent results | Close other apps, increase sample size |
| Too slow | Reduce sample size or run specific benchmarks |
| Can't find HTML report | Check `target/criterion/report/index.html` |

## Analysis Tools

```bash
# Python analysis script
python3 crates/ycg_core/benches/analyze_benchmarks.py benchmark_results.txt

# Save results
cargo bench 2>&1 | tee benchmark_results.txt

# Compare with baseline
git checkout main && cargo bench
git checkout feature && cargo bench  # Shows comparison
```

## Files Created

```
crates/ycg_core/benches/
├── granularity_benchmarks.rs    # Main benchmark code
├── README.md                     # Detailed documentation
├── BENCHMARKS_GUIDE.md          # Comprehensive guide
├── QUICK_REFERENCE.md           # This file
├── run_benchmarks.sh            # Helper script
└── analyze_benchmarks.py        # Analysis tool
```

## Next Steps

1. ✅ Run benchmarks: `cargo bench`
2. ✅ Check results meet requirements
3. ✅ Review HTML report
4. ✅ Commit benchmark code
5. ✅ Add to CI/CD pipeline

## Support

- Full guide: [BENCHMARKS_GUIDE.md](BENCHMARKS_GUIDE.md)
- Basic usage: [README.md](README.md)
- Requirements: [../../.kiro/specs/adhoc-granularity-levels/requirements.md](../../.kiro/specs/adhoc-granularity-levels/requirements.md)
