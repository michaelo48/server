use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Message {
    CreateRoom { room_name: String, max_users: usize },
    JoinRoom { room_id: String, username: String },
    Chat { content: String },
    RoomCreated { room_name: String, room_id: String, max_users: usize },
    JoinedRoom { room_name: String, username: String },
    UserMessage { username: String, content: String },
    Error { message: String },
    Connected,
    GetRoomInfo,
    RoomInfo { room_name: String, users: Vec<String>, current_count: usize, max_users: usize },
    UserLeft { username: String },
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get bind address from environment variable or use default
    let addr = env::var("CHAT_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    
    let listener = TcpListener::bind(&addr).await?;
    println!("Chat server running on {}", addr);
    
    // Display network information
    println!("\n=== Network Information ===");
    println!("Server is listening on all network interfaces");
    println!("Local connections: 127.0.0.1:8080");
    println!("LAN connections: <your-local-ip>:8080");
    println!("To find your local IP:");
    println!("  Windows: ipconfig");
    println!("  Linux/Mac: ifconfig or ip addr");
    println!("========================\n");
    
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let rooms: Rooms = Arc::new(RwLock::new(HashMap::new()));
    
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);
        
        let client_id = Uuid::new_v4().to_string();
        let clients = Arc::clone(&clients);
        let rooms = Arc::clone(&rooms);
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, client_id, clients, rooms).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    client_id: String,
    clients: Clients,
    rooms: Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, writer) = socket.into_split();
    let writer = Arc::new(Mutex::new(writer));
    let mut reader = BufReader::new(reader);
    
    // Send connected message
    send_message(&writer, &Message::Connected).await?;
    
    // Add client to map
    {
        let mut clients_lock = clients.lock().await;
        clients_lock.insert(client_id.clone(), Client {
            username: String::new(),
            room: None,
            socket: Arc::clone(&writer),
        });
    }
    
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // Connection closed
            Ok(_) => {
                if let Ok(msg) = serde_json::from_str::<Message>(&line) {
                    handle_message(msg, &client_id, &clients, &rooms).await?;
                }
            }
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                break;
            }
        }
    }
    
    // Clean up on disconnect
    disconnect_client(&client_id, &clients, &rooms).await?;
    Ok(())
}

