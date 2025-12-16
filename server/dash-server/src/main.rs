//! BTC Exchange Dashboard - WebSocket Server
//!
//! Axum-based server providing:
//! - WebSocket endpoint for real-time market data
//! - Static file serving for the WASM frontend
//! - Mock data engine for demo mode

mod mock;
mod ws;

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use dash_core::WsMessage;

/// Shared application state
pub struct AppState {
    /// Broadcast channel for market data
    pub tx: broadcast::Sender<WsMessage>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "dash_server=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create shared state
    let state = Arc::new(AppState::new());

    // Start mock data engine
    let mock_tx = state.tx.clone();
    tokio::spawn(async move {
        mock::run_mock_engine(mock_tx).await;
    });

    // Build router
    let app = Router::new()
        // WebSocket endpoint
        .route("/ws", get(ws::ws_handler))
        // Health check
        .route("/health", get(|| async { "OK" }))
        // Static files (WASM frontend)
        .fallback_service(ServeDir::new("dist").append_index_html_on_directories(true))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("🚀 Server starting on http://{}", addr);
    tracing::info!("   WebSocket: ws://{}/ws", addr);
    tracing::info!("   Frontend:  http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
