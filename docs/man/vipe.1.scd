vipe(1)

# NAME

vipe - insert an editor into a pipe

# SYNOPSIS

_command1_ | *vipe* [*--suffix* _extension_] | _command2_

# DESCRIPTION

*vipe* reads standard input into a temporary file, opens the file in an editor,
then writes the edited file to standard output.

# OPTIONS

*--suffix* _extension_, *--suffix=* _extension_
	Add a suffix to the temporary file name, useful for editor mode or syntax
	detection. The suffix may be supplied with or without a leading dot.

# ENVIRONMENT

*VISUAL*, *EDITOR*
	Select the editor. *VISUAL* takes precedence over *EDITOR*. If neither is set,
	*/usr/bin/editor* is used when available, then *vi*.

When available, the editor is attached to */dev/tty* so editor interaction does
not write into the pipeline.

# EXIT STATUS

Returns 0 on success and nonzero if the editor or I/O fails.
