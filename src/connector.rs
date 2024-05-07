pub mod connector {
    use std::thread;
    use std::time::Duration;

    pub(crate) trait Connector<'b> {
        fn new(vpn_name: &'b str) -> Self;

        fn is_connected(&self, connection_name: &str) -> bool;

        fn connect_to_vpn(&self) -> bool;

        fn disconnect_from_vpn(&self) -> bool;

        fn run_vpn_connector(&self, connections: &Vec<String>){
            loop {
                let mut connected_to_secure_network: bool = false;
                let connected_to_vpn: bool = self.is_connected(&connections[1]);
                let connected_to_home_network: bool = self.is_connected(&connections[2]);

                println!("Check connection status...\n");
                
                if connected_to_vpn && connected_to_home_network {
                    println!("Status: {}OK\n{}Connected to VPN and home network. VPN will be disabled\n", "\x1B[32m", "\x1B[0m");
                    self.disconnect_from_vpn();
                    thread::sleep(Duration::from_secs(30));
                    continue;
                }

                for connection in &connections[2..] {
                    connected_to_secure_network = self.is_connected(connection);
                    println!("Checking connection to secure network: {}\n", connection);

                    if connected_to_secure_network {
                        println!("Status: {}OK\n{}Connected to secure network\n", "\x1B[32m", "\x1B[0m");
                        thread::sleep(Duration::from_secs(30));
                        break;
                    }
                }
                
        
                if !connected_to_vpn && !(connected_to_secure_network || connected_to_home_network) {
                    println!("Status: {}NOT CONNECTED{}\nTrying to connect...\n", "\x1B[31m", "\x1B[0m");
                    let success: bool = self.connect_to_vpn();
        
                    if !success {
                        println!("Failed to connect to VPN: Check VPN name");
                        return;
                    }
                }
            }
        }
    }
}