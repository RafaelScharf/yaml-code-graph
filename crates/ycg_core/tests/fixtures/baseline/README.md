# Baseline Output Fixtures

This directory contains reference outputs generated from the current version of YCG for backward compatibility testing.

## Purpose

These baseline files are used to verify **Requirement 7: Backward Compatibility**:
- 7.1: When no optimization flags are provided, output should be identical to previous version
- 7.2: When no configuration file exists, output should be identical to previous version  
- 7.3: Same input with default settings should produce byte-identical output

## Generation

To generate or regenerate baseline files, run:

```bash
cd crates/ycg_core
cargo test --test baseline_generator -- --ignored
```

This will:
1. Process all example projects with default settings (no optimizations)
2. Generate YAML outputs for different Level of Detail settings
3. Store them in this directory as reference baselines

## Test Cases

The baseline generator creates outputs for:

### Simple TypeScript Project (`examples/simple-ts`)

**YAML Format:**
- `simple_ts_low.yaml` - Low level of detail
- `simple_ts_medium.yaml` - Medium level of detail
- `simple_ts_high.yaml` - High level of detail

**Ad-Hoc Format (Granularity Levels):**
- `simple_ts_high_adhoc_default.yaml` - Level 0: Default format (ID|Name|Type)
- `simple_ts_high_adhoc_signatures.yaml` - Level 1: Inline signatures (ID|Signature|Type)
- `simple_ts_high_adhoc_logic.yaml` - Level 2: Inline logic (ID|Signature|Type|logic:steps) - **Gold Standard**

### NestJS API Project (`examples/nestjs-api-ts`)

**YAML Format:**
- `nestjs_low.yaml` - Low level of detail
- `nestjs_medium.yaml` - Medium level of detail
- `nestjs_high.yaml` - High level of detail

**Ad-Hoc Format (Granularity Levels):**
- `nestjs_high_adhoc_default.yaml` - Level 0: Default format (ID|Name|Type)
- `nestjs_high_adhoc_signatures.yaml` - Level 1: Inline signatures (ID|Signature|Type)
- `nestjs_high_adhoc_logic.yaml` - Level 2: Inline logic (ID|Signature|Type|logic:steps) - **Gold Standard**

## Usage in Tests

Property-based tests use these baselines via the `baseline_helpers` module:

```rust
use crate::baseline_helpers::{load_baseline, compare_yaml_outputs};

// Load baseline
let baseline = load_baseline("simple_ts_medium")?;

// Generate current output with same config
let current = run_scip_conversion(&scip_path, default_config)?;

// Compare
assert!(compare_yaml_outputs(&baseline, &current)?);
```

## When to Regenerate

Regenerate baselines when:
- ✅ Fixing bugs in core graph generation logic
- ✅ Updating SCIP parsing to handle new language features
- ✅ Making intentional changes to default output format
- ❌ Adding new optimization features (these should NOT change default output)

## Validation

After regenerating, verify:
1. File sizes are reasonable (not empty, not excessively large)
2. YAML structure is valid
3. Token counts are logged during generation
4. All expected test cases are present

## Notes

- These files represent the "previous version" behavior
- Default configuration means: no compaction, YAML format, no framework filtering, no file filtering
- Baselines are committed to version control for CI/CD testing
- If baselines don't exist, tests will skip with a helpful message
