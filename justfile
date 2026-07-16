set dotenv-load := true

# No args auto runs CLI list of options
default: choose

# Interactively choose which project task to execute right now
choose:
    @just --choose

# Run the Leptos web server via Trunk and open it in your browser
run:
    env -u NO_COLOR trunk serve --port 3000 --open

# Cargo fmt and clippy
lint:
    cargo fmt --all -- --check
    cargo clippy --all-targets --locked -- -D warnings

# Cargo fmt and test
test:
    cargo fmt --all -- --check
    cargo test

# Run the Leptos formatter
fmt:
    cargo leptosfmt

# Build a fully minified production-ready release bundle in /dist
build:
    env -u NO_COLOR trunk build --release

# Audit third-party dependencies for known vulnerabilities
audit:
    cargo audit

# Deeply scrub all cargo build objects and Trunk generation files
clean:
    cargo clean
    rm -rf dist

# Check formatting, run lints, and execute tests locally
check-all:
    cargo fmt --all -- --check
    cargo clippy --all-targets --locked -- -D warnings
    cargo test
