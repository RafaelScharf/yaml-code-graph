# Baseline Output Generator - Quick Reference

## What is it?

The baseline output generator creates reference outputs from the current version of YCG for backward compatibility testing. These baselines represent the "previous version" behavior and are used to verify that new optimization features don't break existing functionality.

## Quick Start

### Generate Baselines
```bash
cd crates/ycg_core
cargo test --test baseline_generator -- --ignored
```

### Verify Backward Compatibility
```bash
cargo test --test backward_compatibility_test
```

### List Available Baselines
```bash
ls tests/fixtures/baseline/*.yaml
```

## What Gets Generated?

The generator creates 6 baseline files:

| File | Project | Level of Detail | Purpose |
|------|---------|----------------|---------|
| `simple_ts_low.yaml` | simple-ts | Low | Minimal detail baseline |
| `simple_ts_medium.yaml` | simple-ts | Medium | Standard detail baseline |
| `simple_ts_high.yaml` | simple-ts | High | Maximum detail baseline |
| `nestjs_low.yaml` | nestjs-api-ts | Low | Framework project, minimal |
| `nestjs_medium.yaml` | nestjs-api-ts | Medium | Framework project, standard |
| `nestjs_high.yaml` | nestjs-api-ts | High | Framework project, maximum |

## Configuration Used

All baselines are generated with **default settings**:
- ✓ No compaction (`compact: false`)
- ✓ YAML format (`output_format: Yaml`)
- ✓ No framework filtering (`ignore_framework_noise: false`)
- ✓ No file filtering (empty include/exclude patterns)
- ✓ Gitignore disabled (`use_gitignore: false`)

This ensures baselines represent the behavior users expect when running YCG without any optimization flags.

## Using Baselines in Tests

### Load a Baseline
```rust
use crate::baseline_helpers::load_baseline;

let baseline = load_baseline("simple_ts_medium")?;
```

### Compare Outputs
```rust
use crate::baseline_helpers::compare_yaml_outputs;

let are_equal = compare_yaml_outputs(&baseline, &current_output)?;
assert!(are_equal, "Output should match baseline");
```

### Check if Baseline Exists
```rust
use crate::baseline_helpers::baseline_exists;

if !baseline_exists("simple_ts_medium") {
    println!("Baseline not found, skipping test");
    return Ok(());
}
```

## When to Regenerate

### ✅ DO Regenerate When:

1. **Bug Fixes in Core Logic**
   ```
   Fixed: Symbol parent resolution now correctly handles nested classes
   Action: Regenerate baselines to reflect correct behavior
   ```

2. **SCIP Protocol Updates**
   ```
   Updated: SCIP protobuf to version 0.3.0
   Action: Regenerate baselines with new protocol
   ```

3. **Intentional Default Behavior Changes**
   ```
   Changed: Default LOD now includes method signatures
   Action: Regenerate baselines and document in CHANGELOG
   ```

### ❌ DON'T Regenerate When:

1. **Adding Optimization Features**
   ```
   Added: --compact flag for graph compaction
   Action: NO regeneration needed (opt-in feature)
   ```

2. **Adding New CLI Flags**
   ```
   Added: --ignore-framework-noise flag
   Action: NO regeneration needed (opt-in feature)
   ```

3. **Implementing Optional Features**
   ```
   Added: Ad-hoc output format
   Action: NO regeneration needed (opt-in via --output-format)
   ```

## Regeneration Workflow

1. **Make Your Changes**
   ```bash
   # Edit core library files
   vim crates/ycg_core/src/lib.rs
   ```

2. **Regenerate Baselines**
   ```bash
   cd crates/ycg_core
   cargo test --test baseline_generator -- --ignored
   ```

3. **Review Changes**
   ```bash
   git diff tests/fixtures/baseline/
   ```

4. **Verify Tests Pass**
   ```bash
   cargo test --test backward_compatibility_test
   ```

5. **Commit with Explanation**
   ```bash
   git add tests/fixtures/baseline/
   git commit -m "chore: regenerate baselines after fixing symbol resolution bug
   
   - Fixed parent ID calculation for nested classes
   - All backward compatibility tests pass
   - Baselines now reflect correct behavior"
   ```

## Troubleshooting

### Problem: Baseline files not found
```
⚠ Skipping: Baseline not found
```
**Solution:**
```bash
cargo test --test baseline_generator -- --ignored
```

### Problem: SCIP index missing
```
⚠ Skipping: SCIP file not found at "../../../examples/simple-ts/index.scip"
```
**Solution:**
```bash
cd examples/simple-ts
npx @sourcegraph/scip-typescript index
```

### Problem: Backward compatibility test fails
```
❌ Backward compatibility broken!
Baseline size: 1116 bytes
Current size: 1250 bytes
```
**Solution:**
1. Check `tests/fixtures/baseline/debug_current.yaml` for differences
2. Determine if change is intentional
3. If intentional: regenerate baselines and document
4. If bug: fix the code and verify tests pass

### Problem: Tests are slow
```
test test_all_baselines_backward_compatibility ... ok (12.32s)
```
**Explanation:** This is normal. The test processes 6 SCIP indices with full graph generation. Consider running specific tests during development:
```bash
cargo test --test backward_compatibility_test test_simple_ts_medium
```

## Integration with CI/CD

### GitHub Actions Example
```yaml
- name: Generate Baselines (if needed)
  run: |
    cd crates/ycg_core
    cargo test --test baseline_generator -- --ignored

- name: Run Backward Compatibility Tests
  run: |
    cd crates/ycg_core
    cargo test --test backward_compatibility_test
```

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

cd crates/ycg_core
cargo test --test backward_compatibility_test --quiet

if [ $? -ne 0 ]; then
    echo "❌ Backward compatibility tests failed!"
    echo "If this is intentional, regenerate baselines:"
    echo "  cargo test --test baseline_generator -- --ignored"
    exit 1
fi
```

## FAQ

**Q: Why are baselines committed to git?**
A: So CI/CD can verify backward compatibility without regenerating them each time.

**Q: How big are the baseline files?**
A: Simple TypeScript: ~1-2 KB, NestJS: ~9-21 KB. Total: ~45 KB.

**Q: Can I add more test cases?**
A: Yes! Edit `baseline_generator.rs` and add new `BaselineTestCase` entries.

**Q: What if I want to test with optimizations enabled?**
A: Baselines are only for default behavior. Optimization tests should use property-based testing.

**Q: How do I debug a failing comparison?**
A: The test saves current output to `tests/fixtures/baseline/debug_current.yaml` for manual inspection.

## Related Files

- `baseline_generator.rs` - Generates baseline outputs
- `baseline_helpers.rs` - Helper functions for loading and comparing
- `backward_compatibility_test.rs` - Integration tests using baselines
- `fixtures/baseline/README.md` - Documentation for baseline fixtures
- `fixtures/baseline/*.yaml` - The actual baseline files

## Requirements Validated

This baseline system validates:
- **Requirement 7.1**: No optimization flags → identical output
- **Requirement 7.2**: No config file → identical output
- **Requirement 7.3**: Same input + default settings → byte-identical output
- **Requirement 7.4**: Existing CLI flags maintain previous behavior

## Property Tests Using Baselines

- **Property 4**: Backward Compatibility Without Flags
  - For any input SCIP index, when no optimization flags are provided, the output SHALL be byte-identical to the baseline

## Support

For questions or issues:
1. Check this guide
2. Read `tests/fixtures/baseline/README.md`
3. Review `tests/README.md`
4. Check the design document: `.kiro/specs/ycg-token-optimization/design.md`
