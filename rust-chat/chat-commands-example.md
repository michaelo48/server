# Chat Commands Example

## Using /help Command

```
Welcome to the chat room!
Type /help for available commands

You can now start chatting!
> /help

=== Chat Commands ===
/help   - Show this help message
/count  - Show who is in the room
/leave  - Leave the room and return to main menu
===================
```

## Using /count Command

```
> /count

=== Room: Secret Meeting ===
Users (3/5):
  - Alice
  - Bob
  - Charlie
===============
```

## Chat Session with Commands

```
Welcome to the chat room!
Type /help for available commands

You can now start chatting!
> Hello everyone!
Alice: Hello everyone!
Bob: Hi Alice!
> /count

=== Room: Team Meeting ===
Users (2/4):
  - Alice
  - Bob
===============

Charlie joined the room 'Team Meeting'
> Welcome Charlie!
Alice: Welcome Charlie!
Charlie: Thanks! Good to be here
> /count

=== Room: Team Meeting ===
Users (3/4):
  - Alice
  - Bob
  - Charlie
===============

> Let's discuss the project
Alice: Let's discuss the project
Bob: Sure, I have some updates

David left the room
> /count

=== Room: Team Meeting ===
Users (2/4):
  - Alice
  - Charlie
===============

> /leave
[Terminal clears]

=== Welcome to Rust Chat ===
1. Create a new chat room
2. Join an existing chat room (requires room ID)
3. Exit
Enter your choice (1, 2, or 3): 
```

## Benefits of Commands

1. **No need to remember 'quit'** - Use intuitive `/leave` command
2. **Room awareness** - See who's in the room anytime with `/count`
3. **Help always available** - `/help` shows commands whenever needed
4. **Clean experience** - Terminal clears when leaving for privacy
5. **User notifications** - See when people join or leave
