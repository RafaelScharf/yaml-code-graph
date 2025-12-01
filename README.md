# YCG (YAML Code Graph)

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Status](https://img.shields.io/badge/status-alpha-yellow.svg)]()

> **Semantic Code Transcoder for LLMs**

**YCG** (YAML Code Graph) is a high-performance Rust tool that transforms source code into semantic knowledge graphs optimized for Large Language Models. By combining SCIP (Stack Graph Code Indexing Protocol) for global symbol resolution with Tree-sitter for local enrichment, YCG generates "Pseudo-RDF" YAML representations that maximize semantic density while minimizing token consumption.

## 🚀 Why YCG?

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

### Step 1: Generate SCIP Index

Navigate to your project and create the index:

```bash
cd my-project
npm install  # Install dependencies first
scip-typescript index .
# Creates 'index.scip' in current directory
```

### Step 2: Run YCG

Convert the SCIP index to optimized YAML:

```bash
# Standard mode (flat list of edges)
ycg -i index.scip -o graph.yaml

# Compact mode (adjacency list - recommended)
ycg -i index.scip -o graph.yaml --compact

# High detail mode (includes locals and externals)
ycg -i index.scip -o graph.yaml --lod 2

# Specify project root explicitly
ycg -i index.scip -o graph.yaml --root /path/to/project
```

### CLI Options

| Flag | Description | Default |
|------|-------------|---------|
| `-i, --input` | Path to SCIP index file | Required |
| `-o, --output` | Path to output YAML file | stdout |
| `-r, --root` | Project root directory | Parent of input file |
| `-l, --lod` | Level of Detail (0=Low, 1=Medium, 2=High) | 1 |
| `-c, --compact` | Enable adjacency list optimization | false |

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

### NestJS API Example
```bash
cd examples/nestjs-api-ts
npm install
scip-typescript index .
../../target/release/ycg_cli -i index.scip -o context_map.yaml --compact
```

### Simple TypeScript Example
```bash
cd examples/simple-ts
scip-typescript index .
../../target/release/ycg_cli -i index.scip -o output.yaml
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