async fn handle_message(
    msg: Message,
    client_id: &str,
    clients: &Clients,
    rooms: &Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    match msg {
        Message::CreateRoom { room_name, max_users } => {
            let max_users = if max_users < 2 {
                println!("Room '{}' requested with {} users, setting to minimum of 2", room_name, max_users);
                2
            } else {
                max_users
            };
            
            let room_id = Uuid::new_v4();
            let mut rooms_lock = rooms.write().await;
            rooms_lock.insert(room_id.to_string(), Room {
                id: room_id,
                name: room_name.clone(),
                clients: Vec::new(),
                max_users,
            });
            
            let clients_lock = clients.lock().await;
            if let Some(client) = clients_lock.get(client_id) {
                send_message(&client.socket, &Message::RoomCreated { 
                    room_name, 
                    room_id: room_id.to_string(),
                    max_users,
                }).await?;
            }
            
            println!("Room created with ID: {} (max {} users)", room_id, max_users);
        }
        
        Message::JoinRoom { room_id, username } => {
            let mut rooms_lock = rooms.write().await;
            if let Some(room) = rooms_lock.get_mut(&room_id) {
                // Check if room is full
                if room.clients.len() >= room.max_users {
                    let clients_lock = clients.lock().await;
                    if let Some(client) = clients_lock.get(client_id) {
                        send_message(&client.socket, &Message::Error { 
                            message: format!("Room is full ({}/{} users)", room.clients.len(), room.max_users)
                        }).await?;
                    }
                    return Ok(());
                }
                
                room.clients.push(client_id.to_string());
                let room_name = room.name.clone();
                let user_count = room.clients.len();
                let max_users = room.max_users;
                
                // Notify all users in room
                for client_in_room in &room.clients {
                    let clients_lock = clients.lock().await;
                    if let Some(client) = clients_lock.get(client_in_room) {
                        send_message(&client.socket, &Message::JoinedRoom {
                            room_name: room_name.clone(),
                            username: username.clone(),
                        }).await?;
                    }
                }
                
                // Update client info
                drop(rooms_lock);
                let mut clients_lock = clients.lock().await;
                if let Some(client) = clients_lock.get_mut(client_id) {
                    client.username = username.clone();
                    client.room = Some(room_id.clone());
                }
                
                println!("User '{}' joined room '{}' ({}/{} users)", username, room_name, user_count, max_users);
            } else {
                let clients_lock = clients.lock().await;
                if let Some(client) = clients_lock.get(client_id) {
                    send_message(&client.socket, &Message::Error { 
                        message: "Invalid room ID".to_string() 
                    }).await?;
                }
            }
        }
        
        Message::Chat { content } => {
            let clients_lock = clients.lock().await;
            if let Some(client) = clients_lock.get(client_id) {
                if let Some(room_id) = &client.room {
                    let username = client.username.clone();
                    drop(clients_lock);
                    
                    broadcast_to_room(room_id, Message::UserMessage {
                        username,
                        content,
                    }, clients, rooms).await?;
                }
            }
        }
        
        Message::GetRoomInfo => {
            let clients_lock = clients.lock().await;
            if let Some(client) = clients_lock.get(client_id) {
                if let Some(room_id) = &client.room {
                    let rooms_lock = rooms.read().await;
                    if let Some(room) = rooms_lock.get(room_id) {
                        let mut users = Vec::new();
                        for client_id in &room.clients {
                            if let Some(c) = clients_lock.get(client_id) {
                                users.push(c.username.clone());
                            }
                        }
                        
                        send_message(&client.socket, &Message::RoomInfo {
                            room_name: room.name.clone(),
                            users,
                            current_count: room.clients.len(),
                            max_users: room.max_users,
                        }).await?;
                    }
                }
            }
        }
        
        _ => {}
    }
    
    Ok(())
}

async fn broadcast_to_room(
    room_id: &str,
    msg: Message,
    clients: &Clients,
    rooms: &Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    let rooms_lock = rooms.read().await;
    if let Some(room) = rooms_lock.get(room_id) {
        let clients_lock = clients.lock().await;
        for client_id in &room.clients {
            if let Some(client) = clients_lock.get(client_id) {
                send_message(&client.socket, &msg).await?;
            }
        }
    }
    Ok(())
}

async fn send_message(
    socket: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    msg: &Message,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = socket.lock().await;
    let json = serde_json::to_string(msg)?;
    socket.write_all(json.as_bytes()).await?;
    socket.write_all(b"\n").await?;
    Ok(())
}

async fn disconnect_client(
    client_id: &str,
    clients: &Clients,
    rooms: &Rooms,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut clients_lock = clients.lock().await;
    if let Some(client) = clients_lock.remove(client_id) {
        let username = client.username.clone();
        if let Some(room_id) = client.room {
            drop(clients_lock);
            
            // Remove from room and notify others
            let mut rooms_lock = rooms.write().await;
            if let Some(room) = rooms_lock.get_mut(&room_id) {
                room.clients.retain(|id| id != client_id);
                let remaining_users = room.clients.len();
                let max_users = room.max_users;
                let room_name = room.name.clone();
                
                if room.clients.is_empty() {
                    rooms_lock.remove(&room_id);
                    println!("Room '{}' is now empty and has been removed", room_name);
                } else {
                    // Notify remaining users
                    drop(rooms_lock);
                    broadcast_to_room(&room_id, Message::UserLeft {
                        username: username.clone(),
                    }, clients, rooms).await?;
                    println!("User '{}' left room '{}' ({}/{} users remaining)", 
                             username, room_name, remaining_users, max_users);
                }
            }
        }
    }
    Ok(())
}