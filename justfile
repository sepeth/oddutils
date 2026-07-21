prefix := env_var_or_default("PREFIX", "/usr/local")
destdir := env_var_or_default("DESTDIR", "")
command-prefix := env_var_or_default("COMMAND_PREFIX", "")
bindir := destdir + prefix + "/bin"
mandir := destdir + prefix + "/share/man"
man1dir := mandir + "/man1"
user-prefix := env_var_or_default("USER_PREFIX", env_var("HOME") + "/.local")
user-bindir := user-prefix + "/bin"
user-man1dir := user-prefix + "/share/man/man1"
container-runtime := env_var_or_default("CONTAINER_RUNTIME", "docker")
lima-instance := env_var_or_default("LIMA_INSTANCE", "oddutils-debian")
system-install := env_var_or_default("INSTALL", "install")
bins := "chronic combine errno ifdata ifne isutf8 lckdo mispipe parallel pee sponge ts vidir vipe zrun"

test:
    cargo test

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test

build-release:
    cargo build --release

container-build:
    {{container-runtime}} build --target build -t oddutils-build .

container-test: container-build
    {{container-runtime}} run --rm oddutils-build just test

container-image:
    {{container-runtime}} build -t oddutils .

lima-create:
    limactl create --name="{{lima-instance}}" --param "REPO=$PWD" .lima/debian.yaml

lima-stop:
    limactl stop "{{lima-instance}}"

lima-delete:
    limactl delete --force "{{lima-instance}}"

lima-recreate: lima-delete lima-create

lima-start:
    limactl start "{{lima-instance}}"

lima-test:
    limactl shell --workdir /workspace/oddutils "{{lima-instance}}" bash -lc 'just test'

lima-shell:
    limactl shell --workdir /workspace/oddutils "{{lima-instance}}"

man:
    mkdir -p target/man/man1
    for src in docs/man/*.1.scd; do name=$(basename "$src" .scd); scdoc < "$src" > "target/man/man1/$name"; done

install: build-release install-man
    {{system-install}} -d "{{bindir}}"
    for bin in {{bins}}; do {{system-install}} -m 0755 "target/release/$bin" "{{bindir}}/{{command-prefix}}$bin"; done
    printf 'Installed oddutils to %s\n' "{{bindir}}"

install-system:
    INSTALL="sudo install" just install

install-user: build-release install-user-man
    install -d "{{user-bindir}}"
    for bin in {{bins}}; do install -m 0755 "target/release/$bin" "{{user-bindir}}/{{command-prefix}}$bin"; done
    printf 'Installed oddutils to %s\n' "{{user-bindir}}"

install-man: man
    {{system-install}} -d "{{man1dir}}"
    for src in docs/man/*.1.scd; do name=$(basename "$src" .1.scd); out="target/man/man1/{{command-prefix}}$name.1"; sed -e "1s/^$name(1)/{{command-prefix}}$name(1)/" -e "s/^$name -/{{command-prefix}}$name -/" -e "s/\\*$name\\*/\\*{{command-prefix}}$name\\*/g" "$src" | scdoc > "$out"; {{system-install}} -m 0644 "$out" "{{man1dir}}/{{command-prefix}}$name.1"; done
    printf 'Installed oddutils manpages to %s\n' "{{man1dir}}"

install-user-man: man
    install -d "{{user-man1dir}}"
    for src in docs/man/*.1.scd; do name=$(basename "$src" .1.scd); sed -e "1s/^$name(1)/{{command-prefix}}$name(1)/" -e "s/^$name -/{{command-prefix}}$name -/" -e "s/\\*$name\\*/\\*{{command-prefix}}$name\\*/g" "$src" | scdoc > "{{user-man1dir}}/{{command-prefix}}$name.1"; chmod 0644 "{{user-man1dir}}/{{command-prefix}}$name.1"; done
    printf 'Installed oddutils manpages to %s\n' "{{user-man1dir}}"
