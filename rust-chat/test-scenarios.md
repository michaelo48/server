# Test Scenarios for Rust Chat with User Limits

## Scenario 1: Basic Room Creation with User Limit

1. Start server: `cargo run --bin server`
2. Start client: `cargo run --bin client`
3. Choose option 1 (Create room)
4. Enter room name: "Team Meeting"
5. Enter max users: 4
6. Note the UUID (e.g., `550e8400-e29b-41d4-a716-446655440000`)
7. Enter username: "Alice"

**Expected**: Room created with 4-user limit, Alice joins successfully

## Scenario 2: Joining Within Limit

1. Start second client: `cargo run --bin client`
2. Choose option 2 (Join room)
3. Enter the UUID from Scenario 1
4. Enter username: "Bob"

**Expected**: Bob joins successfully, server shows "2/4 users"

## Scenario 3: Room Full

1. Add two more clients (Charlie and David) following same steps
2. Start fifth client
3. Try to join with same UUID
4. Enter username: "Eve"

**Expected**: 
- Eve receives error "Room is full (4/4 users)"
- "Returning to main menu..."
- Main menu appears again for retry

## Scenario 4: Invalid User Limit

1. Create new room
2. Enter room name: "Small Room"
3. Enter max users: 1

**Expected**: Error "Room must allow at least 2 users", defaults to 2

## Scenario 5: Disconnection Updates

1. In the full room from Scenario 3, have one user type 'quit'

**Expected**: Server shows "User 'X' left room 'Team Meeting' (3/4 users remaining)"

## Scenario 6: Empty Room Cleanup

1. Have all users quit from a room

**Expected**: 
- Server shows "Room 'X' is now empty and will be removed"
- Each user is asked "Return to main menu? (y/n)"

## Scenario 7: Invalid Room ID

1. Start client and choose option 2 (Join)
2. Enter an invalid UUID
3. Enter username

**Expected**:
- Error "Invalid room ID"
- "Returning to main menu..."
- Main menu appears for retry

## Scenario 8: Chat Commands

### 8.1: Help Command
1. Join a room successfully
2. Type `/help`

**Expected**: Display command list

### 8.2: Count Command
1. Join a room with 3 users
2. Type `/count`

**Expected**:
```
=== Room: Team Meeting ===
Users (3/5):
  - Alice
  - Bob
  - Charlie
===============
```

### 8.3: Leave Command
1. Join a room
2. Chat for a while
3. Type `/leave`

**Expected**:
- Terminal clears
- Return to main menu
- Other users see "Alice left the room"

## Scenario 9: Terminal Clearing

1. Create/join a room
2. Observe terminal clears after successful join
3. Type `/leave`
4. Observe terminal clears again

**Expected**: Clean transitions between menu and chat
