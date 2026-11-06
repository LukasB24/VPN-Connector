use std::env;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use connector::{run_vpn_connector, VpnConfig};
use linux_connector::LinuxConnector;

mod connector;
mod linux_connector;

const STARTUP_WAIT_SECONDS: u64 = 10;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);

    let (Some(vpn_name), Some(home_ssid)) = (args.next(), args.next()) else {
        print_usage();
        return ExitCode::FAILURE;
    };

    let config = VpnConfig {
        vpn_name,
        home_ssid,
        secure_ssids: args.collect(),
    };

    thread::sleep(Duration::from_secs(STARTUP_WAIT_SECONDS));
    run_vpn_connector(&LinuxConnector, &config);

    ExitCode::FAILURE
}

fn print_usage() {
    eprintln!(
        "First argument is VPN name.\nYou need to provide at least one WIFI-SSID as argument.\
\nBeware that first WIFI-SSID is interpreted as home network, that your VPN points to.\
\nUsage: ./vpn_connector <VPN-name> <home network name> [optional] <secure network 1> <secure network 2>"
    );
}
