prefix := env_var_or_default("PREFIX", "/usr/local")
destdir := env_var_or_default("DESTDIR", "")
bindir := destdir + prefix + "/bin"
user-bindir := env_var("HOME") + "/.Bin"
bins := "chronic combine errno ifdata ifne isutf8 lckdo mispipe parallel pee sponge ts vidir vipe zrun"

test:
    cargo test

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test

build-release:
    cargo build --release

install: build-release
    install -d "{{bindir}}"
    for bin in {{bins}}; do install -m 0755 "target/release/$bin" "{{bindir}}/$bin"; done
    printf 'Installed oddutils to %s\n' "{{bindir}}"

install-user: build-release
    install -d "{{user-bindir}}"
    for bin in {{bins}}; do install -m 0755 "target/release/$bin" "{{user-bindir}}/$bin"; done
    printf 'Installed oddutils to %s\n' "{{user-bindir}}"
