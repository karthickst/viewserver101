use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocksdb::{DB, Options};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use serde::{Deserialize, Serialize};

// Define a struct to represent a view
#[derive(Serialize, Deserialize, Debug)]
struct View {
    name: String,
    filter: String,
}

// Define a struct to represent a client
struct Client {
    id: String,
    views: Vec<View>,
    sender: mpsc::Sender<Message>,
}

// Define the view server
struct ViewServer {
    db: Arc<DB>,
    clients: Arc<RwLock<HashMap<String, Client>>>,
    views: Arc<RwLock<HashMap<String, View>>>,
}

impl ViewServer {
    async fn new(db_path: &str) -> Self {
        let db = Arc::new(DB::open_default(db_path).unwrap());
        let clients = Arc::new(RwLock::new(HashMap::new()));
        let views = Arc::new(RwLock::new(HashMap::new()));

        ViewServer { db, clients, views }
    }

    async fn start(&self) {
        // Start the WebSocket server
        let addr = "127.0.0.1:8080";
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(self.handle_client(stream));
        }
    }

    async fn handle_client(&self, stream: tokio::net::TcpStream) {
        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
        let (sender, mut receiver) = mpsc::channel(100);

        let client_id = uuid::Uuid::new_v4().to_string();
        let client = Client {
            id: client_id.clone(),
            views: Vec::new(),
            sender,
        };

        self.clients.write().await.insert(client_id, client);

        while let Some(message) = receiver.recv().await {
            match message {
                Message::Text(text) => {
                    // Handle client messages
                    self.handle_client_message(&client_id, &text).await;
                }
                Message::Binary(_) => {}
                Message::Close(_) => {
                    // Remove the client when the connection is closed
                    self.clients.write().await.remove(&client_id);
                    break;
                }
                Message::Ping(_) => {}
                Message::Pong(_) => {}
            }
        }
    }

    async fn handle_client_message(&self, client_id: &str, message: &str) {
        // Parse the client message
        let message: serde_json::Value = serde_json::from_str(message).unwrap();

        match message["type"].as_str().unwrap() {
            "create_view" => {
                // Create a new view
                let view = View {
                    name: message["name"].as_str().unwrap().to_string(),
                    filter: message["filter"].as_str().unwrap().to_string(),
                };

                self.views.write().await.insert(view.name.clone(), view.clone());
                self.clients.write().await.get_mut(client_id).unwrap().views.push(view);
            }
            "update_view" => {
                // Update an existing view
                let view_name = message["name"].as_str().unwrap().to_string();
                let view = self.views.write().await.get_mut(&view_name).unwrap();
                view.filter = message["filter"].as_str().unwrap().to_string();
            }
            "delete_view" => {
                // Delete a view
                let view_name = message["name"].as_str().unwrap().to_string();
                self.views.write().await.remove(&view_name);
                self.clients.write().await.get_mut(client_id).unwrap().views.retain(|v| v.name != view_name);
            }
            _ => {}
        }
    }

    async fn notify_clients(&self, view_name: &str) {
        // Notify all clients subscribed to the view
        for client in self.clients.read().await.values() {
            if client.views.iter().any(|v| v.name == view_name) {
                // Send a notification to the client
                client.sender.send(Message::Text(format!("View {} updated", view_name))).await.unwrap();
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let view_server = ViewServer::new("db").await;
    view_server.start().await;
}
