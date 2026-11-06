use std::fmt;

use crate::config::Config;

#[derive(Debug, PartialEq)]
pub enum Action {
    Connect,
    Disconnect,
    Stay(Reason),
}

#[derive(Debug, PartialEq)]
pub enum Reason {
    AtHome(String),
    OnVpn,
    OnTrusted(String),
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Reason::AtHome(name) => write!(f, "Connected to home network: {name}"),
            Reason::OnVpn => write!(f, "Connected to VPN"),
            Reason::OnTrusted(name) => write!(f, "Connected to secure network: {name}"),
        }
    }
}

pub fn decide(active: &[String], config: &Config) -> Action {
    let is_active = |name: &str| active.iter().any(|connection| connection == name);
    let on_vpn = is_active(&config.vpn);

    if is_active(&config.home_network) {
        return if on_vpn {
            Action::Disconnect
        } else {
            Action::Stay(Reason::AtHome(config.home_network.clone()))
        };
    }

    if on_vpn {
        return Action::Stay(Reason::OnVpn);
    }

    match config.trusted_networks.iter().find(|name| is_active(name)) {
        Some(name) => Action::Stay(Reason::OnTrusted(name.clone())),
        None => Action::Connect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            vpn: "vpn".into(),
            home_network: "home".into(),
            trusted_networks: vec!["office".into()],
        }
    }

    fn decide_for(active: &[&str]) -> Action {
        let active: Vec<String> = active.iter().map(|name| name.to_string()).collect();
        decide(&active, &config())
    }

    #[test]
    fn disables_vpn_at_home() {
        assert_eq!(decide_for(&["home", "vpn"]), Action::Disconnect);
    }

    #[test]
    fn stays_at_home_without_vpn() {
        assert_eq!(
            decide_for(&["home"]),
            Action::Stay(Reason::AtHome("home".into()))
        );
    }

    #[test]
    fn keeps_existing_vpn() {
        assert_eq!(decide_for(&["cafe", "vpn"]), Action::Stay(Reason::OnVpn));
    }

    #[test]
    fn trusts_secure_network() {
        assert_eq!(
            decide_for(&["office"]),
            Action::Stay(Reason::OnTrusted("office".into()))
        );
    }

    #[test]
    fn connects_vpn_on_unknown_network() {
        assert_eq!(decide_for(&["cafe"]), Action::Connect);
    }

    #[test]
    fn connects_vpn_when_offline() {
        assert_eq!(decide_for(&[]), Action::Connect);
    }

    #[test]
    fn reason_display() {
        assert_eq!(
            Reason::AtHome("home".into()).to_string(),
            "Connected to home network: home"
        );
        assert_eq!(Reason::OnVpn.to_string(), "Connected to VPN");
        assert_eq!(
            Reason::OnTrusted("office".into()).to_string(),
            "Connected to secure network: office"
        );
    }
}
