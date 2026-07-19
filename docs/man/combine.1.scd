combine(1)

# NAME

combine - combine lines from two files using boolean operations

# SYNOPSIS

*combine* _file1_ *and* _file2_

*combine* _file1_ *not* _file2_

*combine* _file1_ *or* _file2_

*combine* _file1_ *xor* _file2_

# DESCRIPTION

*combine* combines lines from two files. The input files do not need to be
sorted. Output preserves the order in which lines occur in _file1_, followed by
the order in _file2_ for operations that emit lines from both files.

Use *-* for one input file to read from standard input.

# OPERATIONS

*and*
	Print lines from _file1_ that are also present in _file2_.

*not*
	Print lines from _file1_ that are not present in _file2_.

*or*
	Print all lines from _file1_ followed by all lines from _file2_.

*xor*
	Print lines present in either file, but not both.

# EXIT STATUS

Returns 0 on success and nonzero on input or usage errors.
