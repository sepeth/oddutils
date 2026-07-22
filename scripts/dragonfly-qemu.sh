#!/usr/bin/env sh
set -eu

image_url="${DRAGONFLY_IMAGE_URL:-https://object-storage.public.mtl1.vexxhost.net/swift/v1/1dbafeefbd4f4c80864414a441e72dd2/bsd-cloud-image.org/images/dragonflybsd/6.4.0/2023-04-23/hammer2/dragonflybsd-6.4.0-hammer2-2023-04-23.qcow2}"
cache_dir="${DRAGONFLY_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/oddutils/dragonfly-qemu}"
state_dir="${DRAGONFLY_STATE_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/oddutils/dragonfly-qemu}"
ssh_port="${DRAGONFLY_SSH_PORT:-58123}"
ssh_user="${DRAGONFLY_SSH_USER:-${USER:-oddutils}}"
ssh_key="${DRAGONFLY_SSH_KEY:-$HOME/.lima/_config/user}"
ssh_pub_key="${DRAGONFLY_SSH_PUB_KEY:-$ssh_key.pub}"
workdir="${DRAGONFLY_WORKDIR:-/tmp/oddutils}"
repo_dir="${DRAGONFLY_REPO_DIR:-$(pwd)}"
cpus="${DRAGONFLY_CPUS:-4}"
memory="${DRAGONFLY_MEMORY:-4096}"
disk_size="${DRAGONFLY_DISK_SIZE:-10G}"
mac_address="${DRAGONFLY_MAC_ADDRESS:-52:55:55:e9:fe:e1}"
qemu_system="${QEMU_SYSTEM:-qemu-system-x86_64}"
qemu_img="${QEMU_IMG:-qemu-img}"

image="$cache_dir/dragonflybsd-6.4.0-hammer2.qcow2"
disk="$state_dir/disk.qcow2"
pid_file="$state_dir/qemu.pid"
serial_log="$state_dir/serial.log"
qemu_log="$state_dir/qemu.log"
cidata_dir="$state_dir/cidata"
cidata_iso="$state_dir/cidata.iso"

usage() {
    cat <<'EOF'
usage: scripts/dragonfly-qemu.sh COMMAND

Commands:
  create    Download the DragonFly image and create an overlay disk plus cidata ISO
  start     Start the QEMU guest
  stop      Stop the QEMU guest
  delete    Remove the overlay disk, cidata, and logs
  setup     Install rust, scdoc, and just in the guest
  sync      Copy this checkout into the guest
  test      Start, set up, sync, and run just test in the guest
  shell     Open a shell in the guest checkout
  logs      Print the serial log
EOF
}

need() {
    if ! command -v "$1" >/dev/null 2>&1; then
        printf 'dragonfly-qemu: missing required command: %s\n' "$1" >&2
        exit 1
    fi
}

ssh_opts() {
    printf '%s\n' \
        -o StrictHostKeyChecking=no \
        -o UserKnownHostsFile=/dev/null \
        -o BatchMode=yes \
        -o ConnectTimeout=5 \
        -i "$ssh_key" \
        -p "$ssh_port"
}

ssh_guest() {
    # shellcheck disable=SC2046
    ssh $(ssh_opts) "$ssh_user@127.0.0.1" "$@"
}

download_image() {
    mkdir -p "$cache_dir"
    if [ -s "$image" ]; then
        return
    fi
    need curl
    printf 'Downloading DragonFlyBSD image to %s\n' "$image"
    curl -L --fail -o "$image.tmp" "$image_url"
    mv "$image.tmp" "$image"
}

make_iso() {
    rm -f "$cidata_iso"
    if command -v hdiutil >/dev/null 2>&1; then
        hdiutil makehybrid -quiet -iso -joliet -default-volume-name cidata -o "$cidata_iso" "$cidata_dir"
    elif command -v xorrisofs >/dev/null 2>&1; then
        xorrisofs -quiet -output "$cidata_iso" -volid cidata -joliet -rock "$cidata_dir"
    elif command -v genisoimage >/dev/null 2>&1; then
        genisoimage -quiet -output "$cidata_iso" -volid cidata -joliet -rock "$cidata_dir"
    elif command -v mkisofs >/dev/null 2>&1; then
        mkisofs -quiet -output "$cidata_iso" -volid cidata -joliet -rock "$cidata_dir"
    else
        printf 'dragonfly-qemu: need hdiutil, xorrisofs, genisoimage, or mkisofs to create cidata.iso\n' >&2
        exit 1
    fi
}

