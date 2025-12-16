//! Order book ladder display component
//!
//! Renders bid/ask levels with depth visualization bars.

use dash_core::{colors, OrderBookLevel, OrderBookSnapshot, OrderSide};
use dash_state::MarketState;
use leptos::prelude::*;

/// Order book display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderBookMode {
    #[default]
    Both,
    BidsOnly,
    AsksOnly,
}

/// Order book configuration
#[derive(Debug, Clone)]
pub struct OrderBookConfig {
    pub depth: usize,
    pub mode: OrderBookMode,
    pub show_spread: bool,
    pub show_totals: bool,
    pub compact: bool,
}

impl Default for OrderBookConfig {
    fn default() -> Self {
        Self {
            depth: 15,
            mode: OrderBookMode::Both,
            show_spread: true,
            show_totals: true,
            compact: false,
        }
    }
}

impl OrderBookConfig {
    pub fn compact() -> Self {
        Self {
            depth: 8,
            mode: OrderBookMode::Both,
            show_spread: true,
            show_totals: false,
            compact: true,
        }
    }
}

/// Main order book component
#[component]
pub fn OrderBook(
    #[prop(into)] market: MarketState,
    #[prop(optional)] config: Option<OrderBookConfig>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    let depth = config.depth;
    let show_spread = config.show_spread;
    let show_totals = config.show_totals;
    let compact = config.compact;

    let orderbook = market.orderbook;

    // Compute max quantity for bar scaling
    let max_qty = move || {
        orderbook.get().map_or(1.0, |book| book.max_quantity().max(0.001))
    };

    // Get asks (reversed so lowest ask is at bottom, closest to spread)
    let asks = move || {
        orderbook.get().map_or(vec![], |book| {
            let mut a: Vec<_> = book.asks.iter().take(depth).cloned().collect();
            a.reverse();
            a
        })
    };

    // Get bids (highest bid at top, closest to spread)
    let bids = move || {
        orderbook.get().map_or(vec![], |book| {
            book.bids.iter().take(depth).cloned().collect()
        })
    };

    // Spread info
    let spread_info = move || {
        orderbook.get().and_then(|book| {
            book.spread().zip(book.spread_percent()).map(|(s, pct)| {
                (format!("{:.2}", s), format!("{:.3}%", pct))
            })
        })
    };

    // Totals
    let totals = move || {
        orderbook.get().map(|book| {
            (book.total_bid_depth(), book.total_ask_depth())
        })
    };

    let row_class = if compact { "ob-row compact" } else { "ob-row" };

    view! {
        <div class="orderbook">
            // Header
            <div class="ob-header">
                <span class="ob-col price">"Price"</span>
                <span class="ob-col size">"Size"</span>
                <span class="ob-col total">"Total"</span>
            </div>

            // Asks (sells) - reversed order
            <div class="ob-asks">
                <For
                    each=asks
                    key=|level| format!("{:.8}", level.price.as_f64())
                    children=move |level| {
                        view! {
                            <OrderBookRow
                                level=level.clone()
                                side=OrderSide::Ask
                                max_qty=max_qty()
                                compact=compact
                            />
                        }
                    }
                />
            </div>

            // Spread indicator
            {move || {
                if show_spread {
                    spread_info().map(|(spread, pct)| {
                        view! {
                            <div class="ob-spread">
                                <span class="spread-label">"Spread"</span>
                                <span class="spread-value">{spread}</span>
                                <span class="spread-pct">{pct}</span>
                            </div>
                        }
                    })
                } else {
                    None
                }
            }}

            // Bids (buys)
            <div class="ob-bids">
                <For
                    each=bids
                    key=|level| format!("{:.8}", level.price.as_f64())
                    children=move |level| {
                        view! {
                            <OrderBookRow
                                level=level.clone()
                                side=OrderSide::Bid
                                max_qty=max_qty()
                                compact=compact
                            />
                        }
                    }
                />
            </div>

            // Totals footer
            {move || {
                if show_totals {
                    totals().map(|(bid_total, ask_total)| {
                        view! {
                            <div class="ob-totals">
                                <div class="total-bid">
                                    <span class="label">"Bid Total:"</span>
                                    <span class="value" style=format!("color: {}", colors::BULL)>
                                        {format!("{:.4}", bid_total)}
                                    </span>
                                </div>
                                <div class="total-ask">
                                    <span class="label">"Ask Total:"</span>
                                    <span class="value" style=format!("color: {}", colors::BEAR)>
                                        {format!("{:.4}", ask_total)}
                                    </span>
                                </div>
                            </div>
                        }
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}

/// Single order book row
#[component]
fn OrderBookRow(
    level: OrderBookLevel,
    side: OrderSide,
    max_qty: f64,
    #[prop(default = false)] compact: bool,
) -> impl IntoView {
    let price = level.price.as_f64();
    let qty = level.quantity.as_f64();
    let bar_pct = (qty / max_qty * 100.0).min(100.0);

    let price_str = if price >= 1000.0 {
        format!("{:.2}", price)
    } else if price >= 1.0 {
        format!("{:.4}", price)
    } else {
        format!("{:.6}", price)
    };

    let qty_str = if qty >= 1.0 {
        format!("{:.4}", qty)
    } else {
        format!("{:.6}", qty)
    };

    let value = price * qty;
    let value_str = if value >= 1000.0 {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
    };

    let (bar_color, text_color) = match side {
        OrderSide::Bid => (colors::bull_alpha(0.2), colors::BULL),
        OrderSide::Ask => (colors::bear_alpha(0.2), colors::BEAR),
    };

    let row_class = if compact { "ob-row compact" } else { "ob-row" };

    view! {
        <div
            class=row_class
            class=side.css_class()
            style=format!(
                "background: linear-gradient(to {}, {} {}%, transparent {}%)",
                if side == OrderSide::Bid { "left" } else { "right" },
                bar_color,
                bar_pct,
                bar_pct
            )
        >
            <span class="ob-col price" style=format!("color: {}", text_color)>
                {price_str}
            </span>
            <span class="ob-col size">
                {qty_str}
            </span>
            <span class="ob-col total">
                {value_str}
            </span>
        </div>
    }
}

/// Compact horizontal order book (5 levels each side)
#[component]
pub fn OrderBookCompact(
    #[prop(into)] market: MarketState,
    #[prop(default = 5)] levels: usize,
) -> impl IntoView {
    let orderbook = market.orderbook;

    let data = move || {
        orderbook.get().map(|book| {
            let bids: Vec<_> = book.bids.iter().take(levels).cloned().collect();
            let asks: Vec<_> = book.asks.iter().take(levels).cloned().collect();
            let max_qty = book.max_quantity().max(0.001);
            let spread = book.spread();
            (bids, asks, max_qty, spread)
        })
    };

    view! {
        <div class="orderbook-compact">
            {move || {
                data().map(|(bids, asks, max_qty, spread)| {
                    view! {
                        <div class="obc-container">
                            // Bids (left side)
                            <div class="obc-bids">
                                {bids.into_iter().rev().map(|level| {
                                    let pct = (level.quantity.as_f64() / max_qty * 100.0).min(100.0);
                                    view! {
                                        <div
                                            class="obc-level bid"
                                            style=format!(
                                                "background: linear-gradient(to left, {} {}%, transparent {}%)",
                                                colors::bull_alpha(0.3), pct, pct
                                            )
                                        >
                                            <span class="qty">{format!("{:.4}", level.quantity.as_f64())}</span>
                                            <span class="price">{format!("{:.2}", level.price.as_f64())}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>

                            // Spread center
                            <div class="obc-spread">
                                {spread.map(|s| format!("{:.2}", s)).unwrap_or_else(|| "-".to_string())}
                            </div>

                            // Asks (right side)
                            <div class="obc-asks">
                                {asks.into_iter().map(|level| {
                                    let pct = (level.quantity.as_f64() / max_qty * 100.0).min(100.0);
                                    view! {
                                        <div
                                            class="obc-level ask"
                                            style=format!(
                                                "background: linear-gradient(to right, {} {}%, transparent {}%)",
                                                colors::bear_alpha(0.3), pct, pct
                                            )
                                        >
                                            <span class="price">{format!("{:.2}", level.price.as_f64())}</span>
                                            <span class="qty">{format!("{:.4}", level.quantity.as_f64())}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }
                })
            }}
        </div>
    }
}
