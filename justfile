test:
    cargo test

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