create_cidata() {
    if [ ! -r "$ssh_pub_key" ]; then
        printf 'dragonfly-qemu: missing SSH public key: %s\n' "$ssh_pub_key" >&2
        printf 'Create one with Lima first, or set DRAGONFLY_SSH_KEY/DRAGONFLY_SSH_PUB_KEY.\n' >&2
        exit 1
    fi

    mkdir -p "$cidata_dir"

    cat >"$cidata_dir/meta-data" <<EOF
instance-id: iid-oddutils-dragonfly-qemu
local-hostname: oddutils-dragonfly-qemu
EOF

    cat >"$cidata_dir/network-config" <<'EOF'
version: 2
ethernets:
  vtnet0:
    dhcp4: true
EOF

    cat >"$cidata_dir/user-data" <<EOF
#cloud-config
users:
  - name: $ssh_user
    gecos: $ssh_user
    homedir: /home/$ssh_user
    shell: /bin/sh
    groups: wheel
    lock_passwd: true
    ssh_authorized_keys:
      - $(cat "$ssh_pub_key")

write_files:
  - path: /usr/local/etc/sudoers.d/oddutils
    owner: root:wheel
    permissions: '0440'
    content: |
      $ssh_user ALL=(ALL) NOPASSWD: ALL
  - path: /etc/resolv.conf
    owner: root:wheel
    permissions: '0644'
    content: |
      nameserver 1.1.1.1
      nameserver 8.8.8.8

bootcmd:
  - [ sh, -c, 'ifconfig vtnet0 up || true' ]
  - [ sh, -c, 'dhclient vtnet0 || true' ]

runcmd:
  - [ sh, -c, 'ifconfig vtnet0 up || true' ]
  - [ sh, -c, 'dhclient vtnet0 || true' ]
EOF

    make_iso
}

create_disk() {
    need "$qemu_img"
    if [ -s "$disk" ]; then
        return
    fi
    "$qemu_img" create -f qcow2 -F qcow2 -b "$image" "$disk" "$disk_size"
}

create() {
    mkdir -p "$state_dir"
    download_image
    create_cidata
    create_disk
    printf 'DragonFlyBSD QEMU state is ready in %s\n' "$state_dir"
}

is_running() {
    [ -s "$pid_file" ] || return 1
    pid="$(cat "$pid_file" 2>/dev/null)" || return 1
    [ -n "$pid" ] || return 1
    kill -0 "$pid" >/dev/null 2>&1
}

start() {
    need "$qemu_system"
    create
    if is_running; then
        printf 'DragonFlyBSD QEMU guest is already running on SSH port %s\n' "$ssh_port"
        return
    fi

    rm -f "$pid_file"
    : >"$serial_log"
    : >"$qemu_log"

    "$qemu_system" \
        -m "$memory" \
        -cpu max,-avx512vl,-pdpe1gb \
        -machine q35,vmport=off \
        -accel tcg,thread=multi,tb-size=512 \
        -global ICH9-LPC.disable_s3=1 \
        -global ICH9-LPC.disable_s4=1 \
        -smp "$cpus",sockets=1,cores="$cpus",threads=1 \
        -drive "file=$disk,if=none,discard=on,id=boot-disk" \
        -device virtio-blk-pci,drive=boot-disk,bootindex=1 \
        -boot order=c,splash-time=0,menu=on \
        -drive "id=cdrom0,if=none,format=raw,readonly=on,file=$cidata_iso" \
        -device virtio-scsi,id=scsi0 \
        -device scsi-cd,bus=scsi0.0,drive=cdrom0 \
        -netdev "user,id=net0,hostfwd=tcp:127.0.0.1:$ssh_port-:22" \
        -device "virtio-net-pci,netdev=net0,mac=$mac_address" \
        -device virtio-rng-pci \
        -display none \
        -vga none \
        -parallel none \
        -serial "file:$serial_log" \
        -daemonize \
        -pidfile "$pid_file" \
        >"$qemu_log" 2>&1

    sleep 1
    if ! is_running; then
        printf 'dragonfly-qemu: QEMU exited while starting; QEMU log follows\n' >&2
        cat "$qemu_log" >&2 || true
        printf 'dragonfly-qemu: serial log follows\n' >&2
        tail -n 80 "$serial_log" >&2 || true
        exit 1
    fi
    printf 'Started DragonFlyBSD QEMU guest on SSH port %s\n' "$ssh_port"
}

