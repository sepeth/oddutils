chronic(1)

# NAME

chronic - run a command quietly unless it fails

# SYNOPSIS

*chronic* [*-e*] [*-v*] _command_ [_argument_...]

# DESCRIPTION

*chronic* runs a command and captures its standard output and standard error.
If the command exits successfully, captured output is suppressed. If the command
exits nonzero or is terminated abnormally, the captured output is replayed.

# OPTIONS

*-e*
	Also treat nonempty standard error as a failure trigger. If the command exits
	successfully but writes to standard error, *chronic* exits 2.

*-v*
	Print STDOUT, STDERR, and RETVAL labels around replayed output.

# EXIT STATUS

Returns the command exit status for ordinary command failures, 2 for *-e*
stderr-triggered failures, and 0 when output is suppressed after success.
