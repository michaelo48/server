use std::io::{self, Write};
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let stream = TcpStream::connect(addr).await?;
    println!("Connected to chat server at {}", addr);
    
    let (tx, rx) = mpsc::channel::<String>(100);
    
    // Split the stream for reading and writing
    let (reader, writer) = stream.into_split();
    let reader = BufReader::new(reader);
    
    // Add channels to signal when to return to menu and track join status
    let (menu_tx, mut menu_rx) = mpsc::channel::<bool>(1);
    let (join_status_tx, mut join_status_rx) = mpsc::channel::<bool>(1);
    
    // Spawn task to handle incoming messages with room ID tracking
    let (room_id_tx, mut room_id_rx) = mpsc::channel::<String>(1);
    let tx_clone = tx.clone();
    let menu_tx_clone = menu_tx.clone();
    let join_status_tx_clone = join_status_tx.clone();
    tokio::spawn(async move {
        handle_incoming_messages(reader, tx_clone, room_id_tx, menu_tx_clone, join_status_tx_clone).await;
    });
    
    // Spawn task to handle outgoing messages
    tokio::spawn(async move {
        handle_outgoing_messages(writer, rx).await;
    });
    
    loop {
        // Main menu
        println!("\n=== Welcome to Rust Chat ===");
        println!("1. Create a new chat room");
        println!("2. Join an existing chat room (requires room ID)");
        println!("3. Exit");
        print!("Enter your choice (1, 2, or 3): ");
        io::stdout().flush()?;
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        
        let mut joined_successfully = false;
        
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
                    
                    // Clear terminal after joining
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    clear_terminal();
                    
                    joined_successfully = true;
                }
            },
            "2" => {
                join_room(tx.clone()).await?;
                
                // Wait for join status response
                tokio::select! {
                    Some(_) = menu_rx.recv() => {
                        // Error occurred, return to menu
                        continue;
                    }
                    Some(_) = join_status_rx.recv() => {
                        // Successfully joined - clear terminal
                        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                        clear_terminal();
                        joined_successfully = true;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
                        // Timeout - assume join failed
                        println!("\nConnection timeout. Returning to menu...");
                        continue;
                    }
                }
            },
            "3" => {
                println!("Goodbye!");
                return Ok(());
            }
            _ => {
                println!("Invalid choice. Please try again.");
                continue;
            }
        }
        
        if joined_successfully {
            // Chat loop
            println!("Welcome to the chat room!");
            println!("Type /help for available commands\n");
            println!("You can now start chatting!");
            
            let mut should_leave = false;
            
            loop {
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();
                
                if input.starts_with('/') {
                    match input {
                        "/help" => show_help(),
                        "/count" => {
                            let msg = Message::GetRoomInfo;
                            tx.send(serde_json::to_string(&msg)?).await?;
                        },
                        "/leave" => {
                            should_leave = true;
                            break;
                        },
                        _ => println!("Unknown command. Type /help for available commands."),
                    }
                } else if !input.is_empty() {
                    let message = Message::Chat {
                        content: input.to_string(),
                    };
                    tx.send(serde_json::to_string(&message)?).await?;
                }
            }
            
            // Clear terminal when leaving
            clear_terminal();
            
            if !should_leave {
                // User typed quit in the old version
                print!("\nReturn to main menu? (y/n): ");
                io::stdout().flush()?;
                let mut continue_choice = String::new();
                io::stdin().read_line(&mut continue_choice)?;
                
                if continue_choice.trim().to_lowercase() != "y" {
                    break;
                }
            }
            // If should_leave is true, we automatically return to menu
        }
    }
    
    Ok(())
}

async fn create_room(tx: mpsc::Sender<String>) -> Result<String, Box<dyn std::error::Error>> {
    print!("Enter room name: ");
    io::stdout().flush()?;
    
    let mut room_name = String::new();
    io::stdin().read_line(&mut room_name)?;
    let room_name = room_name.trim().to_string();
    
    print!("Enter maximum number of users (minimum 2): ");
    io::stdout().flush()?;
    
    let mut max_users_str = String::new();
    io::stdin().read_line(&mut max_users_str)?;
    
    let max_users = match max_users_str.trim().parse::<usize>() {
        Ok(n) if n >= 2 => n,
        Ok(_) => {
            println!("Minimum is 2 users. Setting to 2.");
            2
        },
        Err(_) => {
            println!("Invalid number. Setting to 2 users.");
            2
        }
    };
    
    let message = Message::CreateRoom { room_name: room_name.clone(), max_users };
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
    menu_tx: mpsc::Sender<bool>,
    join_status_tx: mpsc::Sender<bool>,
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
                        Message::RoomCreated { room_name, room_id, max_users } => {
                            println!("\nRoom '{}' created successfully!", room_name);
                            println!("Room ID: {}", room_id);
                            println!("Maximum users: {}", max_users);
                            println!("\nShare this Room ID with others to join your chat.");
                            println!("Keep it safe - you'll need it to rejoin later!\n");
                            let _ = room_id_tx.send(room_id).await;
                        }
                        Message::JoinedRoom { room_name, username } => {
                            println!("\n{} joined the room '{}'", username, room_name);
                            // Signal successful join
                            let _ = join_status_tx.send(true).await;
                        }
                        Message::UserMessage { username, content } => {
                            println!("{}: {}", username, content);
                        }
                        Message::Error { message } => {
                            println!("\nError: {}", message);
                            // If room is full, signal to return to menu
                            if message.contains("Room is full") || message.contains("Invalid room ID") {
                                println!("Returning to main menu...");
                                let _ = menu_tx.send(true).await;
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
