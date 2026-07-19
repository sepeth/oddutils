use std::env;
use std::ffi::OsString;
use std::process::{Command, ExitCode};

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
        let output = Command::new("ifconfig").arg(iface).output();
        let Ok(output) = output else {
            return Self::missing();
        };
        if !output.status.success() {
            return Self::missing();
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let mut info = Self {
            exists: true,
            address: None,
            netmask: None,
            broadcast: None,
            mtu: None,
        };

        for line in text.lines() {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if info.mtu.is_none()
                && let Some(index) = parts.iter().position(|part| *part == "mtu")
            {
                info.mtu = parts.get(index + 1).and_then(|value| value.parse().ok());
            }
            if parts.first() == Some(&"inet") {
                info.address = parts.get(1).map(|value| (*value).to_string());
                for (index, part) in parts.iter().enumerate() {
                    match *part {
                        "netmask" => {
                            info.netmask = parts.get(index + 1).map(|value| parse_netmask(value));
                        }
                        "broadcast" => {
                            info.broadcast = parts.get(index + 1).map(|value| (*value).to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        info
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

fn parse_netmask(value: &str) -> String {
    if let Some(hex) = value.strip_prefix("0x")
        && let Ok(mask) = u32::from_str_radix(hex, 16)
    {
        return format!(
            "{}.{}.{}.{}",
            (mask >> 24) & 0xff,
            (mask >> 16) & 0xff,
            (mask >> 8) & 0xff,
            mask & 0xff
        );
    }
    value.to_string()
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
