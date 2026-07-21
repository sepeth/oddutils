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
lima-freebsd-instance := env_var_or_default("LIMA_FREEBSD_INSTANCE", "oddutils-freebsd")
lima-openbsd-instance := env_var_or_default("LIMA_OPENBSD_INSTANCE", "oddutils-openbsd")
lima-openbsd-workdir := env_var_or_default("LIMA_OPENBSD_WORKDIR", "/tmp/oddutils")
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
    limactl shell --workdir /workspace/oddutils "{{lima-instance}}" env TERM=xterm-256color bash

lima-freebsd-create:
    limactl create --name="{{lima-freebsd-instance}}" --param "REPO=$PWD" .lima/freebsd.yaml

lima-freebsd-stop:
    limactl stop "{{lima-freebsd-instance}}"

lima-freebsd-delete:
    limactl delete --force "{{lima-freebsd-instance}}"

lima-freebsd-recreate: lima-freebsd-delete lima-freebsd-create

lima-freebsd-start:
    limactl start "{{lima-freebsd-instance}}"

lima-freebsd-test:
    limactl shell --workdir /workspace/oddutils "{{lima-freebsd-instance}}" sh -lc 'just test'

lima-freebsd-shell:
    limactl shell --workdir /workspace/oddutils "{{lima-freebsd-instance}}" env TERM=xterm-256color sh

lima-openbsd-create:
    limactl create --arch=x86_64 --name="{{lima-openbsd-instance}}" .lima/openbsd.yaml

lima-openbsd-stop:
    limactl stop "{{lima-openbsd-instance}}"

lima-openbsd-delete:
    limactl delete --force "{{lima-openbsd-instance}}"

lima-openbsd-recreate: lima-openbsd-delete lima-openbsd-create

lima-openbsd-start:
    #!/usr/bin/env sh
    set -eu
    limactl start "{{lima-openbsd-instance}}" &
    pid=$!
    until limactl shell "{{lima-openbsd-instance}}" sh -lc 'test -s /var/lib/cloud/data/instance-id' >/dev/null 2>&1; do
        sleep 2
    done
    limactl shell "{{lima-openbsd-instance}}" sh -lc 'sudo mkdir -p /run && cat /var/lib/cloud/data/instance-id | sudo tee /run/lima-boot-done >/dev/null'
    wait "$pid"

lima-openbsd-disk:
    limactl shell "{{lima-openbsd-instance}}" env TERM=xterm-256color sh -lc 'if mount | grep -q " on /usr/local "; then exit 0; fi; printf "e 3\nA6\nn\n64\n*\nf 3\nw\nq\n" | sudo fdisk -e sd0; printf "b\n\n*\nw\nq\n" | sudo disklabel -v -f /etc/fstab -E sd0; sudo /root/bin/create_partitions.sh'

lima-openbsd-setup: lima-openbsd-disk
    limactl shell "{{lima-openbsd-instance}}" env TERM=xterm-256color sh -lc 'command -v cargo >/dev/null && command -v scdoc >/dev/null && command -v just >/dev/null || sudo pkg_add rust scdoc just'

lima-openbsd-sync:
    limactl shell "{{lima-openbsd-instance}}" sh -lc 'rm -rf "{{lima-openbsd-workdir}}" && mkdir -p "{{lima-openbsd-workdir}}"'
    limactl copy --backend=scp -r Cargo.toml Cargo.lock justfile crates docs "{{lima-openbsd-instance}}:{{lima-openbsd-workdir}}/"

lima-openbsd-test: lima-openbsd-setup lima-openbsd-sync
    limactl shell --workdir "{{lima-openbsd-workdir}}" "{{lima-openbsd-instance}}" env TERM=xterm-256color sh -lc 'just test'

lima-openbsd-shell:
    limactl shell "{{lima-openbsd-instance}}" env TERM=xterm-256color sh -lc 'mkdir -p "{{lima-openbsd-workdir}}" && cd "{{lima-openbsd-workdir}}" && exec sh'

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
