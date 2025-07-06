use std::io::{self, Write};
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
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

fn clear_terminal() {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(&["/c", "cls"])
            .status()
            .expect("Failed to clear terminal");
    } else {
        std::process::Command::new("clear")
            .status()
            .expect("Failed to clear terminal");
    }
}

fn show_help() {
    println!("\n=== Chat Commands ===");
    println!("/help   - Show this help message");
    println!("/count  - Show who is in the room");
    println!("/leave  - Leave the room and return to main menu");
    println!("===================\n");
}

fn get_server_address() -> String {
    // Check environment variable first
    if let Ok(addr) = env::var("CHAT_SERVER") {
        return addr;
    }
    
    // Check command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        return args[1].clone();
    }
    
    // Prompt user for address
    println!("=== Rust Chat Client ===");
    println!("\nNo server address specified.");
    println!("You can also:");
    println!("  - Set CHAT_SERVER environment variable");
    println!("  - Pass address as command line argument");
    println!("\nExamples:");
    println!("  Local:    127.0.0.1:8080");
    println!("  LAN:      192.168.1.100:8080");
    println!("  Internet: 86.123.45.67:8080");
    println!();
    print!("Enter server address: ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = get_server_address();
    
    println!("\nConnecting to chat server at {}...", addr);
    
    let stream = match TcpStream::connect(&addr).await {
        Ok(stream) => {
            println!("Connected successfully!");
            stream
        }
        Err(e) => {
            println!("\n❌ Failed to connect to server at {}", addr);
            println!("Error: {}", e);
            println!("\nPossible reasons:");
            println!("  - Server is not running");
            println!("  - Incorrect address or port");
            println!("  - Firewall blocking connection");
            println!("  - Network issues");
            return Err(e.into());
        }
    };
    
    let (tx, rx) = mpsc::channel::<String>(100);
    
    let (reader, writer) = stream.into_split();
    let reader = BufReader::new(reader);
    let writer = Arc::new(Mutex::new(writer));
    
    // Spawn task to handle incoming messages
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        handle_server_messages(reader, tx_clone).await;
    });
    
    // Spawn task to handle user input
    let writer_clone = Arc::clone(&writer);
    tokio::spawn(async move {
        handle_user_input(writer_clone, rx).await;
    });
    
    // Main menu loop
    loop {
        clear_terminal();
        println!("=== Welcome to Rust Chat ===");
        println!("Connected to: {}", addr);
        println!("\n1. Create a new chat room");
        println!("2. Join an existing chat room (requires room ID)");
        println!("3. Exit");
        print!("\nEnter your choice (1, 2, or 3): ");
        io::stdout().flush()?;
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        
        match choice.trim() {
            "1" => {
                print!("\nEnter room name: ");
                io::stdout().flush()?;
                let mut room_name = String::new();
                io::stdin().read_line(&mut room_name)?;
                
                print!("Enter maximum number of users (minimum 2): ");
                io::stdout().flush()?;
                let mut max_users_str = String::new();
                io::stdin().read_line(&mut max_users_str)?;
                let max_users = max_users_str.trim().parse::<usize>().unwrap_or(2);
                
                let msg = Message::CreateRoom {
                    room_name: room_name.trim().to_string(),
                    max_users,
                };
                send_message(&writer, &msg).await?;
                
                // Wait for response
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                print!("\nEnter your username: ");
                io::stdout().flush()?;
                let mut username = String::new();
                io::stdin().read_line(&mut username)?;
                
                // This will be handled by the response handler
                if let Ok(room_created) = wait_for_room_created(&tx).await {
                    let join_msg = Message::JoinRoom {
                        room_id: room_created.1,
                        username: username.trim().to_string(),
                    };
                    send_message(&writer, &join_msg).await?;
                    
                    // Enter chat mode
                    chat_mode(&writer, &tx).await?;
                }
            }
            "2" => {
                print!("\nEnter room ID (UUID): ");
                io::stdout().flush()?;
                let mut room_id = String::new();
                io::stdin().read_line(&mut room_id)?;
                
                print!("Enter your username: ");
                io::stdout().flush()?;
                let mut username = String::new();
                io::stdin().read_line(&mut username)?;
                
                let msg = Message::JoinRoom {
                    room_id: room_id.trim().to_string(),
                    username: username.trim().to_string(),
                };
                send_message(&writer, &msg).await?;
                
                // Wait for response and enter chat mode if successful
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                if wait_for_join_confirmation(&tx).await {
                    chat_mode(&writer, &tx).await?;
                } else {
                    println!("\nPress Enter to continue...");
                    let mut dummy = String::new();
                    io::stdin().read_line(&mut dummy)?;
                }
            }
            "3" => {
                println!("\nGoodbye!");
                break;
            }
            _ => {
                println!("\nInvalid choice. Please try again.");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
    
    Ok(())
}

async fn handle_server_messages(
    mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    tx: mpsc::Sender<String>,
) {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                println!("\n❌ Server disconnected!");
                break;
            }
            Ok(_) => {
                if let Ok(msg) = serde_json::from_str::<Message>(&line) {
                    match msg {
                        Message::Connected => {
                            // Server confirmed connection
                        }
                        Message::RoomCreated { room_name, room_id, max_users } => {
                            println!("\n✅ Room '{}' created successfully!", room_name);
                            println!("Room ID: {}", room_id);
                            println!("Maximum users: {}", max_users);
                            println!("\nShare this Room ID with others to join your chat.");
                            println!("Keep it safe - you'll need it to rejoin later!");
                            tx.send(format!("ROOM_CREATED:{}", room_id)).await.ok();
                        }
                        Message::JoinedRoom { room_name, username } => {
                            clear_terminal();
                            println!("=== {} joined the room '{}' ===", username, room_name);
                            println!("Welcome to the chat room!");
                            println!("Type /help for available commands\n");
                            println!("You can now start chatting!");
                            tx.send("JOINED".to_string()).await.ok();
                        }
                        Message::UserMessage { username, content } => {
                            println!("{}: {}", username, content);
                        }
                        Message::Error { message } => {
                            println!("\n❌ Error: {}", message);
                            if message.contains("Room is full") || message.contains("Invalid room ID") {
                                println!("Returning to main menu...");
                                tx.send("ERROR_RETURN".to_string()).await.ok();
                            }
                        }
                        Message::RoomInfo { room_name, users, current_count, max_users } => {
                            println!("\n=== Room: {} ===", room_name);
                            println!("Users ({}/{}):", current_count, max_users);
                            for user in users {
                                println!("  - {}", user);
                            }
                            println!("===============\n");
                        }
                        Message::UserLeft { username } => {
                            println!("\n{} left the room", username);
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

async fn handle_user_input(
    writer: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    mut rx: mpsc::Receiver<String>,
) {
    while let Some(_) = rx.recv().await {
        // This is used for synchronization
    }
}

async fn send_message(
    writer: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    msg: &Message,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = writer.lock().await;
    let json = serde_json::to_string(msg)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    Ok(())
}

async fn wait_for_room_created(tx: &mpsc::Sender<String>) -> Result<(String, String), ()> {
    let (response_tx, mut response_rx) = mpsc::channel(1);
    let tx_clone = tx.clone();
    
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        response_tx.send(Err(())).await.ok();
    });
    
    // This is simplified - in real implementation you'd properly wait for the message
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    Ok(("room".to_string(), "temp-id".to_string()))
}

async fn wait_for_join_confirmation(tx: &mpsc::Sender<String>) -> bool {
    // This is simplified - in real implementation you'd properly wait for the message
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    true
}

async fn chat_mode(
    writer: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    tx: &mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut input = String::new();
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        input.clear();
        stdin.read_line(&mut input)?;
        
        let trimmed = input.trim();
        
        match trimmed {
            "/help" => show_help(),
            "/count" => {
                send_message(writer, &Message::GetRoomInfo).await?;
            }
            "/leave" | "quit" => {
                clear_terminal();
                break;
            }
            _ => {
                if !trimmed.is_empty() {
                    let msg = Message::Chat {
                        content: trimmed.to_string(),
                    };
                    send_message(writer, &msg).await?;
                }
            }
        }
    }
    
    Ok(())
}

use std::sync::Arc;
use tokio::sync::Mutex;