wait_ssh() {
    i=0
    while [ "$i" -lt 120 ]; do
        if ssh_guest 'uname -s' >/dev/null 2>&1; then
            return
        fi
        i=$((i + 1))
        sleep 2
    done
    printf 'dragonfly-qemu: SSH did not become ready; serial log follows\n' >&2
    tail -n 120 "$serial_log" >&2 || true
    exit 1
}

stop() {
    if ! is_running; then
        printf 'DragonFlyBSD QEMU guest is not running\n'
        return
    fi
    kill "$(cat "$pid_file")"
    i=0
    while is_running && [ "$i" -lt 30 ]; do
        i=$((i + 1))
        sleep 1
    done
    if is_running; then
        printf 'dragonfly-qemu: guest did not stop after SIGTERM\n' >&2
        exit 1
    fi
    rm -f "$pid_file"
    printf 'Stopped DragonFlyBSD QEMU guest\n'
}

delete_state() {
    stop
    case "$state_dir" in
        ""|"/"|"$HOME"|"$HOME/"|"/tmp"|"/private/tmp")
            printf 'dragonfly-qemu: refusing unsafe DRAGONFLY_STATE_DIR: %s\n' "$state_dir" >&2
            exit 1
            ;;
    esac
    rm -rf "$state_dir"
    printf 'Deleted DragonFlyBSD QEMU state from %s\n' "$state_dir"
}

setup_guest() {
    start
    wait_ssh
    ssh_guest 'command -v cargo >/dev/null && command -v scdoc >/dev/null && command -v just >/dev/null || sudo pkg install -y rust scdoc just'
    if ! ssh_guest 'cargo --version' >/dev/null 2>&1; then
        ssh_guest 'sudo rm -rf /tmp/oddutils-pkgfetch && mkdir -p /tmp/oddutils-pkgfetch && sudo pkg fetch -y -o /tmp/oddutils-pkgfetch openssl && sudo pkg add -f /tmp/oddutils-pkgfetch/All/openssl-*.pkg'
        ssh_guest 'cargo --version' >/dev/null
    fi
}

sync_repo() {
    start
    wait_ssh
    tar --exclude .git --exclude target --exclude .lima/dragonflybsd-hammer2.yaml -cf - -C "$repo_dir" . \
        | ssh $(ssh_opts) "$ssh_user@127.0.0.1" "rm -rf '$workdir' && mkdir -p '$workdir' && tar -xf - -C '$workdir'"
}

run_tests() {
    setup_guest
    sync_repo
    ssh_guest "cd '$workdir' && env TERM=xterm-256color just test"
}

open_shell() {
    setup_guest
    ssh -t -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -i "$ssh_key" -p "$ssh_port" "$ssh_user@127.0.0.1" "export TERM=xterm-256color; cd '$workdir' 2>/dev/null || cd /tmp; pwd; exec sh -i"
}

case "${1:-}" in
    create) create ;;
    start) start ;;
    stop) stop ;;
    delete) delete_state ;;
    setup) setup_guest ;;
    sync) sync_repo ;;
    test) run_tests ;;
    shell) open_shell ;;
    logs) tail -n 200 "$serial_log" ;;
    -h|--help|help) usage ;;
    *) usage >&2; exit 2 ;;
esac
