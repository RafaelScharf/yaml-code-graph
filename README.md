# YCG (YAML Code Graph)

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Status](https://img.shields.io/badge/status-alpha-yellow.svg)]()

> **Semantic Code Transcoder for LLMs**

**YCG** (YAML Code Graph) is a high-performance Rust tool that transforms source code into semantic knowledge graphs optimized for Large Language Models. By combining SCIP (Stack Graph Code Indexing Protocol) for global symbol resolution with Tree-sitter for local enrichment, YCG generates "Pseudo-RDF" YAML representations that maximize semantic density while minimizing token consumption.

## 🚀 Quick Start

New to YCG? Check out the [Quick Start Guide](QUICKSTART.md) for a 5-minute tutorial!

### One-Command Workflow

```bash
# Automatic language detection and indexing
ycg index

# Generate optimized YAML graph
ycg generate -i index.scip -o graph.yaml --compact
```

That's it! YCG automatically detects your project language (Rust or TypeScript) and generates the semantic graph.

## Why YCG?

Traditional code context (raw files) is inefficient and noisy for LLMs. YCG addresses this by:

1. **📉 Token Compression:** Achieves >1.5x compression ratio through YAML anchors/aliases and semantic deduplication
2. **🧠 Logic Lifting:** Extracts guard clauses and preconditions from imperative code into declarative predicates
3. **🔗 Semantic Topology:** Explicitly maps references, scopes, and dependencies using deterministic identifiers
4. **⚡ Blazing Fast:** Processes 10,000+ files in <60s using Rust + SCIP + Tree-sitter
5. **🎯 LOD Control:** Adjustable Level of Detail (Low/Medium/High) to fit token budgets

## 🏗️ Architecture

YCG implements a hybrid two-pass pipeline:

### Pass A: Symbol Registration
- Deserializes SCIP Protobuf indexes
- Generates deterministic xxHash-64 identifiers for all symbols
- Builds anchor registry for YAML reference resolution

### Pass B: Graph Construction
- **Definitions:** Extracts symbol nodes with Tree-sitter enrichment (signatures, docs, logic)
- **References:** Resolves caller/callee relationships using scope analysis
- **Optimization:** Applies LOD filtering and optional adjacency list compression

### Key Components
- **SCIP Indexer:** Global symbol resolution without reimplementing static analysis
- **Tree-sitter Parser:** Local enrichment for TypeScript, JavaScript, Rust
- **Logic Lifter:** Detects guard clauses (`if-throw`, `if-return`) and transforms to preconditions
- **YAML Emitter:** Serializes with anchors (&) and aliases (*) for minimal token usage

## 📦 Installation

### Quick Install

```bash
git clone https://github.com/yourusername/ycg.git
cd ycg
cargo build --release
sudo ./install.sh
```

This installs the `ycg` command globally. See [INSTALL.md](INSTALL.md) for detailed installation options.

### Prerequisites
- **Rust 1.75+** with Cargo
- **SCIP Indexer** for your target language:
  ```bash
  npm install -g @sourcegraph/scip-typescript  # For TypeScript/JavaScript
  # Or: scip-python, scip-java, etc.
  ```

### Alternative: Cargo Install

```bash
cargo install --path crates/ycg_cli
```

## 🛠️ Usage

### Quick Start (Automatic Indexing)

YCG can automatically detect your project language and generate the SCIP index:

```bash
cd my-project

# Automatically detect language and generate index + graph
ycg index                           # Creates index.scip
ycg generate -i index.scip -o graph.yaml --compact
```

## 🎯 Token Optimization

YCG provides powerful optimization strategies to reduce token consumption by 40-70% while maintaining semantic accuracy. All optimizations are **opt-in** and can be combined for maximum efficiency.

### Configuration File

Create a `ycg.config.json` in your project root to define optimization settings:

```json
{
  "output": {
    "format": "adhoc",
    "compact": true,
    "ignoreFrameworkNoise": true
  },
  "ignore": {
    "useGitignore": true,
    "customPatterns": [
      "**/node_modules/**",
      "**/dist/**"
    ]
  },
  "include": [
    "src/**/*.ts"
  ]
}
```

**Configuration Precedence:** CLI flags override config file settings.

### Optimization Strategies

#### 1. Graph Compaction (`--compact`)

Filters out low-significance symbols (local variables, anonymous blocks) while preserving architectural information.

```bash
# Enable graph compaction
ycg generate -i index.scip -o graph.yaml --compact

# Expected reduction: ~50% in graph section size
```

**Before (default):**
```yaml
graph:
  - from: validateUser_a3f2
    to: local_11_6d84
    type: defines
  - from: local_11_6d84
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

**Token Reduction:** ~50% in graph section

#### 2. Framework Noise Reduction (`--ignore-framework-noise`)

Removes boilerplate patterns from framework-heavy codebases (NestJS, TypeORM).

```bash
# Enable framework noise reduction
ycg generate -i index.scip -o graph.yaml --ignore-framework-noise
```

**What gets filtered:**
- Dependency injection constructors (only assignment statements)
- Decorator metadata (`@ApiProperty`, `@IsString`, etc.)
- DTO boilerplate while preserving property names and types

**Before:**
```yaml
- id: UserDto_a1b2
  sig: |
    class UserDto {
      @ApiProperty()
      @IsString()
      name: string;
      
      @ApiProperty()
      @IsEmail()
      email: string;
    }
```

**After:**
```yaml
- id: UserDto_a1b2
  sig: |
    class UserDto {
      name: string;
      email: string;
    }
```

**Token Reduction:** ~30% in framework-heavy projects

#### 3. Ad-Hoc Format (`--output-format adhoc`)

Position-based format that eliminates verbose YAML key-value pairs.

```bash
# Use ad-hoc format
ycg generate -i index.scip -o graph.yaml --output-format adhoc
```

**YAML Format:**
```yaml
_defs:
  - id: validateUser_a3f2
    n: validateUser
    t: function
```

**Ad-Hoc Format:**
```yaml
_defs:
  - "validateUser_a3f2|validateUser|function"
```

**Token Reduction:** ~20-30% in definitions section

#### 4. Selective File Processing

Control which files are processed using glob patterns.

```bash
# Include only specific files
ycg generate -i index.scip -o graph.yaml --include "src/**/*.ts"

# Exclude test files
ycg generate -i index.scip -o graph.yaml --exclude "**/*.test.ts"

# Disable gitignore (process all files)
ycg generate -i index.scip -o graph.yaml --no-gitignore
```

**Pattern Precedence:** Include first, then exclude. If a file matches both, it's excluded.

### Combined Optimization Example

Maximize token reduction by combining all strategies:

```bash
ycg generate -i index.scip -o graph.yaml \
  --compact \
  --ignore-framework-noise \
  --output-format adhoc \
  --include "src/**/*.ts" \
  --exclude "**/*.test.ts"
```

**Expected Results:**
- **Token Reduction:** 60-70% compared to default output
- **Processing Time:** <5% overhead
- **Semantic Accuracy:** 100% preserved for significant symbols

### Token Reduction Comparison

| Strategy | Token Reduction | Use Case |
|----------|----------------|----------|
| Default (no flags) | 0% (baseline) | Full detail needed |
| `--compact` | ~50% | Focus on architecture |
| `--ignore-framework-noise` | ~30% | NestJS, TypeORM projects |
| `--output-format adhoc` | ~25% | Minimize syntax overhead |
| **All combined** | **60-70%** | Maximum efficiency |

### Example Config Files

See the `examples/` directory for ready-to-use configurations:

- `ycg.config.minimal.json` - Minimal configuration
- `ycg.config.full.json` - All optimizations enabled
- `ycg.config.typescript.json` - TypeScript project optimized
- `ycg.config.rust.json` - Rust project optimized

For detailed guidance on choosing optimizations, see [OPTIMIZATION_GUIDE.md](OPTIMIZATION_GUIDE.md).

## 🎚️ Ad-Hoc Granularity Levels

YCG's ad-hoc format supports three granularity levels that control the amount of detail included in each symbol definition. This allows you to optimize token usage based on your specific analysis needs.

### Granularity Levels Overview

| Level | Format | Use Case | Token Savings |
|-------|--------|----------|---------------|
| **Level 0: Default** | `ID\|Name\|Type` | Maximum token efficiency, structural information only | Baseline |
| **Level 1: Signatures** | `ID\|Signature\|Type` | API contracts and data flow analysis | +15-20% tokens |
| **Level 2: Logic** | `ID\|Signature\|Type\|logic:steps` | Security analysis, business logic review | +30-40% tokens |

### CLI Flags

```bash
# Level 0: Default (structural only)
ycg generate -i index.scip -o graph.yaml --output-format adhoc

# Level 1: Include inline signatures
ycg generate -i index.scip -o graph.yaml --output-format adhoc --adhoc-inline-signatures

# Level 2: Include inline signatures + logic
ycg generate -i index.scip -o graph.yaml --output-format adhoc --adhoc-inline-logic
```

**Note:** The `--adhoc-inline-logic` flag automatically enables signatures, so you don't need both flags.

### Configuration File

You can also set the granularity level in your `ycg.config.json`:

```json
{
  "output": {
    "format": "adhoc",
    "adhocGranularity": "logic"
  }
}
```

**Valid values:** `"default"`, `"signatures"`, `"logic"`

**Precedence:** CLI flags override config file settings.

### Example Outputs

#### Level 0: Default (Structural Only)

```yaml
_defs:
  - UserDto_698e|UserDto|class
  - StoreService_4844|StoreService|class
  - StoreService_checkStock_7fed|checkStock|method
  - StoreService_purchase_99a1|purchase|method
```

**Use when:** You only need to understand the codebase structure and relationships.

#### Level 1: Inline Signatures

```yaml
_defs:
  - UserDto_698e|UserDto|class
  - StoreService_4844|StoreService|class
  - StoreService_checkStock_7fed|checkStock(itemId:str):bool|method
  - StoreService_purchase_99a1|purchase(user:User,itemId:str):Promise<Order>|method
```

**Use when:** You need to understand API contracts, parameter types, and return values.

**Type Abbreviations:**
- `string` → `str`
- `number` → `num`
- `boolean` → `bool`
- Custom types preserved (e.g., `User`, `Order`)

#### Level 2: Inline Logic (Gold Standard)

```yaml
_defs:
  - UserDto_698e|UserDto|class
  - StoreService_4844|StoreService|class
  - StoreService_checkStock_7fed|checkStock(itemId:str):bool|method|logic:return(item.qty > 0)
  - StoreService_purchase_99a1|purchase(user,itemId)|method|logic:check(stock>0);check(user.balance>=price);action(deduct_balance);action(save_order)
  - AuthService_validate_dd39|validate(user)|method|logic:check(user.isActive && (user.isAdmin || user.isSuper))
  - RolesGuard_canActivate_c9aa|canActivate(ctx)|method|logic:get(user_roles);match(required_roles)?allow:deny
```

**Use when:** You need to understand business logic, security checks, and control flow.

**Logic Keywords:**
- `check(condition)` - Conditional checks, guard clauses
- `action(operation)` - Side effects, state mutations
- `return(expression)` - Return values
- `match(pattern)?true:false` - Pattern matching, ternary operators
- `get(source)` - Data retrieval operations

Logic steps are chained with semicolons (`;`) to represent execution sequence.

### Example Config Files

See the `examples/` directory for ready-to-use configurations:

- `ycg.config.granularity-default.json` - Level 0 (structural only)
- `ycg.config.granularity-signatures.json` - Level 1 (with signatures)
- `ycg.config.granularity-logic.json` - Level 2 (with logic)

### Token Impact

**Before/After Comparison** (NestJS API example):

| Level | Definitions Tokens | Total Tokens | vs. Default |
|-------|-------------------|--------------|-------------|
| Level 0 (Default) | 2,450 | 8,120 | Baseline |
| Level 1 (Signatures) | 2,890 | 8,560 | +5.4% |
| Level 2 (Logic) | 3,380 | 9,050 | +11.5% |

**Recommendation:** Start with Level 0 for initial exploration, then use Level 1 for API analysis, and Level 2 only when you need to understand business logic or security-critical code.

For detailed information about logic keywords and extraction rules, see [LOGIC_KEYWORDS.md](LOGIC_KEYWORDS.md) and [GRANULARITY_GUIDE.md](GRANULARITY_GUIDE.md).

### Step 1: Generate SCIP Index

**Option A: Automatic (Recommended)**

```bash
cd my-project
ycg index                           # Auto-detects Rust or TypeScript
ycg index -d ./src -o custom.scip   # Custom directory and output
```

**Option B: Manual**

Navigate to your project and create the index manually:

```bash
cd my-project

# For TypeScript/JavaScript projects
npm install  # Install dependencies first
scip-typescript index .

# For Rust projects
rust-analyzer scip-export --output index.scip
```

### Step 2: Generate YAML Graph

Convert the SCIP index to optimized YAML:

```bash
# Standard mode (flat list of edges)
ycg generate -i index.scip -o graph.yaml

# Compact mode (adjacency list - recommended)
ycg generate -i index.scip -o graph.yaml --compact

# High detail mode (includes locals and externals)
ycg generate -i index.scip -o graph.yaml --lod 2

# Specify project root explicitly
ycg generate -i index.scip -o graph.yaml --root /path/to/project
```

### CLI Commands

#### `ycg index`

Automatically detect project language and generate SCIP index.

| Flag | Description | Default |
|------|-------------|---------|
| `-d, --directory` | Project directory to index | `.` (current) |
| `-o, --output` | Output path for SCIP index | `index.scip` |

**Supported Languages:**
- **Rust**: Detects `Cargo.toml`, uses `rust-analyzer`
- **TypeScript/JavaScript**: Detects `package.json`, uses `scip-typescript`

#### `ycg generate`

Generate YAML graph from existing SCIP index.

| Flag | Description | Default |
|------|-------------|---------|
| `-i, --input` | Path to SCIP index file | Required |
| `-o, --output` | Path to output YAML file | stdout |
| `-r, --root` | Project root directory | Parent of input file |
| `-l, --lod` | Level of Detail (0=Low, 1=Medium, 2=High) | 1 |
| `-c, --compact` | Enable adjacency list optimization | false |
| `--ignore-framework-noise` | Remove framework boilerplate patterns | false |
| `--output-format <FORMAT>` | Output format: `yaml` or `adhoc` | yaml |
| `--include <PATTERN>` | Include files matching glob pattern (repeatable) | All files |
| `--exclude <PATTERN>` | Exclude files matching glob pattern (repeatable) | None |
| `--no-gitignore` | Disable automatic gitignore processing | false |

### Level of Detail (LOD)

- **Low (0):** Classes and functions only, excludes variables/interfaces
- **Medium (1):** Includes public methods, filters local variables and parameters
- **High (2):** Full detail including private methods, locals, and external references

## 📊 Output Format

### Input (TypeScript)
```typescript
function validateUser(name: string) {
  if (name.length === 0) throw new Error("Empty name");
  return { valid: true, name };
}
```

### Output (YCG YAML)
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
    calls:
      - Error_b8c1
```

### Compact Mode (Adjacency List)
When using `--compact`, edges are grouped by source node and type:
```yaml
graph:
  source_node_id:
    calls: [target1, target2]
    references: [target3]
```

## 🔧 Project Structure

```
ycg/
├── crates/
│   ├── ycg_core/          # Core library
│   │   ├── src/
│   │   │   ├── lib.rs     # Main conversion logic
│   │   │   ├── model.rs   # Data structures (SymbolNode, ReferenceEdge)
│   │   │   └── enricher.rs # Tree-sitter integration
│   │   ├── build.rs       # Protobuf code generation
│   │   └── Cargo.toml
│   └── ycg_cli/           # CLI application
│       ├── src/main.rs
│       └── Cargo.toml
├── examples/
│   ├── nestjs-api-ts/     # Example NestJS project
│   └── simple-ts/         # Minimal TypeScript example
├── proto/
│   └── scip.proto         # SCIP protocol definition
└── Cargo.toml             # Workspace configuration
```

## 🧪 Examples

The `examples/` directory contains sample projects:

### NestJS API Example (Automatic)
```bash
cd examples/nestjs-api-ts
npm install
ycg index                                    # Auto-detects TypeScript
ycg generate -i index.scip -o context_map.yaml --compact
```

### NestJS API Example (Manual)
```bash
cd examples/nestjs-api-ts
npm install
scip-typescript index .
../../target/release/ycg_cli generate -i index.scip -o context_map.yaml --compact
```

### Simple TypeScript Example
```bash
cd examples/simple-ts
ycg index                                    # Auto-detects TypeScript
ycg generate -i index.scip -o output.yaml
```

### Rust Project Example (Dogfooding)
```bash
cd /path/to/ycg
ycg index                                    # Auto-detects Rust
ycg generate -i index.scip -o ycg_graph.yaml --compact --lod 2
```

## 📈 Performance Metrics

YCG automatically reports compression metrics:

```
--- Métrica de Densidade ---
Input Total Tokens (Código Bruto): 45230
Output Total Tokens (Grafo YAML): 28145
Taxa de Compressão: 1.61x
--------------------------
```

## 🛣️ Roadmap

- [x] SCIP Protobuf deserialization
- [x] Tree-sitter enrichment (TypeScript, JavaScript, Rust)
- [x] Logic lifting (guard clauses)
- [x] Deterministic xxHash-64 identifiers
- [x] LOD filtering
- [x] Adjacency list optimization
- [x] Token density metrics
- [ ] Streaming mode for >100MB indexes
- [ ] Python grammar support
- [ ] Property-based testing suite
- [ ] Configuration file support

## 🔧 Troubleshooting

Having issues? Check the [Troubleshooting Guide](TROUBLESHOOTING.md) for common problems and solutions:

- Language detection issues
- Missing indexer tools (rust-analyzer, scip-typescript)
- SCIP export failures
- Performance optimization tips

## 🤝 Contributing

Contributions are welcome! This project follows the SPARC methodology for development. See `.kiro/specs/ycg-core/requirements.md` for detailed requirements.

### Development Setup
```bash
cargo build
cargo test
cargo clippy -- -D warnings
```

## 📝 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [SCIP Protocol](https://github.com/sourcegraph/scip) by Sourcegraph
- [Tree-sitter](https://tree-sitter.github.io/) parsing library
- [xxHash](https://github.com/Cyan4973/xxHash) non-cryptographic hash algorithm

