# User Experience Flow Example

## Scenario: Room Full - Return to Menu

```
=== Welcome to Rust Chat ===
1. Create a new chat room
2. Join an existing chat room (requires room ID)
3. Exit
Enter your choice (1, 2, or 3): 2

Enter room ID (UUID): 550e8400-e29b-41d4-a716-446655440000
Enter your username: Alice

Error: Room is full (3/3 users)
Returning to main menu...

=== Welcome to Rust Chat ===
1. Create a new chat room
2. Join an existing chat room (requires room ID)
3. Exit
Enter your choice (1, 2, or 3): 1

Enter room name: New Meeting Room
Enter maximum number of users (minimum 2): 5

Room 'New Meeting Room' created successfully!
Room ID: 650e8400-e29b-41d4-a716-446655440001
Maximum users: 5

Share this Room ID with others to join your chat.
Keep it safe - you'll need it to rejoin later!

Enter your username: Alice

Alice joined the room 'New Meeting Room'
You can now start chatting! Type 'quit' to exit.
```

## Scenario: After Chatting - Return to Menu

```
You can now start chatting! Type 'quit' to exit.
> Hello everyone!
> How is the project going?
> quit

Return to main menu? (y/n): y

=== Welcome to Rust Chat ===
1. Create a new chat room
2. Join an existing chat room (requires room ID)
3. Exit
Enter your choice (1, 2, or 3): 3
Goodbye!
```

## Benefits

1. **User-Friendly**: No need to restart the application after errors
2. **Efficient**: Quick retry when rooms are full
3. **Flexible**: Users can create a new room if their desired room is full
4. **Persistent**: Stay in the app for multiple chat sessions
