mispipe(1)

# NAME

mispipe - pipe two commands, returning the first command's status

# SYNOPSIS

*mispipe* _command1_ _command2_

# DESCRIPTION

*mispipe* runs _command1_ and _command2_ through the shell, connecting
_command1_'s standard output to _command2_'s standard input. Unlike a normal
shell pipeline, *mispipe* exits with the status of _command1_.

# EXIT STATUS

Returns the exit status of _command1_. If _command1_ is signaled, returns
128 plus the signal number on Unix.
