//! BTC Exchange Dashboard - WASM Entry Point

use dash_components::Dashboard;
use dash_state::provide_app_state;
use dash_websocket::{use_websocket, WsConfig, ExponentialBackoff};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
fn App() -> impl IntoView {
    let state = provide_app_state();

    let ws_config = WsConfig::new(get_ws_url())
        .with_policy(ExponentialBackoff::aggressive())
        .heartbeat(30000);

    let _ws_handle = use_websocket(state.clone(), Some(ws_config.url.clone()));

    view! {
        <Dashboard />
    }
}

fn get_ws_url() -> String {
    dash_websocket::DEFAULT_WS_URL.to_string()
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    tracing::info!("🚀 BTC Exchange Dashboard starting...");

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");
    let app_element = document
        .get_element_by_id("app")
        .expect("should find #app element")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("should be HtmlElement");
    
    app_element.set_inner_html("");
    
    // .forget() keeps the view mounted permanently
    leptos::mount::mount_to(app_element, App).forget();

    tracing::info!("Dashboard mounted");
}