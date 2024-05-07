use std::env;
use std::thread;
use std::time::Duration;
use linux_connector::linux_connector::LinuxVpnConnector;
use connector::connector::Connector;

mod linux_connector;
mod connector;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("First argument is VPN name.\nYou need to provide at least one WIFI-SSID as argument.\
        \nBeware that first WIFI-SSID is interpreted as home network, that your VPN points to. 
        \nUsage: ./vpn_connector <VPN-name> <home network name> [optional] <secure network 1> <secure network 2>");
        return;
    }

    else {
        let vpn_connector = LinuxVpnConnector::new(&args[1]);
        
        thread::sleep(Duration::from_secs(30)); // wait for networkmanager to start
        vpn_connector.run_vpn_connector(&args);
    }
}
