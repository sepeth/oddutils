use std::env;
use std::ffi::{CStr, CString, OsString};
use std::net::Ipv4Addr;
use std::process::ExitCode;
use std::ptr;

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(Action::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Ok(Action::Run(config)) => {
            if run(&config) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("ifdata: {error}");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    actions: Vec<ActionKind>,
    iface: String,
}

#[derive(Debug)]
enum Action {
    Help,
    Run(Config),
}

#[derive(Debug, Clone, Copy)]
enum ActionKind {
    ExistsCode,
    PrintExists,
    PrintAll,
    Address,
    Netmask,
    Network,
    Broadcast,
    Mtu,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Action, String> {
        let mut actions = Vec::new();
        let mut iface = None;

        for arg in args {
            let text = arg.to_string_lossy();
            let action = match text.as_ref() {
                "-h" | "--help" => return Ok(Action::Help),
                "-e" => Some(ActionKind::ExistsCode),
                "-pe" => Some(ActionKind::PrintExists),
                "-p" => Some(ActionKind::PrintAll),
                "-pa" => Some(ActionKind::Address),
                "-pn" => Some(ActionKind::Netmask),
                "-pN" => Some(ActionKind::Network),
                "-pb" => Some(ActionKind::Broadcast),
                "-pm" => Some(ActionKind::Mtu),
                text if text.starts_with("-s") || text == "-ph" || text == "-pf" => {
                    return Err(format!("{text} is not implemented on this platform yet"));
                }
                text if text.starts_with('-') => return Err(format!("unknown option {text}")),
                _ => None,
            };

            if let Some(action) = action {
                actions.push(action);
            } else if iface.replace(text.into_owned()).is_some() {
                return Err("expected one interface name".to_string());
            }
        }

        let iface = iface.ok_or_else(|| "missing interface name".to_string())?;
        Ok(Action::Run(Self { actions, iface }))
    }
}

fn run(config: &Config) -> bool {
    if config.actions.is_empty() {
        return true;
    }

    let info = InterfaceInfo::load(&config.iface);
    for action in &config.actions {
        match action {
            ActionKind::ExistsCode => return info.exists,
            ActionKind::PrintExists => println!("{}", if info.exists { "yes" } else { "no" }),
            _ if !info.exists => {
                eprintln!("No such network interface: {}", config.iface);
                return false;
            }
            ActionKind::PrintAll => println!(
                "{} {} {} {}",
                info.address.as_deref().unwrap_or("NON-IP"),
                info.netmask.as_deref().unwrap_or("NON-IP"),
                info.broadcast.as_deref().unwrap_or("NON-IP"),
                info.mtu
                    .map_or_else(|| "0".to_string(), |mtu| mtu.to_string())
            ),
            ActionKind::Address => println!("{}", info.address.as_deref().unwrap_or("NON-IP")),
            ActionKind::Netmask => println!("{}", info.netmask.as_deref().unwrap_or("NON-IP")),
            ActionKind::Network => println!(
                "{}",
                network_address(info.address.as_deref(), info.netmask.as_deref())
                    .unwrap_or_else(|| "NON-IP".to_string())
            ),
            ActionKind::Broadcast => println!("{}", info.broadcast.as_deref().unwrap_or("NON-IP")),
            ActionKind::Mtu => println!("{}", info.mtu.unwrap_or(0)),
        }
    }

    true
}

#[derive(Debug)]
struct InterfaceInfo {
    exists: bool,
    address: Option<String>,
    netmask: Option<String>,
    broadcast: Option<String>,
    mtu: Option<u32>,
}

impl InterfaceInfo {
    fn load(iface: &str) -> Self {
        let Ok(c_iface) = CString::new(iface) else {
            return Self::missing();
        };
        let mut info = Self {
            exists: interface_exists(&c_iface),
            address: None,
            netmask: None,
            broadcast: None,
            mtu: interface_mtu(&c_iface),
        };

        let mut addrs = ptr::null_mut();
        // SAFETY: `addrs` is a valid out pointer. On success it is later
        // released with `freeifaddrs`.
        if unsafe { libc::getifaddrs(&raw mut addrs) } != 0 {
            return if info.exists { info } else { Self::missing() };
        }
        let _guard = IfAddrs(addrs);
        let mut current = addrs;
        while !current.is_null() {
            // SAFETY: `current` walks the linked list returned by getifaddrs.
            let entry = unsafe { &*current };
            if interface_name_matches(entry, &c_iface) {
                info.exists = true;
                info.capture_ipv4(entry);
            }
            current = entry.ifa_next;
        }

        if info.exists { info } else { Self::missing() }
    }

    fn capture_ipv4(&mut self, entry: &libc::ifaddrs) {
        if entry.ifa_addr.is_null() {
            return;
        }
        // SAFETY: `ifa_addr` is non-null and points to a sockaddr.
        if unsafe { (*entry.ifa_addr).sa_family } != af_inet() {
            return;
        }

        if self.address.is_none() {
            self.address = sockaddr_ipv4(entry.ifa_addr);
        }
        if self.netmask.is_none() {
            self.netmask = sockaddr_ipv4(entry.ifa_netmask);
        }
        if self.broadcast.is_none()
            && (entry.ifa_flags & u32::try_from(libc::IFF_BROADCAST).unwrap_or(0)) != 0
        {
            self.broadcast = sockaddr_ipv4(interface_broadcast(entry));
        }
    }

    fn missing() -> Self {
        Self {
            exists: false,
            address: None,
            netmask: None,
            broadcast: None,
            mtu: None,
        }
    }
}

struct IfAddrs(*mut libc::ifaddrs);

impl Drop for IfAddrs {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: `self.0` was returned by getifaddrs and has not been freed.
            unsafe {
                libc::freeifaddrs(self.0);
            }
        }
    }
}

