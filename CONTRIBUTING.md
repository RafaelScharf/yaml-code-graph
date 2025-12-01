# Contributing to YCG

First off, thanks for taking the time to contribute! 🎉

The YCG (YAML Code Graph) project is an open-source tool designed to bridge the gap between source code and LLMs. We value every contribution, whether it's:
- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Becoming a maintainer

## 🛠️ Project Structure

YCG is structured as a **Rust Workspace** (Monorepo):

* **`crates/ycg_core`**: The library containing the core logic (SCIP parsing, Tree-sitter enrichment, graph topology).
* **`crates/ycg_cli`**: The command-line interface that consumes the core library.
* **`proto/`**: Contains the SCIP Protocol Buffer definitions.

## ⚡ Prerequisites

To build and run YCG, you need:

1.  **Rust**: Make sure you have the latest stable version of Rust installed.
    ```bash
    rustup update stable
    ```
2.  **SCIP Indexer**: To generate test inputs (indexes), you need a SCIP indexer (e.g., for TypeScript).
    ```bash
    npm install -g @sourcegraph/scip-typescript
    ```

## 🚀 Development Workflow

### 1. Fork and Clone
Fork the repository on GitHub, then clone your fork locally:

```bash
git clone [https://github.com/YOUR-USERNAME/ycg.git](https://github.com/YOUR-USERNAME/ycg.git)
cd ycg
```

### 2. Build the Project

Since this is a workspace, you can build all crates at once:

```bash
cargo build

```

To run the CLI directly during development:


```bash
# Run the CLI against a sample SCIP file
cargo run --bin ycg_cli -- -i ./path/to/index.scip
```

### 3. Running Tests

Please ensure all tests pass before submitting a PR.

```bash
# Run unit tests for all crates
cargo test --workspace
```

### 4. Code Style & Linting

We follow standard Rust community guidelines.

-   **Formatting**: We use `rustfmt`.
    
    ```bash
    cargo fmt --all
    ```
    
-   **Linting**: We use `clippy` to catch common mistakes.
    
    ```bash
    cargo clippy --workspace -- -D warnings
    ```
    

## 🐛 Reporting Bugs

Bugs are tracked as GitHub issues. When filing an issue, please include:

1.  The version of `ycg` used.
    
2.  The language of the source code being analyzed (e.g., TypeScript, Rust).
    
3.  A minimal reproducible example (code snippet + expected output vs. actual output).
    

## 📝 Pull Request Process

1.  Create a new branch for your feature or fix: `git checkout -b feature/amazing-feature`.
    
2.  Commit your changes using descriptive messages (we recommend [Conventional Commits](https://www.conventionalcommits.org/)).
    
    -   Example: `feat(core): add support for python guard clauses`
        
    -   Example: `fix(cli): resolve panic on missing input file`
        
3.  Ensure your code is formatted (`cargo fmt`) and linted (`cargo clippy`).
    
4.  Push to your fork and submit a Pull Request.

## ⚖️ License

By contributing, you agree that your contributions will be licensed under its [Apache License 2.0](https://www.google.com/search?q=./LICENSE).