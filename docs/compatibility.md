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
