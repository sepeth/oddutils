errno(1)

# NAME

errno - look up errno names and descriptions

# SYNOPSIS

*errno* _name-or-code_...

*errno* [*-l*|*--list*]

*errno* [*-s*|*--search*] _word_...

*errno* [*-S*|*--search-all-locales*] _word_...

# DESCRIPTION

*errno* looks up errno macro names, numeric errno codes, and their platform
descriptions.

# OPTIONS

*-l*, *--list*
	List known errno values.

*-s*, *--search*
	Search descriptions case-insensitively for all supplied words.

*-S*, *--search-all-locales*
	Search descriptions in all locales reported by *locale -a*.

# EXIT STATUS

Returns 0 when all requested lookups are understood. Returns nonzero for unknown
names, unknown codes, usage errors, or locale search setup failures.
