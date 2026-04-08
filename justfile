# Format Rust code
fmt:
    cargo fmt

# Run clippy with pedantic lints (matches CI)
lint:
    cargo clippy -- -D warnings

# Run Rust tests
test:
    cargo test

# Full CI check: format + lint + test
check: fmt lint test

# Build Python bindings
dev:
    maturin develop --release

# Run Python tests (builds first)
pytest: dev
    pytest tests/ -v
