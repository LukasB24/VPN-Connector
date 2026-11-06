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

enum Action {
    DisableVpn,
    ConnectVpn,
    KeepAsIs(String),
}

fn decide(active: &[String], config: &VpnConfig) -> Action {
    let is_active = |name: &str| active.iter().any(|connection| connection == name);
    let connected_to_vpn = is_active(&config.vpn_name);

    if is_active(&config.home_ssid) {
        return if connected_to_vpn {
            Action::DisableVpn
        } else {
            Action::KeepAsIs(format!("Connected to home network: {}", config.home_ssid))
        };
    }

    if connected_to_vpn {
        return Action::KeepAsIs("Connected to VPN".into());
    }

    match config.secure_ssids.iter().find(|ssid| is_active(ssid)) {
        Some(ssid) => Action::KeepAsIs(format!("Connected to secure network: {ssid}")),
        None => Action::ConnectVpn,
    }
}

/// Returns `false` when the VPN could not be established and monitoring should stop.
fn ensure_secure_connection(connector: &impl Connector, config: &VpnConfig) -> bool {
    println!("{COLOR_WARN}Check connection status...{COLOR_RESET}\n");

    match decide(&connector.active_connections(), config) {
        Action::KeepAsIs(message) => print_ok(&message),
        Action::DisableVpn => {
            print_ok("Connected to VPN and home network. VPN will be disabled");
            if !connector.disconnect(&config.vpn_name) {
                eprintln!("Failed to disconnect from VPN");
            }
        }
        Action::ConnectVpn => {
            println!("Status: {COLOR_ERR}NOT CONNECTED{COLOR_RESET}\nTrying to connect...\n");
            if !connector.connect(&config.vpn_name) {
                eprintln!("Failed to connect to VPN: Check VPN name");
                return false;
            }
            print_ok("Successfully connected to VPN");
        }
    }
    true
}

fn print_ok(message: &str) {
    println!("Status: {COLOR_OK}OK{COLOR_RESET}\n{message}\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> VpnConfig {
        VpnConfig {
            vpn_name: "vpn".into(),
            home_ssid: "home".into(),
            secure_ssids: vec!["office".into()],
        }
    }

    fn active(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn disables_vpn_at_home() {
        let action = decide(&active(&["home", "vpn"]), &config());
        assert!(matches!(action, Action::DisableVpn));
    }

    #[test]
    fn keeps_state_at_home_without_vpn() {
        let action = decide(&active(&["home"]), &config());
        assert!(matches!(action, Action::KeepAsIs(_)));
    }

    #[test]
    fn keeps_existing_vpn() {
        let action = decide(&active(&["cafe", "vpn"]), &config());
        assert!(matches!(action, Action::KeepAsIs(_)));
    }

    #[test]
    fn trusts_secure_ssid() {
        let action = decide(&active(&["office"]), &config());
        assert!(matches!(action, Action::KeepAsIs(_)));
    }

    #[test]
    fn connects_vpn_on_unknown_network() {
        let action = decide(&active(&["cafe"]), &config());
        assert!(matches!(action, Action::ConnectVpn));
    }

    #[test]
    fn connects_vpn_when_offline() {
        let action = decide(&[], &config());
        assert!(matches!(action, Action::ConnectVpn));
    }
}
