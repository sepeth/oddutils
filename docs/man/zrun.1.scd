zrun(1)

# NAME

zrun - automatically uncompress arguments to a command

# SYNOPSIS

*zrun* _command_ _file.gz_...

# DESCRIPTION

*zrun* transparently decompresses compressed file arguments into temporary files
before running _command_. Uncompressed arguments are passed through unchanged.

Supported extensions are *gz*, *Z*, *bz2*, *xz*, *lzo*, *lzma*, and *zst*.

If installed under a name beginning with *z*, such as *zcatlike*, invocation is
treated as *zrun catlike ...*.

# COMPATIBILITY

Decompression currently runs sequentially rather than forking all preprocessors
in parallel.

# EXIT STATUS

Returns the command exit status. Returns nonzero if decompression or command
execution fails.
