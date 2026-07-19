prefix := env_var_or_default("PREFIX", "/usr/local")
destdir := env_var_or_default("DESTDIR", "")
command-prefix := env_var_or_default("COMMAND_PREFIX", "")
bindir := destdir + prefix + "/bin"
mandir := destdir + prefix + "/share/man"
man1dir := mandir + "/man1"
user-prefix := env_var_or_default("USER_PREFIX", env_var("HOME") + "/.local")
user-bindir := user-prefix + "/bin"
user-man1dir := user-prefix + "/share/man/man1"
bins := "chronic combine errno ifdata ifne isutf8 lckdo mispipe parallel pee sponge ts vidir vipe zrun"

test:
    cargo test

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test

check-moreutils: build-isutf8
    tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/oddutils-moreutils-isutf8.XXXXXX")"; \
    trap 'rm -rf "$tmpdir"' EXIT; \
    cp ../moreutils/is_utf8/test.sh "$tmpdir/test.sh"; \
    ln -s "$PWD/target/debug/isutf8" "$tmpdir/isutf8"; \
    cd "$tmpdir"; \
    bash ./test.sh

build-isutf8:
    cargo build -p oddutils-isutf8

build-release:
    cargo build --release

man:
    mkdir -p target/man/man1
    for src in docs/man/*.1.scd; do name=$(basename "$src" .scd); scdoc < "$src" > "target/man/man1/$name"; done

install: build-release install-man
    install -d "{{bindir}}"
    for bin in {{bins}}; do install -m 0755 "target/release/$bin" "{{bindir}}/{{command-prefix}}$bin"; done
    printf 'Installed oddutils to %s\n' "{{bindir}}"

install-user: build-release install-user-man
    install -d "{{user-bindir}}"
    for bin in {{bins}}; do install -m 0755 "target/release/$bin" "{{user-bindir}}/{{command-prefix}}$bin"; done
    printf 'Installed oddutils to %s\n' "{{user-bindir}}"

install-man: man
    install -d "{{man1dir}}"
    for src in docs/man/*.1.scd; do name=$(basename "$src" .1.scd); sed -e "1s/^$name(1)/{{command-prefix}}$name(1)/" -e "s/^$name -/{{command-prefix}}$name -/" -e "s/\\*$name\\*/\\*{{command-prefix}}$name\\*/g" "$src" | scdoc > "{{man1dir}}/{{command-prefix}}$name.1"; chmod 0644 "{{man1dir}}/{{command-prefix}}$name.1"; done
    printf 'Installed oddutils manpages to %s\n' "{{man1dir}}"

install-user-man: man
    install -d "{{user-man1dir}}"
    for src in docs/man/*.1.scd; do name=$(basename "$src" .1.scd); sed -e "1s/^$name(1)/{{command-prefix}}$name(1)/" -e "s/^$name -/{{command-prefix}}$name -/" -e "s/\\*$name\\*/\\*{{command-prefix}}$name\\*/g" "$src" | scdoc > "{{user-man1dir}}/{{command-prefix}}$name.1"; chmod 0644 "{{user-man1dir}}/{{command-prefix}}$name.1"; done
    printf 'Installed oddutils manpages to %s\n' "{{user-man1dir}}"
