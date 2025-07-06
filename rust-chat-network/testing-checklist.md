# Network Setup Testing Checklist

Use this checklist to verify your network setup is working correctly.

## ✅ Local Testing (Same Computer)

- [ ] Start server: `cargo run --bin server`
- [ ] Server shows "Chat server running on 0.0.0.0:8080"
- [ ] Start client: `cargo run --bin client`  
- [ ] Enter address: `127.0.0.1:8080`
- [ ] Client connects successfully
- [ ] Can create room and chat

## ✅ LAN Testing (Same Network)

- [ ] Find server's local IP:
  - Windows: `ipconfig` → look for IPv4 Address
  - Linux/Mac: `ifconfig` or `ip addr`
  - Example: `192.168.1.100`
- [ ] On another computer/device on same WiFi
- [ ] Run client with server's local IP: `192.168.1.100:8080`
- [ ] Can join rooms and chat

## ✅ Firewall Configuration

- [ ] Windows: Allow through firewall
  ```cmd
  netsh advfirewall firewall add rule name="Rust Chat Server" dir=in action=allow protocol=TCP localport=8080
  ```
- [ ] Linux: Open port
  ```bash
  sudo ufw allow 8080/tcp
  ```
- [ ] Test: Can still connect from LAN

## ✅ Internet Testing (Port Forwarding)

- [ ] Access router admin panel (usually `192.168.1.1`)
- [ ] Create port forwarding rule:
  - External Port: 8080
  - Internal Port: 8080  
  - Internal IP: Your computer's local IP
  - Protocol: TCP
- [ ] Find public IP: https://whatismyipaddress.com
- [ ] Have friend test: `YourPublicIP:8080`
- [ ] Can connect and chat

## ✅ Internet Testing (ngrok)

- [ ] Download ngrok: https://ngrok.com/download
- [ ] Server running: `cargo run --bin server`
- [ ] Start tunnel: `ngrok tcp 8080`
- [ ] See forwarding URL (e.g., `tcp://2.tcp.ngrok.io:12345`)
- [ ] Test locally: Connect to ngrok URL
- [ ] Have friend test: Share ngrok URL
- [ ] Can connect and chat

## 🔧 Troubleshooting Guide

### ❌ "Connection Refused"
- Is the server running?
- Correct IP address and port?
- Firewall blocking?

### ❌ "Connection Timeout"  
- Port forwarding configured correctly?
- Public IP correct?
- ISP blocking port 8080? Try 8888

### ❌ "Address Already in Use"
- Another program using port 8080
- Change port or kill other program

### ❌ Can't Connect from Internet
- Router supports port forwarding?
- ISP using CGNAT? (Use ngrok instead)
- Dynamic IP changed? (Check public IP)

## 📊 Performance Expectations

| Connection Type | Latency | Reliability |
|----------------|---------|-------------|
| Local (127.0.0.1) | <1ms | Excellent |
| LAN (192.168.x.x) | 1-5ms | Excellent |
| Internet (Direct) | 10-100ms | Good |
| Internet (ngrok) | 50-200ms | Good |

## 🎉 Success Indicators

- Multiple users can connect simultaneously
- Messages appear instantly for all users
- Room creation/joining works smoothly
- Disconnections handled gracefully
- Can reconnect after network issues