fn interface_exists(iface: &CStr) -> bool {
    // SAFETY: `iface` is a NUL-terminated interface name.
    unsafe { libc::if_nametoindex(iface.as_ptr()) != 0 }
}

fn interface_name_matches(entry: &libc::ifaddrs, iface: &CStr) -> bool {
    if entry.ifa_name.is_null() {
        return false;
    }
    // SAFETY: getifaddrs provides NUL-terminated interface names.
    unsafe { CStr::from_ptr(entry.ifa_name) == iface }
}

fn sockaddr_ipv4(sockaddr_ptr: *const libc::sockaddr) -> Option<String> {
    if sockaddr_ptr.is_null() {
        return None;
    }
    // SAFETY: Caller checked the address family or accepts None for non-IPv4.
    let sockaddr = unsafe { &*sockaddr_ptr };
    if sockaddr.sa_family != af_inet() {
        return None;
    }
    // SAFETY: AF_INET sockaddr values have sockaddr_in layout.
    let inet = unsafe { ptr::read_unaligned(sockaddr_ptr.cast::<libc::sockaddr_in>()) };
    Some(Ipv4Addr::from(inet.sin_addr.s_addr.to_ne_bytes()).to_string())
}

fn af_inet() -> libc::sa_family_t {
    libc::sa_family_t::try_from(libc::AF_INET).unwrap_or_default()
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn interface_broadcast(entry: &libc::ifaddrs) -> *const libc::sockaddr {
    entry.ifa_ifu.cast_const()
}

#[cfg(not(any(target_os = "android", target_os = "linux")))]
fn interface_broadcast(entry: &libc::ifaddrs) -> *const libc::sockaddr {
    entry.ifa_dstaddr.cast_const()
}

#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd"
))]
fn interface_mtu(iface: &CStr) -> Option<u32> {
    // SAFETY: socket arguments are constant values for a datagram IPv4 socket.
    let socket = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if socket < 0 {
        return None;
    }

    // SAFETY: zeroed ifreq is a valid starting point before fields are filled
    // for SIOCGIFMTU.
    let mut request = unsafe { std::mem::zeroed::<libc::ifreq>() };
    copy_interface_name(&mut request.ifr_name, iface);

    // SAFETY: `request` points to writable ifreq storage for SIOCGIFMTU.
    let result = unsafe { libc::ioctl(socket, siocgifmtu(), &raw mut request) };
    // SAFETY: `socket` is an open file descriptor returned by socket.
    unsafe {
        libc::close(socket);
    }

    if result < 0 {
        None
    } else {
        // SAFETY: SIOCGIFMTU initializes the ifru_mtu union member on these
        // targets.
        u32::try_from(unsafe { request.ifr_ifru.ifru_mtu }).ok()
    }
}

#[cfg(any(target_os = "freebsd", target_os = "ios", target_os = "macos"))]
const fn siocgifmtu() -> libc::c_ulong {
    3_223_349_555
}

#[cfg(any(target_os = "android", target_os = "linux", target_os = "netbsd"))]
const fn siocgifmtu() -> libc::c_ulong {
    libc::SIOCGIFMTU
}

#[cfg(not(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd"
)))]
fn interface_mtu(_iface: &CStr) -> Option<u32> {
    None
}

#[allow(
    clippy::cast_possible_wrap,
    reason = "libc::c_char signedness is target-specific; preserve interface-name bytes"
)]
fn copy_interface_name(dest: &mut [libc::c_char], iface: &CStr) {
    let bytes = iface.to_bytes();
    let len = bytes.len().min(dest.len().saturating_sub(1));
    for (slot, byte) in dest.iter_mut().zip(bytes.iter().copied()).take(len) {
        *slot = byte as libc::c_char;
    }
}

fn network_address(address: Option<&str>, netmask: Option<&str>) -> Option<String> {
    let address = parse_ipv4(address?)?;
    let netmask = parse_ipv4(netmask?)?;
    let network = address & netmask;
    Some(format!(
        "{}.{}.{}.{}",
        (network >> 24) & 0xff,
        (network >> 16) & 0xff,
        (network >> 8) & 0xff,
        network & 0xff
    ))
}

fn parse_ipv4(value: &str) -> Option<u32> {
    let octets = value
        .split('.')
        .map(str::parse::<u8>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if octets.len() != 4 {
        return None;
    }
    Some(
        (u32::from(octets[0]) << 24)
            | (u32::from(octets[1]) << 16)
            | (u32::from(octets[2]) << 8)
            | u32::from(octets[3]),
    )
}

fn print_usage() {
    eprintln!("Usage: ifdata [options] iface");
    eprintln!("  -e     Reports interface existence via return code");
    eprintln!("  -p     Print whole config");
    eprintln!("  -pe    Print yes or no according to existence");
    eprintln!("  -pa    Print address");
    eprintln!("  -pn    Print netmask");
    eprintln!("  -pN    Print network address");
    eprintln!("  -pb    Print broadcast");
    eprintln!("  -pm    Print mtu");
}
