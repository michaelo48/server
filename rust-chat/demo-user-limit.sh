#!/bin/bash
# Example: Demonstrating user limits in chat rooms

echo "=== Rust Chat User Limit Demo ==="
echo
echo "This demo shows how room user limits work."
echo "We'll create a room with a 3-user limit and try to add 4 users."
echo
echo "Instructions:"
echo "1. Create a room with 3-user limit in Client 1"
echo "2. Join with Clients 2 and 3 using the room UUID"
echo "3. Try to join with Client 4 - it should be rejected"
echo
echo "Press Enter to start the demo..."
read

# Start server
echo "Starting server..."
cargo run --bin server &
SERVER_PID=$!
sleep 2

# Start 4 clients
echo "Opening 4 client windows..."
for i in {1..4}; do
    if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
        start cmd //k "echo Client $i && cargo run --bin client"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        osascript -e "tell app \"Terminal\" to do script \"echo Client $i && cd '$(pwd)' && cargo run --bin client\""
    else
        gnome-terminal --title="Client $i" -- bash -c "echo Client $i && cargo run --bin client; read"
    fi
    sleep 1
done

echo
echo "Demo is running. Create a room with a 3-user limit in Client 1."
echo "Then try to join with all 4 clients to see the limit in action."
echo
echo "Press Enter to stop the demo..."
read

# Clean up
kill $SERVER_PID 2>/dev/null
echo "Demo stopped."
