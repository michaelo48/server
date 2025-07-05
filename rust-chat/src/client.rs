use std::io::{self, Write};
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let stream = TcpStream::connect(addr).await?;
    println!("Connected to chat server at {}", addr);
    
    let (tx, rx) = mpsc::channel::<String>(100);
    
    // Split the stream for reading and writing
    let (reader, writer) = stream.into_split();
    let reader = BufReader::new(reader);
    
    // Spawn task to handle incoming messages with room ID tracking
    let (room_id_tx, mut room_id_rx) = mpsc::channel::<String>(1);
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        handle_incoming_messages(reader, tx_clone, room_id_tx).await;
    });
    
    // Spawn task to handle outgoing messages
    tokio::spawn(async move {
        handle_outgoing_messages(writer, rx).await;
    });
    
    // Main menu
    println!("\n=== Welcome to Rust Chat ===");
    println!("1. Create a new chat room");
    println!("2. Join an existing chat room (requires room ID)");
    print!("Enter your choice (1 or 2): ");
    io::stdout().flush()?;
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    match choice.trim() {
        "1" => {
            create_room(tx.clone()).await?;
            // Wait for room creation response with ID
            if let Some(room_id) = room_id_rx.recv().await {
                print!("Enter your username: ");
                io::stdout().flush()?;
                
                let mut username = String::new();
                io::stdin().read_line(&mut username)?;
                let username = username.trim().to_string();
                
                let join_message = Message::JoinRoom { room_id, username };
                tx.send(serde_json::to_string(&join_message)?).await?;
            }
        },
        "2" => join_room(tx.clone()).await?,
        _ => {
            println!("Invalid choice. Exiting...");
            return Ok(());
        }
    }
    
    // Chat loop
    println!("\nYou can now start chatting! Type 'quit' to exit.");
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim() == "quit" {
            break;
        }
        
        let message = Message::Chat {
            content: input.trim().to_string(),
        };
        
        tx.send(serde_json::to_string(&message)?).await?;
    }
    
    Ok(())
}

async fn create_room(tx: mpsc::Sender<String>) -> Result<String, Box<dyn std::error::Error>> {
    print!("Enter room name: ");
    io::stdout().flush()?;
    
    let mut room_name = String::new();
    io::stdin().read_line(&mut room_name)?;
    let room_name = room_name.trim().to_string();
    
    let message = Message::CreateRoom { room_name: room_name.clone() };
    tx.send(serde_json::to_string(&message)?).await?;
    
    // Wait a bit for the response to get the room ID
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    Ok(String::new()) // Room ID will be set by the response handler
}

async fn join_room(tx: mpsc::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
    print!("Enter room ID (UUID): ");
    io::stdout().flush()?;
    
    let mut room_id = String::new();
    io::stdin().read_line(&mut room_id)?;
    let room_id = room_id.trim().to_string();
    
    print!("Enter your username: ");
    io::stdout().flush()?;
    
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();
    
    let message = Message::JoinRoom { room_id, username };
    tx.send(serde_json::to_string(&message)?).await?;
    
    Ok(())
}

async fn handle_incoming_messages(
    mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    _tx: mpsc::Sender<String>,
    room_id_tx: mpsc::Sender<String>,
) {
    let mut line = String::new();
    
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                println!("\nServer disconnected");
                break;
            }
            Ok(_) => {
                if let Ok(message) = serde_json::from_str::<Message>(&line) {
                    match message {
                        Message::Connected => {
                            println!("\nConnected to chat server.");
                        }
                        Message::RoomCreated { room_name, room_id } => {
                            println!("\nRoom '{}' created successfully!", room_name);
                            println!("Room ID: {}", room_id);
                            println!("\nShare this Room ID with others to join your chat.");
                            println!("Keep it safe - you'll need it to rejoin later!\n");
                            let _ = room_id_tx.send(room_id).await;
                        }
                        Message::JoinedRoom { room_name, username } => {
                            println!("\n{} joined the room '{}'", username, room_name);
                        }
                        Message::UserMessage { username, content } => {
                            println!("{}: {}", username, content);
                        }
                        Message::Error { message } => {
                            println!("\nError: {}", message);
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from server: {}", e);
                break;
            }
        }
    }
}

async fn handle_outgoing_messages(
    mut writer: tokio::net::tcp::OwnedWriteHalf,
    mut rx: mpsc::Receiver<String>,
) {
    while let Some(message) = rx.recv().await {
        if let Err(e) = writer.write_all(message.as_bytes()).await {
            eprintln!("Error sending message: {}", e);
            break;
        }
        if let Err(e) = writer.write_all(b"\n").await {
            eprintln!("Error sending newline: {}", e);
            break;
        }
    }
}
