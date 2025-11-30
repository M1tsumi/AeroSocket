# Makefile for AeroSocket development
# Provides convenient commands for common development tasks

.PHONY: help build test clean doc fmt clippy audit bench install-deps check-all

# Default target
help:
	@echo "AeroSocket Development Commands:"
	@echo ""
	@echo "  build          - Build all crates"
	@echo "  test           - Run all tests"
	@echo "  test-coverage  - Run tests with coverage"
	@echo "  clean          - Clean build artifacts"
	@echo "  doc            - Generate documentation"
	@echo "  doc-open       - Generate and open documentation"
	@echo "  fmt            - Format code"
	@echo "  fmt-check      - Check code formatting"
	@echo "  clippy         - Run clippy lints"
	@echo "  audit          - Run security audit"
	@echo "  bench          - Run benchmarks"
	@echo "  install-deps   - Install development dependencies"
	@echo "  check-all      - Run all checks (fmt, clippy, test)"
	@echo ""

# Build all crates
build:
	cargo build --all-features --workspace

# Build release
build-release:
	cargo build --release --all-features --workspace

# Run all tests
test:
	cargo test --all-features --workspace

# Run tests with coverage
test-coverage:
	cargo llvm-cov --all-features --workspace --html --output-dir target/coverage

# Clean build artifacts
clean:
	cargo clean

# Generate documentation
doc:
	cargo doc --all-features --workspace --no-deps

# Generate and open documentation
doc-open:
	cargo doc --all-features --workspace --no-deps --open

# Format code
fmt:
	cargo fmt --all

# Check code formatting
fmt-check:
	cargo fmt --all -- --check

# Run clippy lints
clippy:
	cargo clippy --all-features --workspace -- -D warnings

# Run security audit
audit:
	cargo audit
	cargo deny check

# Run benchmarks
bench:
	cargo bench --all-features --workspace

# Install development dependencies
install-deps:
	cargo install cargo-audit cargo-deny cargo-llvm-cov cargo-watch cargo-expand

# Run all checks
check-all: fmt-check clippy test

# Watch for changes and run tests
watch:
	cargo watch -x 'test --all-features --workspace'

# Watch for changes and run clippy
watch-clippy:
	cargo watch -x 'clippy --all-features --workspace -- -D warnings'

# Publish dry run
publish-dry-run:
	cargo publish --dry-run --all-features --workspace

# Update dependencies
update:
	cargo update --workspace

# Check for outdated dependencies
outdated:
	cargo outdated --workspace

# Generate flamegraph (requires cargo-flamegraph)
flamegraph:
	cargo flamegraph --bin aerosocket-server

# Run memory profiling (requires valgrind)
mem-prof:
	RUSTFLAGS='-g' cargo test --all-features --workspace -- --nocapture
	valgrind --tool=massif target/debug/deps/aerosocket-*

# Run performance profiling (requires perf)
perf:
	RUSTFLAGS='-g' cargo test --all-features --workspace -- --nocapture
	perf record --call-graph=dwarf target/debug/deps/aerosocket-*
	perf report

# Generate dependency graph
deps:
	cargo deps --workspace --dot | dot -Tpng > deps.png

# Check for unused dependencies
unused-deps:
	cargo machete

# Run integration tests
integration-test:
	cargo test --test '*' --all-features --workspace

# Run documentation tests
doc-test:
	cargo test --doc --all-features --workspace

# Run example tests
example-test:
	cargo test --examples --all-features --workspace

# Run all test categories
test-all: test doc-test integration-test example-test

# Setup development environment
setup:
	rustup component add rustfmt clippy llvm-tools-preview
	cargo install cargo-audit cargo-deny cargo-llvm-cov cargo-watch cargo-expand cargo-machete

# Generate changelog
changelog:
	cargo install cargo-changelog
	cargo changelog

# Create release
release: check-all test-all
	@echo "Ready for release! Run 'cargo publish' for each crate in order."

# Development server (for examples)
dev-server:
	cargo run --example echo_server --features "server transport-tcp"
