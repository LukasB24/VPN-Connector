# VPN-Connector
A small program that connects your device with VPN, if you're not connected to a specified wifi.

Note: 
- VPN-Connector uses "nmcli" to check and establish connections. Therefore this application is only running on Linux devices.
- In order to work you need to set up a VPN-server.  
- It's possible to add multiple wifi connections as arguments but make sure to specify the vpn name as first argument.
- The program will check every 30 seconds for active connections and connect your device to VPN, if it doesn't find one of your specified connections.

# Getting started
1. cd to vpnConnector directory
2. run ./vpnConnector *VPN-name* *wifi-name*
