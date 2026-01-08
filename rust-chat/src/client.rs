use std::io::{self, Write};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

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
// Terminal Utilities
// ============================================================================

fn clear_terminal() {
    if cfg!(target_os = "windows") {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "cls"])
            .status();
    } else {
        let _ = std::process::Command::new("clear").status();
    }
}

fn show_help() {
    println!("\n=== Chat Commands ===");
    println!("/help   - Show this help message");
    println!("/count  - Show who is in the room");
    println!("/leave  - Leave the room and return to main menu");
    println!("=====================\n");
}

fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_default();
    input.trim().to_string()
}

fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap_or_default();
    read_line()
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let stream = TcpStream::connect(addr).await?;
    println!("Connected to chat server at {}", addr);

    let (tx, rx) = mpsc::channel::<String>(100);
    let (reader, writer) = stream.into_split();
    let reader = BufReader::new(reader);

    // Channels for coordination
    let (menu_tx, mut menu_rx) = mpsc::channel::<bool>(1);
    let (join_tx, mut join_rx) = mpsc::channel::<bool>(1);
    let (room_id_tx, mut room_id_rx) = mpsc::channel::<String>(1);

    // Spawn incoming message handler
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        handle_incoming(reader, tx_clone, room_id_tx, menu_tx, join_tx).await;
    });

    // Spawn outgoing message handler
    tokio::spawn(async move {
        handle_outgoing(writer, rx).await;
    });

    // Main menu loop
    loop {
        println!("\n=== Welcome to Rust Chat ===");
        println!("1. Create a new chat room");
        println!("2. Join an existing chat room (requires room ID)");
        println!("3. Exit");

        let choice = prompt("Enter your choice (1, 2, or 3): ");

        match choice.as_str() {
            "1" => {
                if let Some(room_id) = create_room(&tx, &mut room_id_rx).await? {
                    let username = prompt("Enter your username: ");
                    join_room_by_id(&tx, &room_id, &username).await?;
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    clear_terminal();
                    chat_loop(&tx).await?;
                }
            }
            "2" => {
                let room_id = prompt("Enter room ID (UUID): ");
                let username = prompt("Enter your username: ");
                join_room_by_id(&tx, &room_id, &username).await?;

                // Wait for join result
                tokio::select! {
                    Some(_) = menu_rx.recv() => continue,
                    Some(_) = join_rx.recv() => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                        clear_terminal();
                        chat_loop(&tx).await?;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
                        println!("\nConnection timeout. Returning to menu...");
                        continue;
                    }
                }
            }
            "3" => {
                println!("Goodbye!");
                return Ok(());
            }
            _ => {
                println!("Invalid choice. Please try again.");
            }
        }

        clear_terminal();
    }
}

// ============================================================================
// Room Operations
// ============================================================================

async fn create_room(
    tx: &mpsc::Sender<String>,
    room_id_rx: &mut mpsc::Receiver<String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let room_name = prompt("Enter room name: ");

    let max_users_str = prompt("Enter maximum number of users (minimum 2): ");
    let max_users = max_users_str.parse::<usize>().unwrap_or(2).max(2);

    if max_users < max_users_str.parse::<usize>().unwrap_or(0) {
        println!("Minimum is 2 users. Setting to 2.");
    }

    let message = Message::CreateRoom { room_name, max_users };
    tx.send(serde_json::to_string(&message)?).await?;

    // Wait for room ID response
    tokio::select! {
        Some(room_id) = room_id_rx.recv() => Ok(Some(room_id)),
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
            println!("Timeout waiting for room creation.");
            Ok(None)
        }
    }
}

async fn join_room_by_id(
    tx: &mpsc::Sender<String>,
    room_id: &str,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = Message::JoinRoom {
        room_id: room_id.to_string(),
        username: username.to_string(),
    };
    tx.send(serde_json::to_string(&message)?).await?;
    Ok(())
}

// ============================================================================
// Chat Loop
// ============================================================================

async fn chat_loop(tx: &mpsc::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome to the chat room!");
    println!("Type /help for available commands\n");

    loop {
        let input = read_line();

        if input.starts_with('/') {
            match input.as_str() {
                "/help" => show_help(),
                "/count" => {
                    let msg = Message::GetRoomInfo;
                    tx.send(serde_json::to_string(&msg)?).await?;
                }
                "/leave" => break,
                _ => println!("Unknown command. Type /help for available commands."),
            }
        } else if !input.is_empty() {
            let message = Message::Chat { content: input };
            tx.send(serde_json::to_string(&message)?).await?;
        }
    }

    Ok(())
}

// ============================================================================
// Message Handlers
// ============================================================================

async fn handle_incoming(
    mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    _tx: mpsc::Sender<String>,
    room_id_tx: mpsc::Sender<String>,
    menu_tx: mpsc::Sender<bool>,
    join_tx: mpsc::Sender<bool>,
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
                    process_server_message(message, &room_id_tx, &menu_tx, &join_tx).await;
                }
            }
            Err(e) => {
                eprintln!("Error reading from server: {}", e);
                break;
            }
        }
    }
}

async fn process_server_message(
    message: Message,
    room_id_tx: &mpsc::Sender<String>,
    menu_tx: &mpsc::Sender<bool>,
    join_tx: &mpsc::Sender<bool>,
) {
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
            let _ = join_tx.send(true).await;
        }
        Message::UserMessage { username, content } => {
            println!("{}: {}", username, content);
        }
        Message::Error { message } => {
            println!("\nError: {}", message);
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
            println!("=================\n");
        }
        Message::UserLeft { username } => {
            println!("\n{} left the room", username);
        }
        _ => {}
    }
}

async fn handle_outgoing(
    mut writer: tokio::net::tcp::OwnedWriteHalf,
    mut rx: mpsc::Receiver<String>,
) {
    while let Some(message) = rx.recv().await {
        if writer.write_all(message.as_bytes()).await.is_err() {
            break;
        }
        if writer.write_all(b"\n").await.is_err() {
            break;
        }
    }
}
