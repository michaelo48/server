@echo off
echo === Rust Chat Network Client ===
echo.
echo Connect to a chat server on any network!
echo.
echo Connection examples:
echo   Local server:     127.0.0.1:8080
echo   LAN server:       192.168.1.100:8080
echo   Internet server:  86.123.45.67:8080
echo   Using ngrok:      2.tcp.ngrok.io:12345
echo.

if "%1"=="" (
    echo You can also connect by:
    echo   - Running: connect-to-server.bat SERVER_ADDRESS
    echo   - Setting CHAT_SERVER environment variable
    echo.
    cargo run --bin client
) else (
    echo Connecting to: %1
    set CHAT_SERVER=%1
    cargo run --bin client
)