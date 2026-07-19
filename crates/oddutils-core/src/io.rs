//! IO helpers for Unix filter-style commands.

use std::io::{self, Read, Write};

/// Copy bytes from `input` to `output`, treating a broken output pipe as a
/// normal early termination.
///
/// # Errors
///
/// Returns non-broken-pipe read or write errors from the underlying streams.
pub fn copy_ignoring_broken_pipe(input: &mut impl Read, output: &mut impl Write) -> io::Result<()> {
    match io::copy(input, output) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(error),
    }
}

/// Write all bytes to `output`, treating a broken output pipe as a normal early
/// termination.
///
/// # Errors
///
/// Returns non-broken-pipe write errors from the underlying stream.
pub fn write_all_ignoring_broken_pipe(output: &mut impl Write, buffer: &[u8]) -> io::Result<()> {
    match output.write_all(buffer) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(error),
    }
}
