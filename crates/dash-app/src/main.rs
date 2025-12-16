//! BTC Exchange Dashboard - WASM Entry Point
//!
//! This is the main entry point for the browser-based WASM application.

use dash_components::Dashboard;
use dash_state::provide_app_state;
use dash_websocket::{use_websocket, WsConfig, ExponentialBackoff};
use leptos::prelude::*;

/// Main application component
#[component]
fn App() -> impl IntoView {
    // Initialize app state and provide to component tree
    let state = provide_app_state();

    // Configure WebSocket connection
    let ws_config = WsConfig::new(get_ws_url())
        .with_policy(ExponentialBackoff::aggressive())
        .heartbeat(30000);

    // Start WebSocket connection
    let _ws_handle = use_websocket(state.clone(), Some(ws_config.url.clone()));

    // Mark body as hydrated (hides loading spinner)
    Effect::new(move |_| {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(body) = document.body() {
                let _ = body.class_list().add_1("hydrated");
            }
        }
    });

    view! {
        <Dashboard />
    }
}

/// Determine WebSocket URL based on environment
fn get_ws_url() -> String {
    // Try to get from window location for production
    if let Some(window) = web_sys::window() {
        if let Ok(location) = window.location().host() {
            let protocol = if window.location().protocol().map(|p| p == "https:").unwrap_or(false) {
                "wss"
            } else {
                "ws"
            };
            return format!("{}://{}/ws", protocol, location);
        }
    }

    // Fallback to default dev server
    dash_websocket::DEFAULT_WS_URL.to_string()
}

/// WASM entry point
fn main() {
    // Set up panic hook for better error messages in browser console
    console_error_panic_hook::set_once();

    // Initialize tracing for browser console
    tracing_wasm::set_as_global_default();

    tracing::info!("🚀 BTC Exchange Dashboard starting...");

    // Mount the app to the DOM
    leptos::mount::mount_to_body(App);

    tracing::info!("✅ Dashboard mounted");
}
