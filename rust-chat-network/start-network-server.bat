@echo off
echo === Rust Chat Network Server ===
echo.
echo This server will be accessible from other networks!
echo.
echo Server will listen on: 0.0.0.0:8080
echo This means it accepts connections from:
echo   - Localhost (127.0.0.1:8080)
echo   - LAN (your-local-ip:8080)
echo   - Internet (your-public-ip:8080) - requires port forwarding
echo.
echo To find your local IP address:
echo   Run: ipconfig
echo   Look for "IPv4 Address"
echo.
echo SECURITY WARNING: Exposing servers to the internet has risks!
echo Consider using a VPN or implementing authentication.
echo.
echo Press Ctrl+C to cancel, or any other key to start...
pause >nul

echo.
echo Starting network-accessible server...
echo.

:: You can override the bind address with CHAT_BIND_ADDR environment variable
:: set CHAT_BIND_ADDR=192.168.1.100:8080

cargo run --bin server