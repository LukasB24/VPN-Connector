pub mod linux_connector {
    use std::process::Command;
    use crate::connector::connector::Connector;

 
    pub struct LinuxVpnConnector<'a>{
        vpn_name: &'a str,
    }

    impl<'a> Connector<'a> for LinuxVpnConnector<'a>{
        fn new(vpn_name: &'a str) -> Self {
            LinuxVpnConnector{vpn_name}
        }

        fn is_connected(&self, connection: &str) -> bool {
            let output = Command::new("nmcli")
                .args(&["-t", "-f", "NAME,TYPE", "connection", "show", "--active"])
                .output()
                .expect("Failed to execute command");
            let output = String::from_utf8_lossy(&output.stdout);
    
            for line in output.lines() {
                let mut fields = line.split(':');
                let connection_name: &str = fields.next().unwrap();
    
                if connection_name == connection {
                    return true;
                }
            }
            false
        }
    
        fn connect_to_vpn(&self) -> bool {
            let output: Result<std::process::Output, std::io::Error> = Command::new("nmcli")
                .args(&["connection", "up", &self.vpn_name])
                .output();
    
            match output {
                Ok(output) => {
                    output.status.success()
                }
                Err(output) => {
                    println!("Failed to execute command: {}", output);
                    false
                }
            }
        }

        fn disconnect_from_vpn(&self) -> bool {
            let output: Result<std::process::Output, std::io::Error> = Command::new("nmcli")
                .args(&["connection", "down", &self.vpn_name])
                .output();
    
            match output {
                Ok(output) => {
                    output.status.success()
                }
                Err(output) => {
                    println!("Failed to execute command: {}", output);
                    false
                }
            }
        }
    }
}