//! Ticker bar component for dashboard header
//!
//! Displays current price, 24h change, volume, and key stats.

use dash_core::{colors, ConnectionState, Ticker};
use dash_state::{AppState, MarketState};
use leptos::prelude::*;

/// Ticker bar configuration
#[derive(Debug, Clone)]
pub struct TickerBarConfig {
    pub show_volume: bool,
    pub show_high_low: bool,
    pub show_spread: bool,
    pub compact: bool,
}

impl Default for TickerBarConfig {
    fn default() -> Self {
        Self {
            show_volume: true,
            show_high_low: true,
            show_spread: true,
            compact: false,
        }
    }
}

/// Main ticker bar component
#[component]
pub fn TickerBar(
    #[prop(into)] market: MarketState,
    #[prop(into)] connection: Signal<ConnectionState>,
    #[prop(optional)] config: Option<TickerBarConfig>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    let show_volume = config.show_volume;
    let show_high_low = config.show_high_low;
    let show_spread = config.show_spread;

    let ticker = market.ticker;
    let symbol = market.symbol;

    view! {
        <div class="ticker-bar">
            // Symbol & Connection Status
            <div class="tb-symbol">
                <span class="symbol-name">{move || symbol.get().to_string()}</span>
                <ConnectionIndicator state=connection />
            </div>

            // Main price display
            <div class="tb-price">
                {move || {
                    ticker.get().map(|t| {
                        let color = t.color();
                        let arrow = t.arrow();
                        view! {
                            <>
                                <span class="price-value" style=format!("color: {}", color)>
                                    {format!("{:.2}", t.last_price.as_f64())}
                                </span>
                                <span class="price-change" style=format!("color: {}", color)>
                                    {arrow} {t.change_percent_str()}
                                </span>
                            </>
                        }
                    })
                }}
            </div>

            // 24h Stats
            <div class="tb-stats">
                // 24h Change
                <StatItem
                    label="24h Change"
                    value=move || {
                        ticker.get().map_or("-".to_string(), |t| t.change_str())
                    }
                    color=move || {
                        ticker.get().map_or(colors::NEUTRAL, |t| t.color())
                    }
                />

                // 24h High/Low
                {if show_high_low {
                    Some(view! {
                        <StatItem
                            label="24h High"
                            value=move || {
                                ticker.get().map_or("-".to_string(), |t| {
                                    format!("{:.2}", t.high_24h.as_f64())
                                })
                            }
                            color=colors::BULL
                        />
                        <StatItem
                            label="24h Low"
                            value=move || {
                                ticker.get().map_or("-".to_string(), |t| {
                                    format!("{:.2}", t.low_24h.as_f64())
                                })
                            }
                            color=colors::BEAR
                        />
                    })
                } else {
                    None
                }}

                // 24h Volume
                {if show_volume {
                    Some(view! {
                        <StatItem
                            label="24h Volume"
                            value=move || {
                                ticker.get().map_or("-".to_string(), |t| {
                                    let vol = t.volume_24h.as_f64();
                                    if vol >= 1_000_000.0 {
                                        format!("{:.2}M", vol / 1_000_000.0)
                                    } else if vol >= 1_000.0 {
                                        format!("{:.2}K", vol / 1_000.0)
                                    } else {
                                        format!("{:.4}", vol)
                                    }
                                })
                            }
                            color=colors::TEXT_PRIMARY
                        />
                    })
                } else {
                    None
                }}

                // Spread
                {if show_spread {
                    Some(view! {
                        <StatItem
                            label="Spread"
                            value=move || {
                                ticker.get().map_or("-".to_string(), |t| {
                                    format!("{:.2} ({:.3}%)", t.spread(), t.spread_percent())
                                })
                            }
                            color=colors::WARN
                        />
                    })
                } else {
                    None
                }}
            </div>
        </div>
    }
}

/// Individual stat item
#[component]
fn StatItem<V, C>(
    label: &'static str,
    value: V,
    color: C,
) -> impl IntoView
where
    V: Fn() -> String + 'static,
    C: Fn() -> &'static str + 'static,
{
    view! {
        <div class="tb-stat">
            <span class="stat-label">{label}</span>
            <span class="stat-value" style=move || format!("color: {}", color())>
                {value}
            </span>
        </div>
    }
}

