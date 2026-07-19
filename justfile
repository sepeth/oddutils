prefix := env_var_or_default("PREFIX", "/usr/local")
destdir := env_var_or_default("DESTDIR", "")
bindir := destdir + prefix + "/bin"
mandir := destdir + prefix + "/share/man"
man1dir := mandir + "/man1"
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

man:
    mkdir -p target/man/man1
    for src in docs/man/*.1.scd; do name=$(basename "$src" .scd); scdoc < "$src" > "target/man/man1/$name"; done

install: build-release install-man
    install -d "{{bindir}}"
    for bin in {{bins}}; do install -m 0755 "target/release/$bin" "{{bindir}}/$bin"; done
    printf 'Installed oddutils to %s\n' "{{bindir}}"

install-user: build-release
    install -d "{{user-bindir}}"
    for bin in {{bins}}; do install -m 0755 "target/release/$bin" "{{user-bindir}}/$bin"; done
    printf 'Installed oddutils to %s\n' "{{user-bindir}}"

install-man: man
    install -d "{{man1dir}}"
    for page in target/man/man1/*.1; do install -m 0644 "$page" "{{man1dir}}/$(basename "$page")"; done
    printf 'Installed oddutils manpages to %s\n' "{{man1dir}}"
