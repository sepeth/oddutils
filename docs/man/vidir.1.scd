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
	*vi* is used.

# COMPATIBILITY

Swap/conflict handling is simpler than upstream, and stdin is not reopened from
*/dev/tty* after reading a *-* path list.

# EXIT STATUS

Returns 0 when requested filesystem operations succeed, otherwise nonzero.
