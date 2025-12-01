# YCG CLI Reference

Complete command-line interface reference for YCG (YAML Code Graph).

**Version:** 0.2.0  
**Last Updated:** December 2024

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
  - [ycg index](#ycg-index)
  - [ycg generate](#ycg-generate)
- [Configuration File](#configuration-file)
- [CLI Flags Reference](#cli-flags-reference)
- [Output Formats](#output-formats)
- [Optimization Strategies](#optimization-strategies)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

---

## Installation

### Quick Install

```bash
git clone https://github.com/yourusername/ycg.git
cd ycg
./install.sh
```

The install script will:
1. Check if Rust/Cargo is installed
2. Build the latest version with `cargo build --release`
3. Install the `ycg` binary to `/usr/local/bin`

### Manual Installation

```bash
cargo build --release
sudo cp target/release/ycg_cli /usr/local/bin/ycg
```

### Verify Installation

```bash
ycg --version
# Output: ycg_cli 0.2.0
```

---

## Quick Start

```bash
# 1. Navigate to your project
cd my-project

# 2. Generate SCIP index (auto-detects language)
ycg index

# 3. Generate optimized YAML graph
ycg generate -i index.scip -o graph.yaml --compact
```

---

## Commands

### ycg index

Automatically detect project language and generate SCIP index.

**Syntax:**
```bash
ycg index [OPTIONS]
```

**Options:**

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--directory <PATH>` | `-d` | Project directory to index | `.` (current) |
| `--output <PATH>` | `-o` | Output path for SCIP index | `index.scip` |

**Supported Languages:**

- **Rust:** Detects `Cargo.toml`, uses `rust-analyzer`
- **TypeScript/JavaScript:** Detects `package.json` or `tsconfig.json`, uses `scip-typescript`

**Examples:**

```bash
# Index current directory
ycg index

# Index specific directory
ycg index -d ./src -o src-index.scip

# Index and specify output location
ycg index --directory /path/to/project --output /tmp/index.scip
```

**Requirements:**

- **Rust projects:** `rust-analyzer` must be installed
  ```bash
  rustup component add rust-analyzer
  ```

- **TypeScript/JavaScript projects:** `scip-typescript` must be installed
  ```bash
  npm install -g @sourcegraph/scip-typescript
  ```

**Error Messages:**

- `"Could not detect project language"` - No `Cargo.toml`, `package.json`, or `tsconfig.json` found
- `"rust-analyzer not found"` - Install with `rustup component add rust-analyzer`
- `"scip-typescript not found"` - Install with `npm install -g @sourcegraph/scip-typescript`

---

### ycg generate

Generate YAML graph from existing SCIP index.

**Syntax:**
```bash
ycg generate -i <INPUT> [OPTIONS]
```

**Required Arguments:**

| Flag | Short | Description |
|------|-------|-------------|
| `--input <PATH>` | `-i` | Path to SCIP index file |

**Optional Arguments:**

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--output <PATH>` | `-o` | Path to output YAML file | stdout |
| `--root <PATH>` | `-r` | Project root directory | Parent of input file |
| `--lod <LEVEL>` | `-l` | Level of Detail (0=Low, 1=Medium, 2=High) | `1` |
| `--compact` | `-c` | Enable adjacency list optimization | `false` |
| `--ignore-framework-noise` | | Remove framework boilerplate | `false` |
| `--output-format <FORMAT>` | | Output format: `yaml` or `adhoc` | `yaml` |
| `--include <PATTERN>` | | Include files matching glob (repeatable) | All files |
| `--exclude <PATTERN>` | | Exclude files matching glob (repeatable) | None |
| `--no-gitignore` | | Disable gitignore processing | `false` |
| `--adhoc-inline-signatures` | | Enable Level 1 granularity (requires `adhoc`) | `false` |
| `--adhoc-inline-logic` | | Enable Level 2 granularity (requires `adhoc`) | `false` |

**Examples:**

```bash
# Basic usage
ycg generate -i index.scip -o graph.yaml

# With optimization
ycg generate -i index.scip -o graph.yaml --compact

# Maximum optimization
ycg generate -i index.scip -o graph.yaml \
  --compact \
  --ignore-framework-noise \
  --output-format adhoc

# With file filtering
ycg generate -i index.scip -o graph.yaml \
  --include "src/**/*.ts" \
  --exclude "**/*.test.ts"

# Ad-hoc format with signatures
ycg generate -i index.scip -o graph.yaml \
  --output-format adhoc \
  --adhoc-inline-signatures

# Ad-hoc format with logic (includes signatures)
ycg generate -i index.scip -o graph.yaml \
  --output-format adhoc \
  --adhoc-inline-logic
```

---

## Configuration File

YCG supports configuration files to avoid repeating CLI flags. Create a `ycg.config.json` in your project root.

**Precedence:** CLI flags override config file settings.

### Configuration Schema

```json
{
  "output": {
    "format": "yaml" | "adhoc",
    "compact": true | false,
    "ignoreFrameworkNoise": true | false,
    "adhocGranularity": "default" | "signatures" | "logic"
  },
  "ignore": {
    "useGitignore": true | false,
    "customPatterns": ["pattern1", "pattern2"]
  },
  "include": ["pattern1", "pattern2"]
}
```

### Configuration Options

#### output.format

**Type:** `string`  
**Values:** `"yaml"` | `"adhoc"`  
**Default:** `"yaml"`

Output format for the graph.

- `yaml`: Standard YAML with key-value pairs
- `adhoc`: Compact pipe-separated format (`ID|Name|Type`)

**Example:**
```json
{
  "output": {
    "format": "adhoc"
  }
}
```

#### output.compact

**Type:** `boolean`  
**Default:** `false`

Enable graph compaction (adjacency list format).

**Example:**
```json
{
  "output": {
    "compact": true
  }
}
```

#### output.ignoreFrameworkNoise

**Type:** `boolean`  
**Default:** `false`

Remove framework boilerplate patterns (DI constructors, decorators).

**Example:**
```json
{
  "output": {
    "ignoreFrameworkNoise": true
  }
}
```

#### output.adhocGranularity

**Type:** `string`  
**Values:** `"default"` | `"signatures"` | `"logic"`  
**Default:** `"default"`

Granularity level for ad-hoc format. Requires `format: "adhoc"`.

- `default`: Level 0 - `ID|Name|Type`
- `signatures`: Level 1 - `ID|Signature(args):Return|Type`
- `logic`: Level 2 - `ID|Signature|Type|logic:steps`

**Example:**
```json
{
  "output": {
    "format": "adhoc",
    "adhocGranularity": "signatures"
  }
}
```

#### ignore.useGitignore

**Type:** `boolean`  
**Default:** `true`

Automatically exclude files listed in `.gitignore`.

**Example:**
```json
{
  "ignore": {
    "useGitignore": true
  }
}
```

#### ignore.customPatterns

**Type:** `array of strings`  
**Default:** `[]`

Additional glob patterns to exclude.

**Example:**
```json
{
  "ignore": {
    "customPatterns": [
      "**/node_modules/**",
      "**/dist/**",
      "**/*.test.ts"
    ]
  }
}
```

#### include

**Type:** `array of strings`  
**Default:** `[]` (all files)

Glob patterns for files to include. If specified, only matching files are processed.

**Example:**
```json
{
  "include": [
    "src/**/*.ts",
    "src/**/*.tsx"
  ]
}
```

### Example Configurations

#### Minimal Configuration

```json
{
  "output": {
    "format": "yaml"
  }
}
```

#### TypeScript Project

```json
{
  "output": {
    "format": "yaml",
    "compact": true,
    "ignoreFrameworkNoise": true
  },
  "ignore": {
    "useGitignore": true,
    "customPatterns": [
      "**/node_modules/**",
      "**/dist/**",
      "**/*.test.ts",
      "**/*.spec.ts"
    ]
  },
  "include": [
    "src/**/*.ts",
    "src/**/*.tsx"
  ]
}
```

#### Rust Project

```json
{
  "output": {
    "format": "yaml",
    "compact": true
  },
  "ignore": {
    "useGitignore": true,
    "customPatterns": [
      "**/target/**"
    ]
  },
  "include": [
    "src/**/*.rs",
    "crates/**/src/**/*.rs"
  ]
}
```

#### Maximum Optimization

```json
{
  "output": {
    "format": "adhoc",
    "compact": true,
    "ignoreFrameworkNoise": true,
    "adhocGranularity": "signatures"
  },
  "ignore": {
    "useGitignore": true,
    "customPatterns": [
      "**/node_modules/**",
      "**/dist/**",
      "**/*.test.ts",
      "**/*.spec.ts",
      "**/__tests__/**"
    ]
  },
  "include": [
    "src/**/*.ts"
  ]
}
```

---

## CLI Flags Reference

### Level of Detail (LOD)

Controls how much information is included in the graph.

| Level | Flag | Description | Includes |
|-------|------|-------------|----------|
| **0 (Low)** | `--lod 0` | Classes and functions only | Exported symbols, public methods |
| **1 (Medium)** | `--lod 1` | Default level | + Public methods, filters locals |
| **2 (High)** | `--lod 2` | Full detail | + Private methods, locals, externals |

**Example:**
```bash
# Low detail (architecture overview)
ycg generate -i index.scip -o graph.yaml --lod 0

# High detail (complete analysis)
ycg generate -i index.scip -o graph.yaml --lod 2
```

### Compact Mode

**Flag:** `--compact` or `-c`

Transforms flat edge list into adjacency list format.

**Before (default):**
```yaml
graph:
  - from: validateUser_a3f2
    to: Error_b8c1
    type: calls
  - from: validateUser_a3f2
    to: String_b2c3
    type: references
```

**After (compact):**
```yaml
graph:
  validateUser_a3f2:
    calls: [Error_b8c1]
    references: [String_b2c3]
```

**Token Reduction:** ~30-40%

### Framework Noise Reduction

**Flag:** `--ignore-framework-noise`

Removes framework-specific boilerplate:
- Dependency injection constructors (only `this.x = x` assignments)
- Decorator metadata (`@ApiProperty`, `@IsString`, etc.)
- DTO boilerplate

**Token Reduction:** ~15-30% in framework-heavy projects

### Output Format

**Flag:** `--output-format <FORMAT>`  
**Values:** `yaml` | `adhoc`

#### YAML Format (Default)

Standard YAML with key-value pairs.

```yaml
_defs:
  - id: validateUser_a3f2
    n: validateUser
    t: function
    sig: 'function validateUser(name: string)'
```

#### Ad-Hoc Format

Compact pipe-separated format.

```yaml
_defs:
  - "validateUser_a3f2|validateUser|function"
```

**Token Reduction:** ~15-20%

### Ad-Hoc Granularity Levels

Control the level of detail in ad-hoc format.

#### Level 0: Default (Structural Only)

**No flag required** (default when using `--output-format adhoc`)

**Format:** `ID|Name|Type`

```yaml
_defs:
  - "UserService_a1b2|UserService|class"
  - "UserService_findById_c3d4|findById|method"
```

**Use when:** Architecture review, structure understanding

#### Level 1: Inline Signatures

**Flag:** `--adhoc-inline-signatures`

**Format:** `ID|Signature(args):Return|Type`

```yaml
_defs:
  - "UserService_a1b2|UserService|class"
  - "UserService_findById_c3d4|findById(id:str):Promise<User>|method"
```

**Use when:** API analysis, type checking, integration planning

**Token Overhead:** +15-20%

#### Level 2: Inline Logic

**Flag:** `--adhoc-inline-logic`

**Format:** `ID|Signature|Type|logic:steps`

```yaml
_defs:
  - "UserService_findById_c3d4|findById(id:str):Promise<User>|method|logic:check(id);get(user_repo);return(user)"
```

**Use when:** Security audits, business logic review, compliance verification

**Token Overhead:** +30-40%

**Note:** `--adhoc-inline-logic` implicitly enables signatures.

### File Filtering

#### Include Patterns

**Flag:** `--include <PATTERN>` (repeatable)

Only process files matching the glob pattern.

```bash
# Include only TypeScript files in src
ycg generate -i index.scip -o graph.yaml --include "src/**/*.ts"

# Multiple patterns
ycg generate -i index.scip -o graph.yaml \
  --include "src/**/*.ts" \
  --include "lib/**/*.ts"
```

#### Exclude Patterns

**Flag:** `--exclude <PATTERN>` (repeatable)

Skip files matching the glob pattern.

```bash
# Exclude test files
ycg generate -i index.scip -o graph.yaml --exclude "**/*.test.ts"

# Multiple patterns
ycg generate -i index.scip -o graph.yaml \
  --exclude "**/*.test.ts" \
  --exclude "**/*.spec.ts"
```

#### Gitignore Integration

**Flag:** `--no-gitignore`

By default, YCG respects `.gitignore` files. Use this flag to disable.

```bash
# Process all files (ignore .gitignore)
ycg generate -i index.scip -o graph.yaml --no-gitignore
```

**Pattern Precedence:** Include → Exclude → Gitignore

If a file matches both include and exclude, it's excluded.

---

## Output Formats

### YAML Format (Default)

Standard YAML with explicit key-value pairs.

**Structure:**
```yaml
_meta:
  name: ycg-v1.3
  version: 1.3.0

_defs:
  - id: validateUser_a3f2
    n: validateUser
    t: function
    sig: 'function validateUser(name: string)'
    logic:
      pre:
        - 'must avoid: name.length === 0'

graph:
  validateUser_a3f2:
    calls: [Error_b8c1]
    references: [String_b2c3]
```

**Advantages:**
- Human-readable
- Self-documenting
- Easy to parse with standard YAML libraries

**Disadvantages:**
- More verbose
- Higher token count

### Ad-Hoc Format

Compact pipe-separated positional format.

**Structure:**
```yaml
_meta:
  name: ycg-v1.3
  version: 1.3.0

_defs:
  - "validateUser_a3f2|validateUser|function"
  - "User_b8c1|User|class"

graph:
  validateUser_a3f2:
    calls: [Error_b8c1]
```

**Advantages:**
- Minimal token consumption
- Faster parsing
- Ideal for LLM contexts

**Disadvantages:**
- Less human-readable
- Requires understanding of positional format

---

## Optimization Strategies

### Strategy Combinations

| Combination | Token Reduction | Use Case |
|-------------|----------------|----------|
| Default (no flags) | 0% (baseline) | Full detail needed |
| `--compact` | 30-40% | Architecture focus |
| `--compact --output-format adhoc` | 40-50% | General optimization |
| `--compact --ignore-framework-noise` | 45-55% | Framework projects |
| **All optimizations** | **60-70%** | Maximum efficiency |

### Recommended Workflows

#### Quick Architecture Review

```bash
ycg index
ycg generate -i index.scip -o graph.yaml --compact --lod 0
```

**Result:** High-level structure, minimal tokens

#### API Documentation

```bash
ycg index
ycg generate -i index.scip -o graph.yaml \
  --output-format adhoc \
  --adhoc-inline-signatures \
  --compact
```

**Result:** API contracts with type information

#### Security Audit

```bash
ycg index
ycg generate -i index.scip -o graph.yaml \
  --output-format adhoc \
  --adhoc-inline-logic \
  --include "src/auth/**/*.ts" \
  --include "src/payment/**/*.ts"
```

**Result:** Business logic and security checks visible

#### Maximum Token Efficiency

```bash
ycg index
ycg generate -i index.scip -o graph.yaml \
  --compact \
  --ignore-framework-noise \
  --output-format adhoc \
  --include "src/**/*.ts" \
  --exclude "**/*.test.ts"
```

**Result:** 60-70% token reduction

---

## Examples

### Example 1: Basic Usage

```bash
cd my-project
ycg index
ycg generate -i index.scip -o graph.yaml
```

### Example 2: TypeScript Project with Optimization

```bash
cd nestjs-api
ycg index
ycg generate -i index.scip -o graph.yaml \
  --compact \
  --ignore-framework-noise \
  --include "src/**/*.ts" \
  --exclude "**/*.test.ts"
```

### Example 3: Rust Project

```bash
cd rust-project
ycg index
ycg generate -i index.scip -o graph.yaml --compact --lod 2
```

### Example 4: Using Configuration File

Create `ycg.config.json`:
```json
{
  "output": {
    "format": "adhoc",
    "compact": true,
    "ignoreFrameworkNoise": true
  },
  "include": ["src/**/*.ts"]
}
```

Then run:
```bash
ycg index
ycg generate -i index.scip -o graph.yaml
```

### Example 5: Selective Processing

```bash
# Process only authentication module
ycg generate -i index.scip -o auth-graph.yaml \
  --include "src/auth/**/*.ts" \
  --output-format adhoc \
  --adhoc-inline-logic

# Process only API layer
ycg generate -i index.scip -o api-graph.yaml \
  --include "src/api/**/*.ts" \
  --compact
```

### Example 6: CI/CD Integration

```bash
#!/bin/bash
# generate-graph.sh

# Generate SCIP index
ycg index -o index.scip

# Generate optimized graph
ycg generate -i index.scip -o docs/code-graph.yaml \
  --compact \
  --ignore-framework-noise \
  --output-format adhoc

# Commit to repository
git add docs/code-graph.yaml
git commit -m "Update code graph"
```

---

## Troubleshooting

### Command Not Found

**Problem:** `ycg: command not found`

**Solution:**
```bash
# Reinstall
cd ycg
./install.sh

# Or add to PATH manually
export PATH="/usr/local/bin:$PATH"
```

### Language Detection Failed

**Problem:** `"Could not detect project language"`

**Solution:** Ensure you have one of:
- `Cargo.toml` (Rust)
- `package.json` (TypeScript/JavaScript)
- `tsconfig.json` (TypeScript)

### Indexer Not Found

**Problem:** `"rust-analyzer not found"` or `"scip-typescript not found"`

**Solution:**
```bash
# For Rust
rustup component add rust-analyzer

# For TypeScript
npm install -g @sourcegraph/scip-typescript
```

### Empty Output

**Problem:** Generated graph is empty or has very few symbols

**Solution:**
1. Check LOD level: `--lod 2` for maximum detail
2. Verify SCIP index was generated correctly: `ls -lh index.scip`
3. Check file filtering: remove `--include`/`--exclude` flags

### Granularity Flags Not Working

**Problem:** `--adhoc-inline-signatures` or `--adhoc-inline-logic` has no effect

**Solution:** These flags require `--output-format adhoc`:
```bash
ycg generate -i index.scip -o graph.yaml \
  --output-format adhoc \
  --adhoc-inline-signatures
```

### Configuration File Ignored

**Problem:** `ycg.config.json` settings not applied

**Solution:**
1. Ensure file is in project root (same directory as SCIP index)
2. Verify JSON syntax: `cat ycg.config.json | jq`
3. CLI flags override config - remove conflicting flags

### Performance Issues

**Problem:** Processing is too slow

**Solution:**
1. Use file filtering to reduce scope:
   ```bash
   ycg generate -i index.scip -o graph.yaml --include "src/**/*.ts"
   ```
2. Disable framework noise reduction (most expensive):
   ```bash
   ycg generate -i index.scip -o graph.yaml --compact
   ```
3. Process in parallel (split by directory)

---

## Further Reading

- [README.md](README.md) - Project overview and quick start
- [QUICKSTART.md](QUICKSTART.md) - 5-minute tutorial
- [GRANULARITY_GUIDE.md](GRANULARITY_GUIDE.md) - Detailed granularity levels guide
- [OPTIMIZATION_GUIDE.md](OPTIMIZATION_GUIDE.md) - Token optimization strategies
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues and solutions
- [CHANGELOG.md](CHANGELOG.md) - Version history

---

## Support

- **Issues:** [GitHub Issues](https://github.com/yourusername/ycg/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourusername/ycg/discussions)
- **Documentation:** Full documentation in repository

---

**YCG CLI Reference v0.2.0**  
**Copyright © 2024 - Licensed under Apache 2.0**
