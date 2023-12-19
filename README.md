# VPN-Connector
A small program that connects your PC to VPN, if you're not connected to a specified wifi.

## Note: 
- VPN-Connector uses "nmcli" to check and to establish connections. Therefore this application is only running on Linux devices.
- You need to set up a vpn connection inside networkmanager.
- It's possible to add multiple wifi connections as arguments but make sure to specify the VPN name as first argument.
- The program checks every 30 seconds for active connections and will automatically establish a VPN connection, if you're not connected to a specified wifi or vpn. 

## Getting started
1. download rusts build tool cargo from "https://www.rust-lang.org/learn/get-started"
2. cd to vpnConnector directory
3. *cargo build*
4. *cargo run **VPN-name** **wifi-ssid***
5. to run program directly:
    - cd to location of binary
    - *./vpn_connector **VPN-name** **wifi-ssid***
