use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

// ============================================================================
// Message Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Message {
    // Client -> Server
    CreateRoom { room_name: String, max_users: usize },
    JoinRoom { room_id: String, username: String },
    Chat { content: String },
    GetRoomInfo,

    // Server -> Client
    Connected,
    RoomCreated { room_name: String, room_id: String, max_users: usize },
    JoinedRoom { room_name: String, username: String },
    UserMessage { username: String, content: String },
    RoomInfo { room_name: String, users: Vec<String>, current_count: usize, max_users: usize },
    UserLeft { username: String },
    Error { message: String },
}

// ============================================================================
// Data Structures
// ============================================================================

type Clients = Arc<Mutex<HashMap<String, Client>>>;
type Rooms = Arc<RwLock<HashMap<String, Room>>>;

struct Client {
    username: String,
    room: Option<String>,
    socket: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
}

struct Room {
    id: Uuid,
    name: String,
    clients: Vec<String>,
    max_users: usize,
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("Chat server running on {}", addr);

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let rooms: Rooms = Arc::new(RwLock::new(HashMap::new()));

    loop {
        let (socket, addr) = listener.accept().await?;
        let client_id = addr.to_string();

        let clients = Arc::clone(&clients);
        let rooms = Arc::clone(&rooms);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, client_id, clients, rooms).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

// ============================================================================
// Client Handler
// ============================================================================

async fn handle_client(
    socket: TcpStream,
    client_id: String,
    clients: Clients,
    rooms: Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, writer) = socket.into_split();
    let writer = Arc::new(Mutex::new(writer));
    let mut reader = BufReader::new(reader);

    send_message(&writer, &Message::Connected).await?;

    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }

        if let Ok(message) = serde_json::from_str::<Message>(&line) {
            handle_message(&message, &client_id, &writer, &clients, &rooms).await?;
        }
    }

    handle_disconnect(&client_id, &clients, &rooms).await?;
    Ok(())
}

// ============================================================================
// Message Handling
// ============================================================================

