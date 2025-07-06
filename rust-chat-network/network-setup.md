# Network Setup Guide for Rust Chat

## Making the Server Accessible from Other Networks

### 1. Server Modifications

In `server.rs`, change:
```rust
let addr = "127.0.0.1:8080";
```
to:
```rust
let addr = "0.0.0.0:8080";  // Listens on all network interfaces
```

### 2. Client Modifications

Make the server address configurable via:
- Environment variable
- Command-line argument
- Configuration file
- Or prompt the user

Example with environment variable:
```rust
let addr = std::env::var("CHAT_SERVER")
    .unwrap_or_else(|_| "127.0.0.1:8080".to_string());
```

### 3. Network Configuration Steps

#### A. Port Forwarding (Home Network)
1. Find your local IP: `ipconfig` (Windows) or `ifconfig` (Linux/Mac)
2. Access router admin panel (usually 192.168.1.1 or 192.168.0.1)
3. Set up port forwarding:
   - External Port: 8080
   - Internal Port: 8080
   - Internal IP: Your computer's local IP
   - Protocol: TCP

#### B. Firewall Rules
1. **Windows Firewall:**
   ```cmd
   netsh advfirewall firewall add rule name="Rust Chat Server" dir=in action=allow protocol=TCP localport=8080
   ```

2. **Linux (ufw):**
   ```bash
   sudo ufw allow 8080/tcp
   ```

#### C. Find Your Public IP
- Visit: https://whatismyipaddress.com/
- Or run: `curl ifconfig.me`

### 4. Connection Instructions

#### For Local Network (LAN):
- Server: Run on `0.0.0.0:8080`
- Clients: Connect to `<server-local-ip>:8080`
- Example: `192.168.1.100:8080`

#### For Internet (WAN):
- Server: Run on `0.0.0.0:8080` with port forwarding
- Clients: Connect to `<server-public-ip>:8080`
- Example: `86.123.45.67:8080`

### 5. Security Considerations

⚠️ **WARNING**: Exposing servers to the internet has security risks!

**Recommended Security Measures:**
1. **Use a VPN** (Preferred for private use)
   - Set up WireGuard, OpenVPN, or Tailscale
   - Only authorized users can connect

2. **Implement Authentication**
   - Add password protection
   - Use API keys or tokens

3. **Enable TLS/SSL**
   - Encrypt communication
   - Prevent eavesdropping

4. **Rate Limiting**
   - Prevent DoS attacks
   - Limit connections per IP

5. **Whitelist IPs** (if possible)
   - Only allow specific IP addresses

### 6. Cloud Hosting Alternative

Instead of home hosting, consider:
- **AWS EC2** (free tier available)
- **DigitalOcean** ($5/month droplets)
- **Linode**
- **Google Cloud Platform**

Example AWS EC2 setup:
1. Launch t2.micro instance (free tier)
2. Configure security group to allow TCP port 8080
3. Deploy your Rust binary
4. Use Elastic IP for stable address

### 7. Using ngrok (Quick Testing)

For temporary public access without port forwarding:
1. Download ngrok: https://ngrok.com/
2. Run your server: `cargo run --bin server`
3. In another terminal: `ngrok tcp 8080`
4. Share the ngrok URL with clients

Example:
```
Forwarding tcp://2.tcp.ngrok.io:12345 -> localhost:8080
```

Clients connect to: `2.tcp.ngrok.io:12345`

### 8. Modified Client Example

```rust
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get server address from environment or use default
    let addr = env::var("CHAT_SERVER")
        .unwrap_or_else(|_| {
            println!("Enter server address (e.g., 192.168.1.100:8080): ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        });
    
    println!("Connecting to chat server at {}", addr);
    let stream = TcpStream::connect(&addr).await?;
    // ... rest of client code
}
```

### 9. Testing Your Setup

1. **Local Test**: Connect from same machine
   ```
   CHAT_SERVER=127.0.0.1:8080 cargo run --bin client
   ```

2. **LAN Test**: Connect from another device on same network
   ```
   CHAT_SERVER=192.168.1.100:8080 cargo run --bin client
   ```

3. **WAN Test**: Have a friend connect from outside
   ```
   CHAT_SERVER=86.123.45.67:8080 cargo run --bin client
   ```

### 10. Troubleshooting

**Connection Refused:**
- Check firewall rules
- Verify port forwarding
- Ensure server is running

**Timeout:**
- Check public IP is correct
- Verify ISP doesn't block the port
- Try a different port (e.g., 8888)

**Can't connect from outside:**
- Some ISPs use CGNAT (Carrier-Grade NAT)
- Try using a VPN or cloud hosting
- Use ngrok for testing