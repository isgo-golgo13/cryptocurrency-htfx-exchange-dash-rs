//! WebSocket handler for client connections

use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;

use crate::AppState;
use dash_core::WsMessage;

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();

    tracing::info!("New WebSocket client connected");

    // Spawn task to forward broadcast messages to client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to serialize message: {}", e);
                }
            }
        }
    });

    // Spawn task to handle incoming messages from client
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Handle client messages (e.g., subscription requests)
                    handle_client_message(&text).await;
                }
                Message::Ping(data) => {
                    tracing::trace!("Received ping");
                    // Pong is sent automatically by axum
                }
                Message::Close(_) => {
                    tracing::info!("Client initiated close");
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {
            tracing::info!("Send task completed");
        }
        _ = recv_task => {
            tracing::info!("Receive task completed");
        }
    }

    tracing::info!("WebSocket client disconnected");
}

/// Handle messages from client
async fn handle_client_message(text: &str) {
    // Parse client commands (e.g., subscribe to specific symbols)
    #[derive(serde::Deserialize)]
    #[serde(tag = "type")]
    enum ClientMessage {
        #[serde(rename = "subscribe")]
        Subscribe { symbol: String },
        #[serde(rename = "unsubscribe")]
        Unsubscribe { symbol: String },
        #[serde(rename = "ping")]
        Ping,
    }

    match serde_json::from_str::<ClientMessage>(text) {
        Ok(ClientMessage::Subscribe { symbol }) => {
            tracing::info!("Client subscribed to {}", symbol);
            // TODO: Implement subscription filtering
        }
        Ok(ClientMessage::Unsubscribe { symbol }) => {
            tracing::info!("Client unsubscribed from {}", symbol);
        }
        Ok(ClientMessage::Ping) => {
            tracing::trace!("Client ping");
        }
        Err(_) => {
            tracing::trace!("Unknown client message: {}", text);
        }
    }
}

/// Broadcast a message to all connected clients
pub async fn broadcast(tx: &broadcast::Sender<WsMessage>, msg: WsMessage) {
    // Ignore send errors (no receivers)
    let _ = tx.send(msg);
}
