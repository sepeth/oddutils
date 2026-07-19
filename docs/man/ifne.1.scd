ifne(1)

# NAME

ifne - run a command if standard input is not empty

# SYNOPSIS

*ifne* [*-n*] _command_ [_argument_...]

# DESCRIPTION

*ifne* reads standard input. By default it runs _command_ only when input is not
empty, forwarding the captured input to the command.

# OPTIONS

*-n*
	Reverse the test. Run _command_ only when input is empty. If input is not
	empty, pass it through to standard output without running _command_.

# EXIT STATUS

Returns 0 when no command is run successfully. When a command is run, returns
the command's exit status.
