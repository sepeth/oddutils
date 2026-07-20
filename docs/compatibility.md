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
- falls back to direct copy if an atomic rename fails
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
- accepts an empty format string
- flushes output after each stamped line

Known gap:

- `-r` timestamp conversion is implemented with a Rust parser for common ISO,
  syslog, and mail-style timestamps. This intentionally is not a byte-for-byte
  clone of Perl `Date::Parse`; oddutils also recognizes ISO timestamps without
  fractional seconds, such as `2026-07-20T18:31:19Z`.

## chronic

Implemented behavior:

- `chronic [-ev] COMMAND...`
- inherits standard input
- captures stdout and stderr while the child runs
- suppresses both streams when the child exits successfully
- replays captured output when the child exits nonzero or is signaled
- preserves ordinary nonzero child exit codes
- `-e` replays stderr and exits `2` when the child succeeds but writes stderr
- `-v` prints `STDOUT`, `STDERR`, and `RETVAL` labels around replayed output

## ifne

Implemented behavior:

- `ifne [-n] command [args]`
- exits successfully without running the command when stdin is empty
- runs the command with stdin forwarded when stdin is not empty
- starts the command after the initial nonempty read and streams the remaining
  stdin instead of buffering all input first
- with `-n`, runs the command only when stdin is empty
- with `-n` and nonempty stdin, writes stdin back to stdout without running the
  command
- returns the child exit status for ordinary child exits

## pee

Implemented behavior:

- `pee [--[no-]ignore-sigpipe] [--[no-]ignore-write-errors] [command ...]`
- runs each command through the shell and writes a copy of stdin to each
  command
- does not copy stdin to stdout by itself
- child stdout and stderr are inherited by `pee`
- defaults to ignoring SIGPIPE and write errors
- closes all child input pipes and waits for children on write-error shutdown
- returns the bitwise OR of child exit statuses

## mispipe

Implemented behavior:

- `mispipe command1 command2`
- runs both commands through the shell
- connects `command1` stdout to `command2` stdin
- inherits `mispipe` stdin for `command1`
- inherits stdout/stderr for `command2`
- returns the exit status of `command1`
- if `command1` is signaled, returns `128 + signal` on Unix

## isutf8

Implemented behavior:

- `isutf8 [OPTION]... [FILE]...`
- reads standard input when no files are supplied
- supports `-q/--quiet`, `-l/--list`, `--list-only`, `-i/--invert`, and
  `-v/--verbose`
- returns success only when all checked inputs are valid UTF-8
- lists valid files with `-i` and invalid files with `-l`

Known difference:

- invalid UTF-8 diagnostics use Rust's UTF-8 parser and simpler explanatory
  text instead of the upstream byte-range-specific messages.

## errno

Implemented behavior:

- `errno name-or-code`
- `errno -l/--list`
- `errno -s/--search word...`
- `errno -S/--search-all-locales word...`
- case-insensitive errno name lookup
- descriptions come from platform `strerror(3)`

Known gap:

- the errno table is curated for common Darwin/POSIX values rather than
  generated from the active C headers at build time.

## vipe

Implemented behavior:

- `vipe [--suffix=extension]`
- reads stdin into a temporary file
- runs `$VISUAL`, then `$EDITOR`, then `vi`
- uses `/usr/bin/editor` before `vi` when no editor environment variable is set
- attaches the editor to `/dev/tty` when available
- writes the edited temporary file to stdout
- supports suffix values with or without a leading dot

## vidir

Implemented behavior:

- `vidir [--verbose] [directory|file|-]...`
- defaults to editing the current directory
- expands directory arguments to their direct children
- reads newline-delimited paths from stdin for `-`
- writes numbered edit lines to a temporary file and runs `$VISUAL`, then
  `$EDITOR`, then `/usr/bin/editor`, then `vi`
- attaches the editor to `/dev/tty` when available
- renames changed paths and removes paths whose lines were deleted
- removes paths whose edited name is empty
- creates parent directories for edited target paths

Known differences:

- swap/conflict handling is simpler than upstream and needs broader
  compatibility tests.

## combine

Implemented behavior:

- `combine file1 and file2`
- `combine file1 not file2`
- `combine file1 or file2`
- `combine file1 xor file2`
- supports one `-` argument for stdin
- preserves upstream-style ordering and duplicate behavior
- preserves carriage returns in input lines like upstream Perl `chomp`

Known difference:

- using `-` for both files is rejected because stdin cannot be read twice.

## zrun

Implemented behavior:

- `zrun command file.gz [...]`
- transparently decompresses `gz`, `Z`, `bz2`, `xz`, `lzo`, `lzma`, and `zst`
  arguments to temporary files before running the command
- passes uncompressed arguments through unchanged
- returns the command exit status
- supports `z<program>` invocation behavior when installed under such a name
- launches preprocessors for compressed arguments before waiting for them

## ifdata

Implemented behavior:

- `ifdata -e iface`
- `ifdata -pe iface`
- `ifdata -p iface`
- `ifdata -pa/-pn/-pN/-pb/-pm iface`
- supports multiple print options and emits one line per option

Known gaps:

- Linux-only hardware-address, flag, and statistics options are not implemented
  yet.
- this implementation shells out to `ifconfig` instead of using ioctl calls
  directly.

## lckdo

Implemented behavior:

- `lckdo [options] lockfile program [arguments]`
- supports `-w`, `-W sec`, `-e`, `-E fd`, `-n`, `-q`, `-s`, `-x`, and `-t`
- uses Unix `flock`
- returns the child exit status after successfully acquiring the lock
- returns `EX_TEMPFAIL` (`75`) when the lock is held

## parallel

Implemented behavior:

- `parallel [OPTIONS] command -- arguments`
- `parallel [OPTIONS] -- commands`
- supports `-j maxjobs`, `-l maxload`, `-i`, and `-n args`
- runs raw commands through the shell in `parallel -- command...` mode
- ORs child exit statuses for its own exit status

Known gap:

- stdout/stderr are captured per job and replayed when each job finishes rather
  than streamed through live serialization pipes.
