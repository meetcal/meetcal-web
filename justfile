# Run the Leptos web server via Trunk and open it in your browser
run-web:
    trunk serve --port 3000 --open

# Check formatting, run lints, and execute tests locally
check-all:
    cargo fmt --all -- --check
    cargo clippy --all-targets --locked -- -D warnings
    cargo test
