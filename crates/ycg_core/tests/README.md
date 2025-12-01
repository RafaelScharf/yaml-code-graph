# YCG Core Tests

This directory contains tests for the YCG core library, including unit tests, integration tests, and property-based tests.

## Test Structure

```
tests/
├── baseline_generator.rs       # Generates reference outputs for backward compatibility
├── baseline_helpers.rs         # Helper functions for baseline comparison
├── backward_compatibility_test.rs  # Integration tests for Requirement 7
├── fixtures/
│   └── baseline/              # Reference outputs from current version
│       ├── README.md          # Documentation for baseline fixtures
│       ├── simple_ts_*.yaml   # Baselines for simple TypeScript project
│       └── nestjs_*.yaml      # Baselines for NestJS project
```

## Running Tests

### All Tests
```bash
cargo test
```

### Specific Test Module
```bash
cargo test --test baseline_generator
cargo test --test backward_compatibility_test
```

### Generate Baseline Outputs
```bash
cargo test --test baseline_generator -- --ignored
```

This command:
1. Processes example projects with default settings
2. Generates YAML outputs for Low, Medium, and High detail levels
3. Stores them in `tests/fixtures/baseline/`

### Backward Compatibility Tests
```bash
cargo test --test backward_compatibility_test
```

These tests verify **Requirement 7: Backward Compatibility**:
- No optimization flags → output identical to baseline
- No config file → output identical to baseline
- Same input + default settings → byte-identical output

## Baseline Management

### When to Regenerate Baselines

✅ **DO regenerate when:**
- Fixing bugs in core graph generation
- Updating SCIP parsing for new language features
- Making intentional changes to default output format
- Updating to new SCIP protocol version

❌ **DON'T regenerate when:**
- Adding new optimization features (should not affect default output)
- Adding new CLI flags (should be opt-in)
- Implementing framework noise reduction (should be opt-in)

### Regeneration Process

1. Make your changes to the core library
2. Run the baseline generator:
   ```bash
   cd crates/ycg_core
   cargo test --test baseline_generator -- --ignored
   ```
3. Review the changes:
   ```bash
   git diff tests/fixtures/baseline/
   ```
4. Verify backward compatibility tests still pass:
   ```bash
   cargo test --test backward_compatibility_test
   ```
5. Commit the updated baselines with a clear explanation

### Baseline Validation

After regenerating, verify:
- ✓ All 6 baseline files exist (simple_ts_* and nestjs_*)
- ✓ Files are not empty
- ✓ YAML structure is valid
- ✓ Token counts are reasonable
- ✓ Backward compatibility tests pass

## Property-Based Testing

Property-based tests use the `proptest` crate and are configured to run 100 iterations minimum.

Each property test is tagged with a comment referencing the design document:
```rust
// **Feature: ycg-token-optimization, Property 4: Backward Compatibility Without Flags**
```

## Test Coverage

Current test coverage includes:

### Unit Tests
- Configuration loading and merging
- File filtering with glob patterns
- Symbol classification
- Framework pattern detection
- Format transformation

### Integration Tests
- Full pipeline with various configurations
- Backward compatibility verification
- Baseline comparison

### Property-Based Tests (Planned)
- Property 4: Backward Compatibility Without Flags
- Property 12: CLI Precedence Over Config File
- Property 13: YAML Output Validity
- Property 14: Graph Edge Referential Integrity

## Troubleshooting

### Baseline files not found
```
⚠ Skipping: Baseline not found. Generate with: cargo test --test baseline_generator -- --ignored
```
**Solution:** Run the baseline generator as shown above.

### SCIP file not found
```
⚠ Skipping: SCIP file not found at "../../../examples/simple-ts/index.scip"
```
**Solution:** Generate SCIP indices for example projects:
```bash
cd examples/simple-ts
npx @sourcegraph/scip-typescript index

cd ../nestjs-api-ts
npx @sourcegraph/scip-typescript index
```

### Backward compatibility test fails
```
❌ Backward compatibility broken!
```
**Solution:** 
1. Check if you intentionally changed default behavior
2. If yes, regenerate baselines and document the change
3. If no, investigate the regression and fix it

## CI/CD Integration

In CI/CD pipelines:

1. **Generate baselines** (one-time or when updating):
   ```bash
   cargo test --test baseline_generator -- --ignored
   ```

2. **Run backward compatibility tests** (every build):
   ```bash
   cargo test --test backward_compatibility_test
   ```

3. **Fail the build** if backward compatibility is broken

## Contributing

When adding new tests:

1. Follow the existing test structure
2. Add property-based tests for universal properties
3. Add unit tests for specific examples and edge cases
4. Update this README if adding new test categories
5. Ensure tests are deterministic and reproducible

## References

- Design Document: `.kiro/specs/ycg-token-optimization/design.md`
- Requirements: `.kiro/specs/ycg-token-optimization/requirements.md`
- Tasks: `.kiro/specs/ycg-token-optimization/tasks.md`
