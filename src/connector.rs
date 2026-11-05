use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

const COLOR_OK: &str = "\x1B[32m";
const COLOR_WARN: &str = "\x1B[38;2;255;165;0m";
const COLOR_ERR: &str = "\x1B[31m";
const COLOR_RESET: &str = "\x1B[0m";

const SETTLE_SECONDS: u64 = 2;

pub struct VpnConfig {
    pub vpn_name: String,
    pub home_ssid: String,
    pub secure_ssids: Vec<String>,
}

pub trait Connector {
    fn active_connections(&self) -> Vec<String>;

    fn watch_network_changes(&self) -> Option<Receiver<()>>;

    fn connect(&self, name: &str) -> bool;

    fn disconnect(&self, name: &str) -> bool;
}

pub fn run_vpn_connector(connector: &impl Connector, config: &VpnConfig) {
    let Some(changes) = connector.watch_network_changes() else {
        return;
    };

    while ensure_secure_connection(connector, config) {
        if changes.recv().is_err() {
            eprintln!("Network monitor stopped");
            return;
        }

        thread::sleep(Duration::from_secs(SETTLE_SECONDS));
        while changes.try_recv().is_ok() {}
    }
}

fn ensure_secure_connection(connector: &impl Connector, config: &VpnConfig) -> bool {
    println!("{COLOR_WARN}Check connection status...{COLOR_RESET}\n");

    let active = connector.active_connections();
    let is_active = |name: &str| active.iter().any(|connection| connection == name);
    let connected_to_vpn = is_active(&config.vpn_name);

    if is_active(&config.home_ssid) {
        if connected_to_vpn {
            print_ok("Connected to VPN and home network. VPN will be disabled");
            if !connector.disconnect(&config.vpn_name) {
                eprintln!("Failed to disconnect from VPN");
            }
        } else {
            print_ok(&format!("Connected to home network: {}", config.home_ssid));
        }
        return true;
    }

    if connected_to_vpn {
        print_ok("Connected to VPN");
        return true;
    }

    if let Some(ssid) = config.secure_ssids.iter().find(|ssid| is_active(ssid)) {
        print_ok(&format!("Connected to secure network: {ssid}"));
        return true;
    }

    println!("Status: {COLOR_ERR}NOT CONNECTED{COLOR_RESET}\nTrying to connect...\n");
    if !connector.connect(&config.vpn_name) {
        eprintln!("Failed to connect to VPN: Check VPN name");
        return false;
    }

    print_ok("Successfully connected to VPN");
    true
}

fn print_ok(message: &str) {
    println!("Status: {COLOR_OK}OK{COLOR_RESET}\n{message}\n");
}
