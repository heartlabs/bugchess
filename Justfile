# Bugchess task runner
# Install: cargo install just

# List all recipes
default:
    @just --list

# Run all workspace tests
test:
    cargo test --workspace

# Run clippy lints
lint:
    cargo clippy --workspace -- -D warnings

# Format code (requires nightly)
fmt:
    cargo +nightly fmt

# Check formatting (CI use)
fmt-check:
    cargo +nightly fmt --check

# Build WASM release
build-wasm:
    bash build.sh

# Build native debug
build:
    cargo build

# Serve locally (builds first)
serve:
    bash dev.sh

# Generate docs
doc:
    cargo doc --workspace --no-deps --open
