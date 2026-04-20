# Format Rust + Python code
fmt:
    cargo fmt
    ruff format python tests

# Run all linters (matches CI)
lint:
    cargo clippy --all-targets -- -D warnings
    ruff check python tests

# Auto-fix Python lint issues (sort imports, remove unused, etc.)
lint-fix:
    ruff check --fix python tests
    ruff format python tests

# Run Rust tests
test:
    cargo test

# Full CI check: format + lint + test
check: fmt lint test

# Build Python bindings
dev:
    maturin develop --release

# Run Python unit tests (builds first; benchmarks are skipped by default)
pytest: dev
    pytest tests/ -v --ignore=tests/benchmarks

# Run Rust benchmarks (Criterion)
bench-rust:
    cargo bench --features bench

# Run Python benchmarks (pytest-benchmark)
bench-py: dev
    pytest tests/benchmarks/ --benchmark-enable --benchmark-only

# Run all benchmarks
bench: bench-rust bench-py
