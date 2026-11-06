pub struct Config {
    pub vpn: String,
    pub home_network: String,
    pub trusted_networks: Vec<String>,
}

impl Config {
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Option<Config> {
        let mut args = args.into_iter();
        let vpn = args.next()?;
        let home_network = args.next()?;

        Some(Config {
            vpn,
            home_network,
            trusted_networks: args.collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_vpn_home_and_trusted() {
        let config = Config::from_args(args(&["vpn", "home", "a", "b"])).unwrap();
        assert_eq!(config.vpn, "vpn");
        assert_eq!(config.home_network, "home");
        assert_eq!(config.trusted_networks, ["a", "b"]);
    }

    #[test]
    fn trusted_is_optional() {
        let config = Config::from_args(args(&["vpn", "home"])).unwrap();
        assert!(config.trusted_networks.is_empty());
    }

    #[test]
    fn rejects_missing_home() {
        assert!(Config::from_args(args(&["vpn"])).is_none());
    }
}
