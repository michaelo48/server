@echo off
echo Building the Rust chat application...
cargo build --release

echo.
echo === Rust Chat Demo ===
echo.
echo This will start the chat server and open two client terminals.
echo You can create a room in one client and join it from the other using the UUID.
echo.
echo Press any key to continue or Ctrl+C to cancel...
pause >nul

echo Starting chat server...
start /B cargo run --bin server

timeout /t 2 /nobreak >nul

echo Opening first client...
start cmd /k cargo run --bin client

timeout /t 2 /nobreak >nul

echo Opening second client...
start cmd /k cargo run --bin client

echo.
echo Chat server is running in the background.
echo Try creating a room in one client and joining it from the other using the UUID!
echo.
echo Press any key to stop the demo and close all windows...
pause >nul

taskkill /F /IM server.exe >nul 2>&1
echo Demo stopped.
