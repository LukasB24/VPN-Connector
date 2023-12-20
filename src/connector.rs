pub mod connector {
    use std::thread;
    use std::time::Duration;

    pub(crate) trait Connector<'b> {
        fn new(vpn_name: &'b str) -> Self;

        fn is_connected(&self, connection_name: &str) -> bool;

        fn connect_to_vpn(&self) -> bool;

        fn run_vpn_connector(&self, connections: &Vec<String>){
            loop {
                let mut connected: bool = false;
                println!("Check connection status...\n");
        
                for connection in &connections[2..] {
                    connected = self.is_connected(connection);
        
                    if connected {
                        println!("Status: {}OK\n{}Connected to secure network\n", "\x1B[32m", "\x1B[0m");
                        thread::sleep(Duration::from_secs(30));
                        break;
                    }
                }
        
                if !connected {
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