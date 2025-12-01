# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- **CI**: Upgraded GitHub Actions workflow (`ci.yml`) to include Protobuf compiler installation, dependency caching (`Swatinem/rust-cache`), and strict linting with `clippy` and `rustfmt`.
- **CLI**: Updated `ycg_cli` edition to "2024" in `Cargo.toml`.

## [0.1.0] - 2025-12-01
### Added
- **Core**: Initial project initialization with Rust Workspace structure.
- **Core**: Implemented `ycg_core` crate with SCIP parsing, Tree-sitter enrichment, and Logic Lifting engine.
- **CLI**: Implemented `ycg_cli` crate for command-line interaction.
- **Proto**: Added `scip.proto` definitions for protobuf compilation.
- **Examples**: Added `simple-ts` example for basic TypeScript testing.
- **Examples**: Added `nestjs-api-ts` example to validate complex dependency graphs and authentication guards.