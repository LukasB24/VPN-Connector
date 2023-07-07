# VPN-Connector
A small program that connects your device to VPN, if you're not connected to a specified wifi.

## Note: 
- VPN-Connector uses "nmcli" to check and establish connections. Therefore this application is only running on Linux devices.
- In order to work you need to set up a VPN server.  
- It's possible to add multiple wifi connections as arguments but make sure to specify the VPN name as first argument.
- The program checks every 30 seconds for active connections and will automatically establish a VPN connection for your device if it fails to detect any. 

## Getting started
1. cd to vpnConnector directory
2. run *cargo build*
3. run *cargo run **VPN-name** **wifi-name***
