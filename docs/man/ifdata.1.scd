ifdata(1)

# NAME

ifdata - get network interface information

# SYNOPSIS

*ifdata* _options_ _iface_

# DESCRIPTION

*ifdata* prints simple, script-friendly information about a network interface.
Multiple print options may be supplied; each result is printed on its own line.

# OPTIONS

*-e*
	Test whether _iface_ exists using the exit status.

*-pe*
	Print *yes* or *no* according to whether _iface_ exists.

*-p*
	Print address, netmask, broadcast address, and MTU.

*-pa*
	Print the IPv4 address.

*-pn*
	Print the netmask.

*-pN*
	Print the network address.

*-pb*
	Print the broadcast address.

*-pm*
	Print the MTU.

# COMPATIBILITY

Linux-only hardware address, flag, and statistics options are not implemented
yet.

# EXIT STATUS

Returns 0 for successful queries and for *-pe*. Returns nonzero when a required
interface does not exist or a requested option is unsupported.
