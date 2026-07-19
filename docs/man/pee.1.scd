pee(1)

# NAME

pee - tee standard input to pipes

# SYNOPSIS

*pee* [*--ignore-sigpipe*|*--no-ignore-sigpipe*] [*--ignore-write-errors*|*--no-ignore-write-errors*] [_command_...]

# DESCRIPTION

*pee* is like *tee*(1) for commands. It runs each command through the shell and
writes a copy of standard input to each command. It does not copy input to its
own standard output; use *pee cat ...* for that.

# OPTIONS

*--ignore-sigpipe*, *--no-ignore-sigpipe*
	Control whether SIGPIPE is ignored. Ignoring is the default.

*--ignore-write-errors*, *--no-ignore-write-errors*
	Control whether pipe write errors are ignored. Ignoring is the default.

# EXIT STATUS

Returns the bitwise OR of child command exit statuses, or nonzero on write
failure when write errors are not ignored.
