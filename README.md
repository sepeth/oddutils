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

## Utility Status

| Utility | Status | Notes |
| --- | --- | --- |
| `sponge` | Initial implementation | Reads all input before writing the output file; supports `-a`. |
| `ts` | Initial implementation | Timestamp standard input; `-r` is not implemented yet. |
| `chronic` | Initial implementation | Run a command quietly unless it fails. |
| `ifne` | Initial implementation | Run a command if standard input is not empty. |
| `pee` | Initial implementation | Tee standard input to pipes. |
| `mispipe` | Initial implementation | Pipe two commands, returning the first status. |
| `isutf8` | Initial implementation | Check input for valid UTF-8. |
| `errno` | Planned | Look up errno names and descriptions. |
| `vidir` | Planned | Edit directory entries in `$EDITOR`. |
| `vipe` | Planned | Insert an editor into a pipe. |
| `combine` | Planned | Combine line sets with boolean operations. |
| `zrun` | Planned | Run commands over compressed arguments. |
| `ifdata` | Planned | Read network interface information. |
| `lckdo` | Planned | Deprecated upstream; likely lowest priority. |
| `parallel` | Planned | Lower priority because GNU parallel commonly owns the name. |

## Development

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

The upstream `moreutils` checkout is expected beside this repository at
`../moreutils` for reference and future compatibility comparisons.
