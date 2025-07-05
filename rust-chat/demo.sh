#!/bin/bash

# Example script to demonstrate the chat application

echo "Building the Rust chat application..."
cargo build --release

echo -e "\n=== Rust Chat Demo ===\n"
echo "This will start the chat server and open two client terminals."
echo "You can create a room in one client and join it from the other using the UUID."
echo -e "\nPress Enter to continue or Ctrl+C to cancel..."
read

# Start the server in the background
echo "Starting chat server..."
cargo run --bin server &
SERVER_PID=$!

# Give the server time to start
sleep 2

# Open first client in a new terminal (Windows)
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    start cmd //c "cargo run --bin client && pause"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    osascript -e 'tell app "Terminal" to do script "cd \"'$(pwd)'\" && cargo run --bin client"'
else
    # Linux
    gnome-terminal -- bash -c "cargo run --bin client; read -p 'Press Enter to close...'"
fi

# Give first client time to start
sleep 2

# Open second client in another terminal
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    start cmd //c "cargo run --bin client && pause"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    osascript -e 'tell app "Terminal" to do script "cd \"'$(pwd)'\" && cargo run --bin client"'
else
    # Linux
    gnome-terminal -- bash -c "cargo run --bin client; read -p 'Press Enter to close...'"
fi

echo -e "\nChat server is running. Press Ctrl+C to stop the server."
echo "Try creating a room in one client and joining it from the other using the UUID!"

# Wait for user to stop the server
wait $SERVER_PID
