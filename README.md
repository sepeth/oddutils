# oddutils

`oddutils` is a Rust implementation of the most useful ideas from
[`moreutils`](https://joeyh.name/code/moreutils/), the collection of Unix tools
that fill gaps left by the classic toolbox.

The project is Unix-only for now. It aims to be a modern clone: command names
and primary workflows should feel familiar to `moreutils` users, while the Rust
implementation may document intentional differences instead of preserving every
implementation quirk.

## Architecture

`oddutils` is a single Cargo workspace:

- `crates/oddutils-core`: shared Unix, IO, and command support.
- `crates/sponge`: the first utility, implemented as a standalone binary.

Each utility will be a separate binary crate so commands can be installed and
packaged like normal Unix tools while still sharing common code.

## License

Given that `moreutils` is used as a reference, `oddutils` is also licensed under the GNU General Public License version 2 only.
See `COPYING`.

## Utility Status

| Utility | Notes |
| --- | --- |
| `sponge` | Reads all input before writing the output file; supports `-a`. |
| `ts` | Timestamp standard input; `-r` supports common ISO, syslog, and mail-style timestamps. |
| `chronic` | Run a command quietly unless it fails. |
| `ifne` | Run a command if standard input is not empty. |
| `pee` | Tee standard input to pipes. |
| `mispipe` | Pipe two commands, returning the first status. |
| `isutf8` | Check input for valid UTF-8. |
| `errno` | Look up errno names and descriptions. |
| `vidir` | Edit directory entries in `$EDITOR`. |
| `vipe` | Insert an editor into a pipe. |
| `combine` | Combine line sets with boolean operations. |
| `zrun` | Run commands over compressed arguments. |
| `ifdata` | Read core network interface information. |
| `lckdo` | Run a command with a lock held. |
| `parallel` | Run commands in parallel. |

## Development

### Local

Enter the dev shell:

```sh
nix develop
```

Then run:

```sh
cargo test
cargo clippy --all-targets --all-features
cargo fmt --check
```

Generate manpages:

```sh
just man
```

### Installation

For a user-local install, binaries go to `~/.local/bin` and manpages go to
`~/.local/share/man/man1` by default:

```sh
just install-user
```

Install to a custom user prefix:

```sh
USER_PREFIX="$HOME/opt/oddutils" just install-user
```

Install system-wide under `/usr/local`:

```sh
just install-system
```

Set `COMMAND_PREFIX` to install command and manpage names alongside `moreutils`
without colliding with the original tools. For example, `sponge` installs as
`osponge`:

```sh
COMMAND_PREFIX=o just install-system
COMMAND_PREFIX=o just install-user
USER_PREFIX="$HOME/opt/oddutils" COMMAND_PREFIX=o just install-user
```

Override the install prefix or package into a staging root:

```sh
PREFIX="$HOME/.local" just install
DESTDIR=/tmp/pkgroot PREFIX=/usr/local just install
```

Make sure both locations are discoverable by your shell:

```sh
export PATH="$HOME/.local/bin:$PATH"
export MANPATH="$HOME/.local/share/man:$(manpath)"
```

### Container

Build and test in a Linux container:

```sh
just container-test
just container-image
docker run --rm oddutils ts --help
```

The final image installs binaries and manpages under `/usr/local`.
Set `CONTAINER_RUNTIME=podman` to use Podman instead of Docker.

### Lima

Build and test in a local Lima Debian trixie VM:

```sh
just lima-create
just lima-start
just lima-test
```

The Lima instance mounts the current checkout at `/workspace/oddutils`.
Set `LIMA_INSTANCE=name` to use a different instance name.
`just lima-shell` uses `TERM=xterm-256color` inside the VM so terminal editors
work from terminals whose terminfo entries are not installed in the guest.
Use `just lima-recreate` to delete and recreate that VM after template changes.

Build and test in a local Lima FreeBSD VM:

```sh
just lima-freebsd-create
just lima-freebsd-start
just lima-freebsd-test
```

The FreeBSD instance also mounts the current checkout at `/workspace/oddutils`.
Set `LIMA_FREEBSD_INSTANCE=name` to use a different instance name.
Use `just lima-freebsd-recreate` to delete and recreate that VM after template changes.

Build and test in an experimental local Lima OpenBSD VM:

```sh
just lima-openbsd-create
just lima-openbsd-start
just lima-openbsd-test
```

This uses an unofficial OpenBSD 7.9 cloud-init image because Lima does not ship an
OpenBSD template and OpenBSD does not publish official cloud images. The image is
x86_64-only, so it runs under QEMU emulation on Apple Silicon. The OpenBSD VM runs
without Lima guest integration, so `just lima-openbsd-start` completes Lima's boot
marker over SSH, and `just lima-openbsd-test` copies the checkout into `/tmp/oddutils`
before running tests. Set `LIMA_OPENBSD_INSTANCE=name` to use a different instance
name, or `LIMA_OPENBSD_WORKDIR=/path` to change the guest workdir.

Build and test in an experimental local DragonFlyBSD QEMU VM:

```sh
just dragonfly-qemu-test
```

DragonFlyBSD does not publish official cloud images, and Lima does not currently
generate DragonFly-compatible cloud-init network metadata. This harness uses the
bsd-cloud-image.org DragonFlyBSD 6.4 HAMMER2 image directly with QEMU and writes
NoCloud metadata for DragonFly's `vtnet0` network interface. It keeps the base
image under `~/.cache/oddutils/dragonfly-qemu` and VM state under
`~/.local/state/oddutils/dragonfly-qemu`.

Useful commands:

```sh
just dragonfly-qemu-start
just dragonfly-qemu-shell
just dragonfly-qemu-logs
just dragonfly-qemu-stop
just dragonfly-qemu-delete
```

Set `DRAGONFLY_SSH_PORT`, `DRAGONFLY_STATE_DIR`, or `DRAGONFLY_CACHE_DIR` to
customize the local VM. The harness uses Lima's SSH key from `~/.lima/_config/user`
by default; set `DRAGONFLY_SSH_KEY` and `DRAGONFLY_SSH_PUB_KEY` to use another key.
