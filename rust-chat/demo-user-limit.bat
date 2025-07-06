@echo off
echo === Rust Chat User Limit Demo ===
echo.
echo This demo shows how room user limits work.
echo We'll create a room with a 3-user limit and try to add 4 users.
echo.
echo Instructions:
echo 1. Create a room with 3-user limit in Client 1
echo 2. Join with Clients 2 and 3 using the room UUID
echo 3. Try to join with Client 4 - it should be rejected
echo.
echo Press any key to start the demo...
pause >nul

echo Starting server...
start /B cargo run --bin server
timeout /t 2 /nobreak >nul

echo Opening 4 client windows...
start "Client 1" cmd /k "echo Client 1 && cargo run --bin client"
timeout /t 1 /nobreak >nul
start "Client 2" cmd /k "echo Client 2 && cargo run --bin client"
timeout /t 1 /nobreak >nul
start "Client 3" cmd /k "echo Client 3 && cargo run --bin client"
timeout /t 1 /nobreak >nul
start "Client 4" cmd /k "echo Client 4 && cargo run --bin client"

echo.
echo Demo is running. Create a room with a 3-user limit in Client 1.
echo Then try to join with all 4 clients to see the limit in action.
echo.
echo Press any key to stop the demo...
pause >nul

taskkill /F /IM server.exe >nul 2>&1
echo Demo stopped.
