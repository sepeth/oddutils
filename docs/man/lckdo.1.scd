lckdo(1)

# NAME

lckdo - run a program with a lock held

# SYNOPSIS

*lckdo* [_options_] _lockfile_ _program_ [_argument_...]

# DESCRIPTION

*lckdo* runs a program while holding a Unix file lock on _lockfile_. It is
deprecated upstream in favor of *flock*(1), but is included for compatibility.

# OPTIONS

*-w*
	Wait indefinitely for the lock.

*-W* _seconds_
	Wait up to _seconds_ for the lock.

*-e*
	Execute the program directly after acquiring the lock.

*-E* _fd_
	Execute the program directly and keep the lock open on file descriptor _fd_.

*-n*
	Do not create _lockfile_ if it does not exist.

*-q*
	Suppress lock-failure output.

*-s*
	Take a shared lock.

*-x*
	Take an exclusive lock. This is the default.

*-t*
	Test whether the lock is held.

# EXIT STATUS

Returns the child exit status after acquiring the lock. Returns 75 when the lock
is held and 73 for lockfile open/create failures.
