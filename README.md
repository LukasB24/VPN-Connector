# VPN-Connector
A small program that connects your PC to VPN, if you're not connected to a specified wifi.

## Note: 
- VPN-Connector uses "nmcli" to check and to establish connections. Therefore this application is only running on Linux devices.
- You need to set up a vpn connection inside networkmanager.
- It's possible to add multiple wifi connections as arguments but make sure to specify the VPN name as first argument.
- The program waits 60 seconds after start, then reacts to every network change reported by "nmcli monitor". It establishes the VPN connection if you're connected to neither the home network, a secure network nor the VPN.
- All names are NetworkManager connection names (for wifi usually the SSID).
- The first network argument is your home network: the VPN is disabled while you are connected to it. Further arguments are trusted networks, where the VPN is not needed.

## Getting started
1. download rusts build tool cargo from "https://www.rust-lang.org/learn/get-started"
2. cd to vpnConnector directory
3. *cargo build*
4. *cargo run **VPN-name** **wifi-ssid***
5. to run program directly:
    - cd to location of binary
    - *./vpn_connector **VPN-name** **wifi-ssid***

## Logs
Logs are written to `logs/` inside the project directory (fixed at build time, so rebuild if you move the project) and also shown in the terminal. Files rotate at 1 MB, the newest 5 are kept. Use `RUST_LOG=debug` for more detail.
