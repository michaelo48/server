# Rust Terminal Chat Application

A secure, real-time chat application built in Rust that runs in the terminal. Users can create private chat rooms protected by UUIDs and communicate with others in real-time.

## Features

- Create private chat rooms with unique UUIDs
- Join existing chat rooms using their UUID
- Real-time message broadcasting
- Multiple concurrent chat rooms
- Clean disconnection handling
- Simple terminal-based UI
- Hidden room system - rooms are not discoverable without the UUID

## Requirements

- Rust 1.70+ (for async/await support)
- Cargo

## Building the Project

1. Navigate to the project directory:
```bash
cd rust-chat
```

2. Build the project:
```bash
cargo build --release
```

## Running the Application

### Start the Server

First, start the chat server:

```bash
cargo run --bin server
```

The server will start listening on `127.0.0.1:8080`.

### Connect Clients

In separate terminal windows, start client instances:

```bash
cargo run --bin client
```

Each client will:
1. Connect to the server
2. Present options to:
   - Press 1 to create a new chat room
   - Press 2 to join an existing chat room (requires UUID)

## Usage

### Creating a Room
1. Select option 1
2. Enter a name for your room (for display purposes)
3. You'll receive a unique UUID for the room
4. Share this UUID with people you want to invite
5. Enter your username to join

### Joining a Room
1. Select option 2
2. Enter the UUID of the room you want to join
3. Enter your username

### Chatting
- Once in a room, type messages and press Enter to send
- Messages from other users will appear automatically
- Type 'quit' to exit the chat

## Security Features

- **Private Rooms**: Rooms are not listed or discoverable. You need the exact UUID to join.
- **UUID Protection**: Each room is protected by a cryptographically secure UUID v4.
- **No Room Discovery**: The server doesn't provide any way to list or discover existing rooms.

## Architecture

- **Server**: Handles multiple client connections using Tokio async runtime
- **Client**: Manages user input and server communication concurrently
- **Protocol**: JSON-based message passing over TCP sockets
- **Room Management**: UUID-based room identification and access control

## Message Types

- `CreateRoom`: Request to create a new chat room
- `JoinRoom`: Request to join a room by UUID
- `Chat`: Send a message to the room
- `RoomCreated`: Confirmation with room name and UUID
- `JoinedRoom`: Notification when someone joins
- `UserMessage`: Broadcast message from a user
- `Error`: Error notifications
- `Connected`: Server connection confirmation

## Example Usage

```
=== Welcome to Rust Chat ===
1. Create a new chat room
2. Join an existing chat room (requires room ID)
Enter your choice (1 or 2): 1

Enter room name: Secret Meeting

Room 'Secret Meeting' created successfully!
Room ID: 550e8400-e29b-41d4-a716-446655440000

Share this Room ID with others to join your chat.
Keep it safe - you'll need it to rejoin later!

Enter your username: Alice

Alice joined the room 'Secret Meeting'
You can now start chatting! Type 'quit' to exit.
```

## Notes

- The server automatically removes empty rooms
- Clients are notified when users join their room
- Disconnected clients are automatically removed from rooms
- Room UUIDs are generated using the UUID v4 standard for maximum randomness
