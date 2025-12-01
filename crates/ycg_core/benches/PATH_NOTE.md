# Important Note About Paths

## Working Directory

When Cargo runs benchmarks, the working directory is set to the **crate directory** (`crates/ycg_core`), not the workspace root.

This means:
- ✅ Correct path: `../../examples/simple-ts/index.scip`
- ❌ Wrong path: `examples/simple-ts/index.scip` (would look in `crates/ycg_core/examples/`)
- ❌ Wrong path: `../../../examples/simple-ts/index.scip` (goes too far up)

## Path Structure

```
YCG/                          <- Workspace root
├── examples/
│   ├── simple-ts/
│   │   └── index.scip
│   └── nestjs-api-ts/
│       └── index.scip
└── crates/
    └── ycg_core/             <- Benchmark runs from HERE
        ├── benches/
        │   └── granularity_benchmarks.rs
        └── Cargo.toml
```

From `crates/ycg_core`, to reach `examples/simple-ts/index.scip`:
- Go up 2 levels: `../../`
- Then into examples: `examples/simple-ts/index.scip`
- Full path: `../../examples/simple-ts/index.scip`

## Verification

To verify the paths are correct:

```bash
# From workspace root
cd crates/ycg_core
ls ../../examples/simple-ts/index.scip
ls ../../examples/nestjs-api-ts/index.scip
```

Both should exist and show the SCIP files.

## If Benchmarks Skip

If you see "⚠ Skipping benchmark: SCIP file not found", check:

1. **SCIP files exist:**
   ```bash
   ls -lh examples/*/index.scip
   ```

2. **Paths in benchmark code are correct:**
   ```rust
   let scip_path = PathBuf::from("../../examples/simple-ts/index.scip");
   ```

3. **Generate SCIP files if missing:**
   ```bash
   cd examples/simple-ts
   npx @sourcegraph/scip-typescript index .
   
   cd ../nestjs-api-ts
   npx @sourcegraph/scip-typescript index .
   ```

## Auto-Formatting Warning

⚠️ **Warning:** Some IDE auto-formatters may incorrectly "fix" these paths. If benchmarks suddenly stop working after formatting, check that the paths still have `../../examples/` and not some other variant.
