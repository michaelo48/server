# Rust Chat - Network Edition

This is a network-enabled version of the Rust Chat application that can accept connections from other computers on your LAN or even the internet.

## Quick Start

### 1. Start the Server
```bash
cargo run --bin server
```
Or use the batch file:
```bash
start-network-server.bat
```

The server will listen on `0.0.0.0:8080` (all network interfaces).

### 2. Find Your IP Address
- **Windows**: Run `ipconfig` and look for "IPv4 Address"
- **Linux/Mac**: Run `ifconfig` or `ip addr`

### 3. Connect Clients

#### Same Computer
```bash
cargo run --bin client
# Enter: 127.0.0.1:8080
```

#### Same Network (LAN)
```bash
cargo run --bin client
# Enter: 192.168.1.100:8080  (use your server's local IP)
```

#### Different Network (Internet)
Requires port forwarding on your router or using a service like ngrok.

## Connection Methods

### Method 1: Interactive Prompt
```bash
cargo run --bin client
# You'll be prompted to enter the server address
```

### Method 2: Command Line Argument
```bash
cargo run --bin client 192.168.1.100:8080
```

### Method 3: Environment Variable
```bash
set CHAT_SERVER=192.168.1.100:8080
cargo run --bin client
```

### Method 4: Using Batch File
```bash
connect-to-server.bat 192.168.1.100:8080
```

## Internet Access Setup

### Option 1: Port Forwarding
1. Access your router's admin panel
2. Forward port 8080 to your computer's local IP
3. Find your public IP at https://whatismyipaddress.com
4. Share your public IP with friends: `86.123.45.67:8080`

### Option 2: Using ngrok (Easier)
1. Download ngrok from https://ngrok.com
2. Run your server: `cargo run --bin server`
3. In another terminal: `ngrok tcp 8080`
4. Share the ngrok URL: `2.tcp.ngrok.io:12345`

## Security Warnings

⚠️ **Exposing a server to the internet has security risks!**

For private use, consider:
- Using a VPN (WireGuard, Tailscale, etc.)
- Implementing authentication
- Using firewall rules to limit access
- Running on a cloud server instead of home network

## Troubleshooting

### "Connection Refused"
- Check if the server is running
- Verify firewall isn't blocking port 8080
- Ensure you're using the correct IP address

### "Connection Timeout"
- Check port forwarding configuration
- Verify your public IP hasn't changed
- Try a different port if ISP blocks 8080

### Windows Firewall
Allow the app through firewall:
```cmd
netsh advfirewall firewall add rule name="Rust Chat Server" dir=in action=allow protocol=TCP localport=8080
```

## Features
- Multi-network support (LAN and WAN)
- Configurable server address
- Network diagnostics in server output
- Connection status messages
- Same chat features as original version