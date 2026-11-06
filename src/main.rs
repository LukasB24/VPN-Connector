use std::env;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use log::{error, info};

use config::Config;
use nmcli::Nmcli;

mod backend;
mod config;
mod logging;
mod monitor;
mod nmcli;
mod policy;

const STARTUP_WAIT: Duration = Duration::from_secs(60);

const USAGE: &str = "\
Usage: vpn_connector <vpn> <home network> [trusted network...]

  <vpn>              NetworkManager connection name of the VPN
  <home network>     connection your VPN points to; the VPN is disabled there
  [trusted network]  connections where no VPN is needed";

fn main() -> ExitCode {
    let Some(config) = Config::from_args(env::args().skip(1)) else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };

    let _logger = logging::init_or_terminal_only(&logging::log_dir());

    info!(
        "Waiting {}s for the network to start...",
        STARTUP_WAIT.as_secs()
    );
    thread::sleep(STARTUP_WAIT);

    if let Err(error) = monitor::run(&Nmcli, &config) {
        error!("{error}");
    }
    ExitCode::FAILURE
}
