//! Main dashboard layout component
//!
//! Orchestrates all dashboard panels into a cohesive trading interface.

use dash_charts::{CandlestickChart, DepthChart};
use dash_state::{use_app_state, AppState, Panel};
use leptos::prelude::*;

use crate::{OrderBook, TickerBar, TradeHistory};

/// Dashboard layout configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DashboardLayout {
    #[default]
    Standard,
    ChartFocused,
    OrderBookFocused,
    Compact,
}

/// Main dashboard component
#[component]
pub fn Dashboard() -> impl IntoView {
    let state = use_app_state();

    view! {
        <div class="dashboard">
            // Header with ticker
            <header class="dash-header">
                <TickerBar
                    market=state.market.clone()
                    connection=state.connection.into()
                />
            </header>

            // Main content area
            <main class="dash-main">
                // Left panel: Order Book
                <aside class="dash-sidebar left">
                    <PanelContainer title="Order Book" panel=Panel::OrderBook>
                        <OrderBook market=state.market.clone() />
                    </PanelContainer>
                </aside>

                // Center: Charts
                <section class="dash-center">
                    <PanelContainer title="Chart" panel=Panel::CandleChart>
                        <div class="chart-container">
                            <CandlestickChart candles=state.market.candles.into() />
                        </div>
                    </PanelContainer>

                    <PanelContainer title="Market Depth" panel=Panel::DepthChart>
                        <div class="depth-container">
                            <DepthChart depth=state.market.depth.into() />
                        </div>
                    </PanelContainer>
                </section>

                // Right panel: Trades
                <aside class="dash-sidebar right">
                    <PanelContainer title="Recent Trades" panel=Panel::Trades>
                        <TradeHistory market=state.market.clone() />
                    </PanelContainer>
                </aside>
            </main>

            // Footer with status
            <footer class="dash-footer">
                <StatusBar state=state.clone() />
            </footer>
        </div>
    }
}

/// Panel container with header and visibility toggle
#[component]
fn PanelContainer(
    title: &'static str,
    panel: Panel,
    children: Children,
) -> impl IntoView {
    let state = use_app_state();
    let is_visible = move || state.is_panel_visible(panel);

    view! {
        <div class="panel" class:collapsed=move || !is_visible()>
            <div class="panel-header">
                <span class="panel-title">{title}</span>
                <button
                    class="panel-toggle"
                    on:click=move |_| state.toggle_panel(panel)
                    title=move || if is_visible() { "Collapse" } else { "Expand" }
                >
                    {move || if is_visible() { "−" } else { "+" }}
                </button>
            </div>
            <div class="panel-content" class:hidden=move || !is_visible()>
                {children()}
            </div>
        </div>
    }
}

/// Status bar with connection info and last update times
#[component]
fn StatusBar(state: AppState) -> impl IntoView {
    let connection = state.connection;
    let error = state.error;
    let last_trade = state.market.last_update.trade;
    let last_orderbook = state.market.last_update.orderbook;

    view! {
        <div class="status-bar">
            // Connection status
            <div class="sb-connection">
                <span class="sb-label">"Status:"</span>
                <span class=move || format!("sb-value {}", connection.get().css_class())>
                    {move || connection.get().label()}
                </span>
            </div>

            // Error display
            {move || {
                error.get().map(|e| {
                    view! {
                        <div class="sb-error">
                            <span class="error-icon">"⚠"</span>
                            <span class="error-msg">{e}</span>
                        </div>
                    }
                })
            }}

            // Last update times
            <div class="sb-updates">
                <span class="sb-item">
                    <span class="sb-label">"Trade:"</span>
                    <span class="sb-value">{move || format_timestamp(last_trade.get())}</span>
                </span>
                <span class="sb-item">
                    <span class="sb-label">"Book:"</span>
                    <span class="sb-value">{move || format_timestamp(last_orderbook.get())}</span>
                </span>
            </div>

            // Version
            <div class="sb-version">
                <span>"v0.1.0"</span>
            </div>
        </div>
    }
}

/// Format timestamp for status bar
fn format_timestamp(ts: i64) -> String {
    if ts == 0 {
        "-".to_string()
    } else {
        use chrono::{TimeZone, Utc};
        Utc.timestamp_millis_opt(ts)
            .single()
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

/// Compact dashboard for mobile/small screens
#[component]
pub fn DashboardCompact() -> impl IntoView {
    let state = use_app_state();

    view! {
        <div class="dashboard compact">
            // Compact ticker
            <header class="dash-header compact">
                <TickerBar
                    market=state.market.clone()
                    connection=state.connection.into()
                    config=Some(crate::ticker_bar::TickerBarConfig {
                        show_volume: false,
                        show_high_low: false,
                        show_spread: false,
                        compact: true,
                    })
                />
            </header>

            // Tabbed content
            <main class="dash-main compact">
                <DashboardTabs state=state.clone() />
            </main>
        </div>
    }
}

/// Tab-based navigation for compact view
#[component]
fn DashboardTabs(state: AppState) -> impl IntoView {
    let active_tab = RwSignal::new(0usize);

    let tabs = vec!["Chart", "Book", "Trades", "Depth"];

    view! {
        <div class="dash-tabs">
            // Tab headers
            <div class="tab-headers">
                {tabs.iter().enumerate().map(|(i, name)| {
                    view! {
                        <button
                            class="tab-header"
                            class:active=move || active_tab.get() == i
                            on:click=move |_| active_tab.set(i)
                        >
                            {*name}
                        </button>
                    }
                }).collect_view()}
            </div>

            // Tab content
            <div class="tab-content">
                {move || match active_tab.get() {
                    0 => view! {
                        <div class="tab-panel">
                            <CandlestickChart candles=state.market.candles.into() />
                        </div>
                    }.into_any(),
                    1 => view! {
                        <div class="tab-panel">
                            <OrderBook
                                market=state.market.clone()
                                config=Some(crate::order::OrderBookConfig::compact())
                            />
                        </div>
                    }.into_any(),
                    2 => view! {
                        <div class="tab-panel">
                            <TradeHistory
                                market=state.market.clone()
                                config=Some(crate::trade_history::TradeHistoryConfig::compact())
                            />
                        </div>
                    }.into_any(),
                    3 => view! {
                        <div class="tab-panel">
                            <DepthChart depth=state.market.depth.into() />
                        </div>
                    }.into_any(),
                    _ => view! { <div>"Unknown tab"</div> }.into_any(),
                }}
            </div>
        </div>
    }
}

/// Error boundary wrapper
#[component]
pub fn DashboardWithErrorBoundary() -> impl IntoView {
    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <div class="error-boundary">
                    <h2>"Something went wrong"</h2>
                    <ul>
                        {move || errors.get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                            .collect_view()
                        }
                    </ul>
                    <button on:click=|_| {
                        // Refresh page
                        let _ = web_sys::window()
                            .and_then(|w| w.location().reload().ok());
                    }>
                        "Reload"
                    </button>
                </div>
            }
        }>
            <Dashboard />
        </ErrorBoundary>
    }
}
