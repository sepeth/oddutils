install-dir := env_var("HOME") + "/.Bin"
bins := "chronic combine errno ifdata ifne isutf8 lckdo mispipe parallel pee sponge ts vidir vipe zrun"

test:
    cargo test

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test

build-release:
    cargo build --release

install-local: build-release
    mkdir -p "{{install-dir}}"
    for bin in {{bins}}; do cp "target/release/$bin" "{{install-dir}}/"; done
    printf 'Installed oddutils to %s\n' "{{install-dir}}"
