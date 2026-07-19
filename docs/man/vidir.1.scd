vidir(1)

# NAME

vidir - edit directories and filenames

# SYNOPSIS

*vidir* [*--verbose*] [_directory_|_file_|*-*]...

# DESCRIPTION

*vidir* opens a numbered list of paths in an editor. Edit path text to rename
items. Delete lines to remove items. Directory arguments are expanded to their
direct children. Use *-* to read newline-delimited paths from standard input.

# OPTIONS

*-v*, *--verbose*
	Print actions taken by the program.

# ENVIRONMENT

*VISUAL*, *EDITOR*
	Select the editor. *VISUAL* takes precedence over *EDITOR*. If neither is set,
	*/usr/bin/editor* is used when available, then *vi*.

When available, the editor is attached to */dev/tty*.

# COMPATIBILITY

Swap/conflict handling is simpler than upstream and needs broader compatibility
tests.

# EXIT STATUS

Returns 0 when requested filesystem operations succeed, otherwise nonzero.
