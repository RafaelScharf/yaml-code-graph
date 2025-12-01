.PHONY: build install uninstall test clean help check fmt example all dev run-example bench

# Default target
help:
	@echo "YCG - YAML Code Graph Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  make build       - Build release binary"
	@echo "  make install     - Install ycg globally (requires sudo)"
	@echo "  make uninstall   - Remove ycg from system"
	@echo "  make test        - Run all tests"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make check       - Run clippy and format checks"
	@echo "  make fmt         - Format code with rustfmt"
	@echo "  make example     - Run example on simple-ts (requires install)"
	@echo "  make run-example - Run example without installing"
	@echo "  make bench       - Run benchmarks"
	@echo "  make all         - Build and install"
	@echo "  make dev         - Run full development workflow"
	@echo ""

# Build release binary
build:
	@echo "🔨 Building YCG CLI in release mode..."
	@cargo build --release
	@echo "✅ Binary available at: ./target/release/ycg_cli"

# Install globally
install: build
	@echo "� I nstalling YCG globally..."
	@sudo ./install.sh

# Uninstall
uninstall:
	@echo "🗑️  Removing YCG from system..."
	@sudo rm -f /usr/local/bin/ycg
	@echo "✅ Uninstalled successfully"

# Run tests
test:
	@echo "� Runniing tests..."
	@cargo test

# Run tests with output
test-verbose:
	@echo "🧪 Running tests (verbose)..."
	@cargo test -- --nocapture

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -f examples/*/test_output.yaml
	@rm -f examples/*/output.yaml
	@echo "✅ Clean complete"

# Run clippy and format checks
check:
	@echo "🔍 Running clippy..."
	@cargo clippy -- -D warnings
	@echo "📝 Checking formatting..."
	@cargo fmt -- --check
	@echo "✅ All checks passed"

# Format code
fmt:
	@echo "✨ Formatting code..."
	@cargo fmt
	@echo "✅ Code formatted"

# Run example (requires installation)
example:
	@echo "🎯 Running example on simple-ts..."
	@ycg -i examples/simple-ts/index.scip -o examples/simple-ts/output.yaml --compact
	@echo "✅ Output saved to examples/simple-ts/output.yaml"

# Run example without installing
run-example:
	@echo "🎯 Running example on simple-ts (without install)..."
	@cargo run --release --bin ycg_cli -- -i examples/simple-ts/index.scip -o examples/simple-ts/output.yaml --compact
	@echo "✅ Output saved to examples/simple-ts/output.yaml"

# Run benchmarks
bench:
	@echo "⚡ Running benchmarks..."
	@cargo bench

# Build and install in one command
all: build install

# Development workflow
dev: fmt check test build
	@echo "✅ Development checks passed!"

# Quick development cycle (no tests)
quick: fmt build
	@echo "✅ Quick build complete!"
