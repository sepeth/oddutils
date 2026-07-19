parallel(1)

# NAME

parallel - run commands in parallel

# SYNOPSIS

*parallel* [_options_] _command_ *--* _argument_...

*parallel* [_options_] *--* _command_...

# DESCRIPTION

*parallel* runs jobs concurrently. In command mode, it runs _command_ once for
each argument after *--*. In raw command mode, with no command before *--*, each
post-separator argument is run as a shell command.

# OPTIONS

*-j* _maxjobs_
	Limit how many jobs run at the same time. The default is one job per CPU.

*-i*
	Replace instances of *{}* in the command with the current argument.

*-n* _count_
	Pass _count_ arguments to each command invocation. Incompatible with *-i*.

*-l* _maxload_
	Wait to start new jobs while the one-minute load average is at or above
	_maxload_.

# COMPATIBILITY

Output is captured per job and replayed when the job finishes, instead of being
streamed through live serialization pipes.

# EXIT STATUS

Returns the bitwise OR of child exit statuses.
