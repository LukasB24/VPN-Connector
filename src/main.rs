use std::process::Command;
use std::env;
use std::thread;
use std::time::Duration;

fn is_connected(name: &str, vpn: &str) -> bool {
    let output = Command::new("nmcli")
        .args(&["-t", "-f", "NAME,TYPE", "connection", "show", "--active"])
        .output()
        .expect("Failed to execute command");
    let output = String::from_utf8_lossy(&output.stdout);

    for line in output.lines() {
        let mut fields = line.split(':');
        let connection_name = fields.next().unwrap();
        let connection_type = fields.next().unwrap();

        if connection_name == name && connection_type == "802-11-wireless"
        || connection_name == vpn {
            return true;
        }
    }
    false
}

fn connect_to_vpn(name: &str) -> bool {
    let output = Command::new("nmcli")
        .args(&["connection", "up", name])
        .output();

    match output {
        Ok(output) => {
            return output.status.success();
        }
        Err(output) => {
            println!("Failed to execute command: {}", output);
            false
        }
    }
}

fn run_vpn_connector(args: &Vec<String>, vpn: &str){

    loop {
        let mut connected: bool = false;
        println!("Check connection status...\n");

        for connection in &args[2..] {
            connected = is_connected(connection, vpn);
            if connected {
                println!("Status: {}OK\n{}Connected to secure network\n", "\x1B[32m", "\x1B[0m");
                thread::sleep(Duration::from_secs(30));
                break;
            }
        }

        if !connected {
            println!("Status: {}NOT CONNECTED{}\nTrying to connect...\n", "\x1B[31m", "\x1B[0m");
            let success = connect_to_vpn(vpn);

            if !success {
                println!("Failed to connect to VPN: Check VPN name");
                return;
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("First argument is VPN name.\nYou need to provide at least one WLAN name as argument.\
        \n\nUsage: ./vpnConnector <VPN-name> <connection name>");
        return;
    }

    else {
        thread::sleep(Duration::from_secs(5)); // wait for network manager to start
        run_vpn_connector(&args, &args[1]);
    }
}
