# Troubleshooting Guide

## Common Issues

### `ycg index` Command

#### Error: "Could not detect project language"

**Cause**: YCG looks for `Cargo.toml` (Rust) or `package.json` (TypeScript/JavaScript) to detect the language.

**Solution**:
1. Ensure you're in the correct project directory
2. Verify the manifest file exists:
   ```bash
   ls Cargo.toml    # For Rust
   ls package.json  # For TypeScript/JavaScript
   ```
3. If using a non-standard structure, use manual indexing instead

#### Error: "rust-analyzer not found in PATH"

**Cause**: The Rust SCIP indexer is not installed.

**Solution**:
```bash
# Install rust-analyzer via rustup
rustup component add rust-analyzer

# Verify installation
rust-analyzer --version
```

**Alternative**: Install from source following the [official guide](https://rust-analyzer.github.io/manual.html#installation)

#### Error: "scip-typescript not found"

**Cause**: The TypeScript SCIP indexer is not installed.

**Solution**:
```bash
# Install globally via npm
npm install -g @sourcegraph/scip-typescript

# Verify installation
scip-typescript --version
```

**Alternative**: Use `npx` for local execution (no global install needed):
```bash
npx @sourcegraph/scip-typescript index .
```

#### Error: "rust-analyzer scip-export failed"

**Cause**: The SCIP export feature may be experimental or unavailable in your rust-analyzer version.

**Solution**:
1. Update rust-analyzer to the latest version:
   ```bash
   rustup update
   rustup component add rust-analyzer --force
   ```
2. Check if SCIP export is supported:
   ```bash
   rust-analyzer --help | grep scip
   ```
3. If unavailable, use manual indexing with an alternative tool

#### Error: "scip-typescript failed with exit code"

**Cause**: TypeScript compilation errors or missing dependencies.

**Solution**:
1. Install project dependencies first:
   ```bash
   npm install
   ```
2. Verify TypeScript compiles without errors:
   ```bash
   npx tsc --noEmit
   ```
3. Check for `tsconfig.json` in the project root
4. Review scip-typescript output for specific errors

### `ycg generate` Command

#### Error: "Failed to deserialize SCIP index"

**Cause**: Corrupted or incompatible SCIP index file.

**Solution**:
1. Regenerate the SCIP index:
   ```bash
   ycg index -o index.scip
   ```
2. Verify the index file is not empty:
   ```bash
   ls -lh index.scip
   ```
3. Ensure the SCIP indexer completed successfully

#### Error: "No symbols found in SCIP index"

**Cause**: The project may be empty or the indexer didn't process any files.

**Solution**:
1. Verify source files exist in the project
2. Check if the indexer ran in the correct directory
3. For TypeScript, ensure `tsconfig.json` includes the source files
4. For Rust, ensure `Cargo.toml` is properly configured

#### Low Semantic Density (<1.5x compression)

**Cause**: Project may have minimal logic or many external dependencies.

**Solution**:
1. Use higher LOD (Level of Detail):
   ```bash
   ycg generate -i index.scip -o graph.yaml --lod 2
   ```
2. Enable compact mode for better compression:
   ```bash
   ycg generate -i index.scip -o graph.yaml --compact
   ```
3. This is expected for projects with mostly type definitions

## Platform-Specific Issues

### macOS

#### Error: "command not found: rust-analyzer"

**Cause**: PATH not updated after rustup installation.

**Solution**:
```bash
source ~/.cargo/env
rust-analyzer --version
```

### Windows

#### Error: "rust-analyzer is not recognized"

**Cause**: Cargo bin directory not in PATH.

**Solution**:
1. Add `%USERPROFILE%\.cargo\bin` to PATH
2. Restart terminal
3. Verify: `rust-analyzer --version`

### Linux

#### Error: "Permission denied: rust-analyzer"

**Cause**: Binary not executable.

**Solution**:
```bash
chmod +x ~/.cargo/bin/rust-analyzer
```

## Performance Issues

### Slow SCIP Index Generation

**For Large Projects (>10,000 files)**:
1. Use incremental indexing if supported
2. Exclude unnecessary directories (e.g., `node_modules`, `target`)
3. Consider indexing specific subdirectories only

### High Memory Usage

**Solution**:
1. Use LOD 0 (low detail) for initial exploration:
   ```bash
   ycg generate -i index.scip -o graph.yaml --lod 0
   ```
2. Process subdirectories separately
3. Increase system swap space if needed

## Getting Help

If you encounter an issue not covered here:

1. Check the [GitHub Issues](https://github.com/yourusername/ycg/issues)
2. Run with verbose output (if available)
3. Include the following in bug reports:
   - YCG version: `ycg --version`
   - Operating system
   - Project language and size
   - Complete error message
   - Steps to reproduce

## Debugging Tips

### Enable Verbose Logging

```bash
# Set RUST_LOG environment variable
export RUST_LOG=debug
ycg index
ycg generate -i index.scip -o graph.yaml
```

### Validate SCIP Index

```bash
# Check if index is valid Protobuf
file index.scip
# Should output: "index.scip: Protocol Buffer data"
```

### Test with Minimal Example

```bash
# Create minimal test project
mkdir test-project
cd test-project

# For Rust
cargo init
ycg index

# For TypeScript
npm init -y
echo 'function test() {}' > index.ts
echo '{"compilerOptions": {}}' > tsconfig.json
ycg index
```
