use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("Chat server running on {}", addr);
    
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let rooms: Rooms = Arc::new(RwLock::new(HashMap::new()));
    
    loop {
        let (socket, addr) = listener.accept().await?;
        let client_id = addr.to_string();
        
        let clients_clone = clients.clone();
        let rooms_clone = rooms.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, client_id, clients_clone, rooms_clone).await {
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
    
    // Send connection confirmation
    send_message(&writer, &Message::Connected).await?;
    
    let mut line = String::new();
    
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // Client disconnected
            Ok(_) => {},
            Err(_) => break,
        }
        
        if let Ok(message) = serde_json::from_str::<Message>(&line) {
            match message {
                Message::CreateRoom { room_name, max_users } => {
                    let room_id = Uuid::new_v4();
                    let room_id_str = room_id.to_string();
                    
                    let mut rooms_guard = rooms.write().await;
                    // Check if room name already exists
                    let name_exists = rooms_guard.values().any(|r| r.name == room_name);
                    
                    if name_exists {
                        let error = Message::Error { 
                            message: "Room name already exists".to_string() 
                        };
                        send_message(&writer, &error).await?;
                    } else if max_users < 2 {
                        let error = Message::Error { 
                            message: "Room must allow at least 2 users".to_string() 
                        };
                        send_message(&writer, &error).await?;
                    } else {
                        rooms_guard.insert(room_id_str.clone(), Room {
                            id: room_id,
                            name: room_name.clone(),
                            clients: Vec::new(),
                            max_users,
                        });
                        println!("Room '{}' created with ID: {} (max {} users)", room_name, room_id_str, max_users);
                        let response = Message::RoomCreated { 
                            room_name,
                            room_id: room_id_str,
                            max_users 
                        };
                        send_message(&writer, &response).await?;
                    }
                }
                
                Message::JoinRoom { room_id, username } => {
                    let (room_name, can_join) = {
                        let rooms_guard = rooms.read().await;
                        if let Some(room) = rooms_guard.get(&room_id) {
                            let can_join = room.clients.len() < room.max_users;
                            (Some((room.name.clone(), room.max_users, room.clients.len())), can_join)
                        } else {
                            (None, false)
                        }
                    };
                    
                    if let Some((room_name, max_users, current_users)) = room_name {
                        if !can_join {
                            let error = Message::Error { 
                                message: format!("Room is full ({}/{} users)", current_users, max_users) 
                            };
                            send_message(&writer, &error).await?;
                        } else {
                            // Add client to the room
                            {
                                let mut rooms_guard = rooms.write().await;
                                if let Some(room) = rooms_guard.get_mut(&room_id) {
                                    room.clients.push(client_id.clone());
                                }
                            }
                            
                            // Register client
                            {
                                let mut clients_guard = clients.lock().await;
                                clients_guard.insert(client_id.clone(), Client {
                                    username: username.clone(),
                                    room: Some(room_id.clone()),
                                    socket: writer.clone(),
                                });
                            }
                            
                            // Get current user count for display
                            let current_users = {
                                let rooms_guard = rooms.read().await;
                                rooms_guard.get(&room_id).map(|r| r.clients.len()).unwrap_or(1)
                            };
                            
                            // Notify all clients in the room
                            let join_msg = Message::JoinedRoom { 
                                room_name: room_name.clone(), 
                                username: username.clone() 
                            };
                            broadcast_to_room(&clients, &rooms, &room_id, &join_msg, Some(&client_id)).await?;
                            send_message(&writer, &join_msg).await?;
                            
                            println!("User '{}' joined room '{}' ({}/{} users)", username, room_name, current_users, max_users);
                        }
                    } else {
                        let error = Message::Error { 
                            message: "Invalid room ID".to_string() 
                        };
                        send_message(&writer, &error).await?;
                    }
                }
                
                Message::Chat { content } => {
                    let (username, room_id) = {
                        let clients_guard = clients.lock().await;
                        if let Some(client) = clients_guard.get(&client_id) {
                            if let Some(room) = &client.room {
                                (client.username.clone(), Some(room.clone()))
                            } else {
                                (String::new(), None)
                            }
                        } else {
                            (String::new(), None)
                        }
                    };
                    
                    if let Some(room_id) = room_id {
                        let chat_msg = Message::UserMessage {
                            username,
                            content,
                        };
                        broadcast_to_room(&clients, &rooms, &room_id, &chat_msg, None).await?;
                    }
                }
                
                Message::GetRoomInfo => {
                    let (room_id, username) = {
                        let clients_guard = clients.lock().await;
                        if let Some(client) = clients_guard.get(&client_id) {
                            (client.room.clone(), client.username.clone())
                        } else {
                            (None, String::new())
                        }
                    };
                    
                    if let Some(room_id) = room_id {
                        let rooms_guard = rooms.read().await;
                        if let Some(room) = rooms_guard.get(&room_id) {
                            let mut users = Vec::new();
                            let clients_guard = clients.lock().await;
                            
                            for client_id in &room.clients {
                                if let Some(client) = clients_guard.get(client_id) {
                                    users.push(client.username.clone());
                                }
                            }
                            
                            let room_info = Message::RoomInfo {
                                room_name: room.name.clone(),
                                users,
                                current_count: room.clients.len(),
                                max_users: room.max_users,
                            };
                            
                            send_message(&writer, &room_info).await?;
                        }
                    }
                }
                
                _ => {}
            }
        }
    }
    
    // Clean up on disconnect
    let (room_id_to_notify, username_to_notify) = {
        let mut clients_guard = clients.lock().await;
        if let Some(client) = clients_guard.remove(&client_id) {
            let username = client.username.clone();
            let room = client.room.clone();
            (room, username)
        } else {
            (None, String::new())
        }
    };
    
    if let Some(room_id) = room_id_to_notify {
        // Notify other users that this user left
        if !username_to_notify.is_empty() {
            let leave_msg = Message::UserLeft { username: username_to_notify.clone() };
            let _ = broadcast_to_room(&clients, &rooms, &room_id, &leave_msg, None).await;
        }
        
        // Clean up room
        let mut rooms_guard = rooms.write().await;
        if let Some(room) = rooms_guard.get_mut(&room_id) {
            room.clients.retain(|id| id != &client_id);
            let remaining_users = room.clients.len();
            let room_name = room.name.clone();
            let max_users = room.max_users;
            
            if remaining_users == 0 {
                println!("Room '{}' (ID: {}) is now empty and will be removed", room_name, room_id);
                rooms_guard.remove(&room_id);
            } else {
                println!("User '{}' left room '{}' ({}/{} users remaining)", username_to_notify, room_name, remaining_users, max_users);
            }
        }
    }
    
    Ok(())
}

async fn send_message(
    socket: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>, 
    message: &Message
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
        if let Some(exclude) = exclude_client {
            if client_id == exclude {
                continue;
            }
        }
        
        if let Some(client_room) = &client.room {
            if client_room == room_id {
                let _ = send_message(&client.socket, message).await;
            }
        }
    }
    
    Ok(())
}