async fn handle_message(
    message: &Message,
    client_id: &str,
    writer: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    clients: &Clients,
    rooms: &Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    match message {
        Message::CreateRoom { room_name, max_users } => {
            handle_create_room(room_name, *max_users, writer, rooms).await?;
        }
        Message::JoinRoom { room_id, username } => {
            handle_join_room(room_id, username, client_id, writer, clients, rooms).await?;
        }
        Message::Chat { content } => {
            handle_chat(content, client_id, clients, rooms).await?;
        }
        Message::GetRoomInfo => {
            handle_get_room_info(client_id, writer, clients, rooms).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn handle_create_room(
    room_name: &str,
    max_users: usize,
    writer: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    rooms: &Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rooms_guard = rooms.write().await;

    // Check if room name already exists
    let name_exists = rooms_guard.values().any(|r| r.name == room_name);

    if name_exists {
        send_message(writer, &Message::Error {
            message: "Room name already exists".to_string(),
        }).await?;
        return Ok(());
    }

    if max_users < 2 {
        send_message(writer, &Message::Error {
            message: "Room must allow at least 2 users".to_string(),
        }).await?;
        return Ok(());
    }

    let room_id = Uuid::new_v4();
    let room_id_str = room_id.to_string();

    rooms_guard.insert(room_id_str.clone(), Room {
        id: room_id,
        name: room_name.to_string(),
        clients: Vec::new(),
        max_users,
    });

    println!("Room '{}' created with ID: {} (max {} users)", room_name, room_id_str, max_users);

    send_message(writer, &Message::RoomCreated {
        room_name: room_name.to_string(),
        room_id: room_id_str,
        max_users,
    }).await?;

    Ok(())
}

async fn handle_join_room(
    room_id: &str,
    username: &str,
    client_id: &str,
    writer: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    clients: &Clients,
    rooms: &Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check room exists and has space
    let room_info = {
        let rooms_guard = rooms.read().await;
        rooms_guard.get(room_id).map(|room| {
            (room.name.clone(), room.max_users, room.clients.len(), room.clients.len() < room.max_users)
        })
    };

    let Some((room_name, max_users, current_users, can_join)) = room_info else {
        send_message(writer, &Message::Error {
            message: "Invalid room ID".to_string(),
        }).await?;
        return Ok(());
    };

    if !can_join {
        send_message(writer, &Message::Error {
            message: format!("Room is full ({}/{} users)", current_users, max_users),
        }).await?;
        return Ok(());
    }

    // Add client to room
    {
        let mut rooms_guard = rooms.write().await;
        if let Some(room) = rooms_guard.get_mut(room_id) {
            room.clients.push(client_id.to_string());
        }
    }

    // Register client
    {
        let mut clients_guard = clients.lock().await;
        clients_guard.insert(client_id.to_string(), Client {
            username: username.to_string(),
            room: Some(room_id.to_string()),
            socket: Arc::clone(writer),
        });
    }

    // Get updated user count
    let user_count = {
        let rooms_guard = rooms.read().await;
        rooms_guard.get(room_id).map(|r| r.clients.len()).unwrap_or(1)
    };

    // Notify all clients in the room
    let join_msg = Message::JoinedRoom {
        room_name: room_name.clone(),
        username: username.to_string(),
    };

    broadcast_to_room(clients, rooms, room_id, &join_msg, Some(client_id)).await?;
    send_message(writer, &join_msg).await?;

    println!("User '{}' joined room '{}' ({}/{} users)", username, room_name, user_count, max_users);

    Ok(())
}

async fn handle_chat(
    content: &str,
    client_id: &str,
    clients: &Clients,
    rooms: &Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    let (username, room_id) = {
        let clients_guard = clients.lock().await;
        clients_guard.get(client_id).map(|client| {
            (client.username.clone(), client.room.clone())
        }).unwrap_or_default()
    };

    if let Some(room_id) = room_id {
        let chat_msg = Message::UserMessage {
            username,
            content: content.to_string(),
        };
        broadcast_to_room(clients, rooms, &room_id, &chat_msg, None).await?;
    }

    Ok(())
}

async fn handle_get_room_info(
    client_id: &str,
    writer: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    clients: &Clients,
    rooms: &Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    let room_id = {
        let clients_guard = clients.lock().await;
        clients_guard.get(client_id).and_then(|c| c.room.clone())
    };

    let Some(room_id) = room_id else {
        return Ok(());
    };

    let rooms_guard = rooms.read().await;
    let Some(room) = rooms_guard.get(&room_id) else {
        return Ok(());
    };

    let users: Vec<String> = {
        let clients_guard = clients.lock().await;
        room.clients.iter()
            .filter_map(|id| clients_guard.get(id).map(|c| c.username.clone()))
            .collect()
    };

    send_message(writer, &Message::RoomInfo {
        room_name: room.name.clone(),
        users,
        current_count: room.clients.len(),
        max_users: room.max_users,
    }).await?;

    Ok(())
}

// ============================================================================
// Disconnect Handling
// ============================================================================

async fn handle_disconnect(
    client_id: &str,
    clients: &Clients,
    rooms: &Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    let (room_id, username) = {
        let mut clients_guard = clients.lock().await;
        clients_guard.remove(client_id).map(|client| {
            (client.room, client.username)
        }).unwrap_or_default()
    };

    let Some(room_id) = room_id else {
        return Ok(());
    };

    // Notify other users
    if !username.is_empty() {
        let leave_msg = Message::UserLeft { username: username.clone() };
        let _ = broadcast_to_room(clients, rooms, &room_id, &leave_msg, None).await;
    }

    // Clean up room
    let mut rooms_guard = rooms.write().await;
    if let Some(room) = rooms_guard.get_mut(&room_id) {
        room.clients.retain(|id| id != client_id);

        let remaining = room.clients.len();
        let room_name = room.name.clone();
        let max_users = room.max_users;

        if remaining == 0 {
            println!("Room '{}' (ID: {}) is now empty and will be removed", room_name, room_id);
            rooms_guard.remove(&room_id);
        } else {
            println!("User '{}' left room '{}' ({}/{} users remaining)", username, room_name, remaining, max_users);
        }
    }

    Ok(())
}

// ============================================================================
// Utility Functions
// ============================================================================

async fn send_message(
    socket: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    message: &Message,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(message)?;
    let mut socket_guard = socket.lock().await;
    socket_guard.write_all(json.as_bytes()).await?;
    socket_guard.write_all(b"\n").await?;
    Ok(())
}

async fn broadcast_to_room(
    clients: &Clients,
    rooms: &Rooms,
    room_id: &str,
    message: &Message,
    exclude_client: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let clients_guard = clients.lock().await;

    for (client_id, client) in clients_guard.iter() {
        if exclude_client.map_or(false, |id| client_id == id) {
            continue;
        }

        if client.room.as_deref() == Some(room_id) {
            let _ = send_message(&client.socket, message).await;
        }
    }

    Ok(())
}
