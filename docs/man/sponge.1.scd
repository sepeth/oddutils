sponge(1)

# NAME

sponge - soak up standard input and write to a file

# SYNOPSIS

*sponge* [*-a*] [_file_]

# DESCRIPTION

*sponge* reads all standard input before writing to _file_. This makes it useful
in pipelines that read from and write to the same file.

If no file is specified, *sponge* writes the captured input to standard output.

# OPTIONS

*-a*
	Append standard input to the original file contents, replacing the file with
	the combined contents.

# EXIT STATUS

Returns 0 on success and nonzero on read, write, open, or rename errors.