/// Connection status indicator
#[component]
pub fn ConnectionIndicator(
    #[prop(into)] state: Signal<ConnectionState>,
) -> impl IntoView {
    let indicator_style = move || {
        let s = state.get();
        let color = match s {
            ConnectionState::Connected => colors::BULL,
            ConnectionState::Connecting | ConnectionState::Reconnecting => colors::WARN,
            ConnectionState::Disconnected => colors::BEAR,
        };
        format!("background-color: {}", color)
    };

    view! {
        <div class="connection-indicator" title=move || state.get().label()>
            <span class="indicator-dot" style=indicator_style />
            <span class="indicator-label">{move || state.get().label()}</span>
        </div>
    }
}

/// Compact price display (for mobile/sidebar)
#[component]
pub fn PriceDisplay(
    #[prop(into)] ticker: Signal<Option<Ticker>>,
) -> impl IntoView {
    view! {
        <div class="price-display">
            {move || {
                ticker.get().map(|t| {
                    let color = t.color();
                    view! {
                        <div class="pd-main">
                            <span class="pd-price" style=format!("color: {}", color)>
                                {format!("{:.2}", t.last_price.as_f64())}
                            </span>
                            <span class="pd-change" style=format!("color: {}", color)>
                                {t.arrow()} {t.change_percent_str()}
                            </span>
                        </div>
                    }
                })
            }}
        </div>
    }
}

/// Mini ticker strip (horizontal scrolling ticker)
#[component]
pub fn MiniTickerStrip(
    #[prop(into)] tickers: Signal<Vec<Ticker>>,
) -> impl IntoView {
    view! {
        <div class="mini-ticker-strip">
            <For
                each=move || tickers.get()
                key=|t| t.symbol.to_string()
                children=move |ticker| {
                    let color = ticker.color();
                    view! {
                        <div class="mts-item">
                            <span class="mts-symbol">{ticker.symbol.to_string()}</span>
                            <span class="mts-price" style=format!("color: {}", color)>
                                {format!("{:.2}", ticker.last_price.as_f64())}
                            </span>
                            <span class="mts-change" style=format!("color: {}", color)>
                                {ticker.change_percent_str()}
                            </span>
                        </div>
                    }
                }
            />
        </div>
    }
}

/// Range position indicator (where price is in 24h range)
#[component]
pub fn RangeIndicator(
    #[prop(into)] ticker: Signal<Option<Ticker>>,
    #[prop(default = 200.0)] width: f64,
    #[prop(default = 8.0)] height: f64,
) -> impl IntoView {
    let range_data = move || {
        ticker.get().map(|t| {
            let pos = t.range_position();
            let low = t.low_24h.as_f64();
            let high = t.high_24h.as_f64();
            let current = t.last_price.as_f64();
            (pos, low, high, current)
        })
    };

    view! {
        <div class="range-indicator">
            <div class="ri-labels">
                <span class="ri-low" style=format!("color: {}", colors::BEAR)>
                    {move || range_data().map_or("-".to_string(), |(_, low, _, _)| format!("{:.2}", low))}
                </span>
                <span class="ri-high" style=format!("color: {}", colors::BULL)>
                    {move || range_data().map_or("-".to_string(), |(_, _, high, _)| format!("{:.2}", high))}
                </span>
            </div>
            <svg
                class="ri-bar"
                viewBox=format!("0 0 {} {}", width, height)
                style="width: 100%; height: auto;"
            >
                // Background track
                <rect
                    x="0" y="0"
                    width=width height=height
                    fill=colors::BG_ELEVATED
                    rx="4"
                />

                // Gradient fill
                <defs>
                    <linearGradient id="rangeGradient">
                        <stop offset="0%" stop-color=colors::BEAR />
                        <stop offset="50%" stop-color=colors::WARN />
                        <stop offset="100%" stop-color=colors::BULL />
                    </linearGradient>
                </defs>
                <rect
                    x="2" y="2"
                    width=width - 4.0 height=height - 4.0
                    fill="url(#rangeGradient)"
                    rx="2"
                    opacity="0.3"
                />

                // Current position marker
                {move || {
                    range_data().map(|(pos, _, _, _)| {
                        let x = 4.0 + pos * (width - 8.0);
                        view! {
                            <circle
                                cx=x
                                cy=height / 2.0
                                r="4"
                                fill=colors::TEXT_PRIMARY
                                stroke=colors::BG_PANEL
                                stroke-width="2"
                            />
                        }
                    })
                }}
            </svg>
        </div>
    }
}
