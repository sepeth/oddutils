ts(1)

# NAME

ts - timestamp input

# SYNOPSIS

*ts* [*-r*] [*-i*|*-s*] [*-m*] [_format_]

# DESCRIPTION

*ts* adds a timestamp and a space to the beginning of each input line.

The default absolute timestamp format is *%b %d %H:%M:%S*. For incremental and
since-start modes, the default is *%H:%M:%S*.

Custom formats use *strftime*(3). The moreutils-style *%.S*, *%.s*, and *%.T*
forms add microsecond resolution.

# OPTIONS

*-i*
	Print time elapsed since the previous line.

*-s*
	Print time elapsed since program start.

*-m*
	Use a monotonic clock as the source for absolute timestamps.

*-r*
	Reserved for relative timestamp conversion. This mode is not implemented yet.

# EXIT STATUS

Returns 0 on success and nonzero on usage or I/O errors.
