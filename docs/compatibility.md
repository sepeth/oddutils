# Compatibility Notes

`oddutils` is a modern clone of `moreutils`, not a bug-compatible port. The
contract is documented behavior and common Unix workflows.

## sponge

Implemented behavior:

- `sponge [-a] [file]`
- reads all standard input before opening the output path
- writes to standard output if no file is supplied
- preserves permissions for an existing regular output file
- replaces regular output files atomically with `rename(2)` when possible
- falls back to direct writing for special files and symlinks
- with `-a`, produces original file contents followed by standard input for
  regular files

Intentional differences from upstream may be added here as they are discovered.

## ts

Implemented behavior:

- `ts [-i | -s] [-m] [format]`
- adds a timestamp plus one space to the beginning of each input line
- defaults to `%b %d %H:%M:%S` for absolute timestamps
- defaults to `%H:%M:%S` for `-i` and `-s`
- supports custom `strftime(3)` formats
- supports moreutils-style subsecond `%.S`, `%.s`, and `%.T` expansions

Known gap:

- `-r` timestamp conversion is not implemented yet. Upstream uses Perl
  `Date::Parse` and `Time::Duration`; oddutils needs a deliberate Rust date
  parsing strategy before enabling this mode.
