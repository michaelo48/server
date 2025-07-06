# Quick Internet Access with ngrok

ngrok is the easiest way to make your chat server accessible from the internet without dealing with port forwarding.

## Setup Steps

### 1. Download ngrok
Visit https://ngrok.com/download and download for your OS.

### 2. Extract and Add to PATH (Optional)
Extract ngrok.exe to a folder and add it to your system PATH for easy access.

### 3. Start Your Chat Server
```bash
cd rust-chat-network
cargo run --bin server
```

### 4. Start ngrok Tunnel
In a new terminal:
```bash
ngrok tcp 8080
```

You'll see output like:
```
Session Status                online
Account                       your-email@example.com (Plan: Free)
Version                       3.5.0
Region                        United States (us)
Forwarding                    tcp://2.tcp.ngrok.io:12345 -> localhost:8080
```

### 5. Share the Connection Info
Give your friends the forwarding address:
- Server: `2.tcp.ngrok.io`
- Port: `12345`
- Full address: `2.tcp.ngrok.io:12345`

### 6. Connect from Anywhere
Friends can now connect using:
```bash
cargo run --bin client
# Enter: 2.tcp.ngrok.io:12345
```

Or:
```bash
connect-to-server.bat 2.tcp.ngrok.io:12345
```

## Important Notes

- **Free tier limitations**: Connection may close after 8 hours
- **URL changes**: Each time you restart ngrok, you get a new URL
- **Latency**: Adds some delay compared to direct connection
- **Security**: Anyone with the URL can connect (implement auth if needed)

## Alternative: Tailscale (Permanent Solution)

For a more permanent solution with better security:
1. Install Tailscale on all computers
2. Each computer gets a permanent IP (100.x.x.x)
3. Only devices in your Tailscale network can connect
4. No port forwarding or public exposure needed

## Quick Test

1. Server terminal:
```
cargo run --bin server
```

2. ngrok terminal:
```
ngrok tcp 8080
```

3. Client terminal (even from another network):
```
set CHAT_SERVER=2.tcp.ngrok.io:12345
cargo run --bin client
```

That's it! Your chat server is now accessible from anywhere in the world.