use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Message {
    CreateRoom { room_name: String },
    JoinRoom { room_id: String, username: String },
    Chat { content: String },
    RoomCreated { room_name: String, room_id: String },
    JoinedRoom { room_name: String, username: String },
    UserMessage { username: String, content: String },
    Error { message: String },
    Connected,
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
                Message::CreateRoom { room_name } => {
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
                    } else {
                        rooms_guard.insert(room_id_str.clone(), Room {
                            id: room_id,
                            name: room_name.clone(),
                            clients: Vec::new(),
                        });
                        println!("Room '{}' created with ID: {}", room_name, room_id_str);
                        let response = Message::RoomCreated { 
                            room_name,
                            room_id: room_id_str 
                        };
                        send_message(&writer, &response).await?;
                    }
                }
                
                Message::JoinRoom { room_id, username } => {
                    let room_name = {
                        let rooms_guard = rooms.read().await;
                        rooms_guard.get(&room_id).map(|r| r.name.clone())
                    };
                    
                    if let Some(room_name) = room_name {
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
                        
                        // Notify all clients in the room
                        let join_msg = Message::JoinedRoom { 
                            room_name: room_name.clone(), 
                            username: username.clone() 
                        };
                        broadcast_to_room(&clients, &rooms, &room_id, &join_msg, Some(&client_id)).await?;
                        send_message(&writer, &join_msg).await?;
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
                
                _ => {}
            }
        }
    }
    
    // Clean up on disconnect
    let mut clients_guard = clients.lock().await;
    if let Some(client) = clients_guard.remove(&client_id) {
        if let Some(room_id) = client.room {
            let mut rooms_guard = rooms.write().await;
            if let Some(room) = rooms_guard.get_mut(&room_id) {
                room.clients.retain(|id| id != &client_id);
                if room.clients.is_empty() {
                    println!("Room '{}' (ID: {}) is now empty and will be removed", room.name, room_id);
                    rooms_guard.remove(&room_id);
                }
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
