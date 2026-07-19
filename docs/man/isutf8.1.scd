isutf8(1)

# NAME

isutf8 - check whether files are valid UTF-8

# SYNOPSIS

*isutf8* [_option_...] [_file_...]

# DESCRIPTION

*isutf8* checks whether input is syntactically valid UTF-8. It reads named
files, or standard input when no files are supplied.

# OPTIONS

*-q*, *--quiet*
	Suppress normal output and report only through the exit status.

*-l*, *--list*, *--list-only*
	Print only names of files containing invalid UTF-8.

*-i*, *--invert*
	List valid UTF-8 files instead of invalid files.

*-v*, *--verbose*
	Print extra byte context for the first error.

# EXIT STATUS

Returns 0 when all checked input is valid UTF-8. Returns nonzero for invalid
input or read errors.
