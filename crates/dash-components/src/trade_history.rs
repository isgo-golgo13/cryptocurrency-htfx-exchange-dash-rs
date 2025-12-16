//! Trade history (tape) component
//!
//! Displays recent trades in a scrolling list with whale detection.

use dash_core::{colors, Trade, TradeSide, TradeClassification, ValueThresholdClassifier, TradeClassifier};
use dash_state::MarketState;
use leptos::prelude::*;

/// Trade history configuration
#[derive(Debug, Clone)]
pub struct TradeHistoryConfig {
    pub max_visible: usize,
    pub show_value: bool,
    pub highlight_whales: bool,
    pub compact: bool,
}

impl Default for TradeHistoryConfig {
    fn default() -> Self {
        Self {
            max_visible: 50,
            show_value: true,
            highlight_whales: true,
            compact: false,
        }
    }
}

impl TradeHistoryConfig {
    pub fn compact() -> Self {
        Self {
            max_visible: 20,
            show_value: false,
            highlight_whales: true,
            compact: true,
        }
    }
}

/// Trade history component
#[component]
pub fn TradeHistory(
    #[prop(into)] market: MarketState,
    #[prop(optional)] config: Option<TradeHistoryConfig>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    let max_visible = config.max_visible;
    let show_value = config.show_value;
    let highlight_whales = config.highlight_whales;
    let compact = config.compact;

    let trades = market.trades;
    let classifier = ValueThresholdClassifier::default();

    let visible_trades = move || {
        trades.get().into_iter().take(max_visible).collect::<Vec<_>>()
    };

    view! {
        <div class="trade-history" class:compact=compact>
            // Header
            <div class="th-header">
                <span class="th-col time">"Time"</span>
                <span class="th-col side">"Side"</span>
                <span class="th-col price">"Price"</span>
                <span class="th-col size">"Size"</span>
                {if show_value {
                    Some(view! { <span class="th-col value">"Value"</span> })
                } else {
                    None
                }}
            </div>

            // Trade list
            <div class="th-list">
                <For
                    each=visible_trades
                    key=|trade| trade.id.clone()
                    children=move |trade| {
                        let classification = if highlight_whales {
                            Some(classifier.classify(&trade))
                        } else {
                            None
                        };

                        view! {
                            <TradeRow
                                trade=trade
                                show_value=show_value
                                classification=classification
                                compact=compact
                            />
                        }
                    }
                />
            </div>
        </div>
    }
}

/// Single trade row
#[component]
fn TradeRow(
    trade: Trade,
    show_value: bool,
    classification: Option<TradeClassification>,
    #[prop(default = false)] compact: bool,
) -> impl IntoView {
    let time_str = if compact {
        trade.time_short()
    } else {
        trade.time_str()
    };

    let price = trade.price.as_f64();
    let qty = trade.quantity.as_f64();
    let value = trade.value();

    let price_str = if price >= 1000.0 {
        format!("{:.2}", price)
    } else {
        format!("{:.4}", price)
    };

    let qty_str = if qty >= 1.0 {
        format!("{:.4}", qty)
    } else {
        format!("{:.6}", qty)
    };

    let value_str = if value >= 1_000_000.0 {
        format!("{:.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.2}K", value / 1_000.0)
    } else {
        format!("{:.2}", value)
    };

    let side_color = trade.side.color();
    let side_label = trade.side.label();
    let side_arrow = trade.side.arrow();

    let row_class = match classification {
        Some(TradeClassification::Whale) => "th-row whale",
        Some(TradeClassification::Large) => "th-row large",
        _ => "th-row",
    };

    let whale_icon = classification.and_then(|c| c.icon());

    view! {
        <div class=row_class class=trade.side.css_class()>
            <span class="th-col time">{time_str}</span>
            <span class="th-col side" style=format!("color: {}", side_color)>
                {side_arrow}
                {if !compact { Some(format!(" {}", side_label)) } else { None }}
            </span>
            <span class="th-col price" style=format!("color: {}", side_color)>
                {price_str}
            </span>
            <span class="th-col size">
                {qty_str}
                {whale_icon.map(|icon| view! { <span class="whale-icon">{icon}</span> })}
            </span>
            {if show_value {
                Some(view! { <span class="th-col value">{value_str}</span> })
            } else {
                None
            }}
        </div>
    }
}

/// Trade flow summary (aggregated buy/sell volumes)
#[component]
pub fn TradeFlowSummary(
    #[prop(into)] market: MarketState,
    #[prop(default = 50)] window: usize,
) -> impl IntoView {
    let trades = market.trades;

    let flow_data = move || {
        let trade_list = trades.get();
        let recent: Vec<_> = trade_list.iter().take(window).collect();

        if recent.is_empty() {
            return (0.0, 0.0, 0.5);
        }

        let mut buy_vol = 0.0;
        let mut sell_vol = 0.0;

        for trade in recent {
            match trade.side {
                TradeSide::Buy => buy_vol += trade.quantity.as_f64(),
                TradeSide::Sell => sell_vol += trade.quantity.as_f64(),
            }
        }

        let total = buy_vol + sell_vol;
        let buy_ratio = if total > 0.0 { buy_vol / total } else { 0.5 };

        (buy_vol, sell_vol, buy_ratio)
    };

    view! {
        <div class="trade-flow-summary">
            <div class="tfs-header">"Trade Flow"</div>
            <div class="tfs-content">
                // Buy volume
                <div class="tfs-side buy">
                    <span class="label">"Buy"</span>
                    <span class="value" style=format!("color: {}", colors::BULL)>
                        {move || format!("{:.4}", flow_data().0)}
                    </span>
                </div>

                // Visual bar
                <div class="tfs-bar">
                    <div
                        class="tfs-bar-fill buy"
                        style=move || format!(
                            "width: {}%; background: {}",
                            flow_data().2 * 100.0,
                            colors::BULL
                        )
                    />
                    <div
                        class="tfs-bar-fill sell"
                        style=move || format!(
                            "width: {}%; background: {}",
                            (1.0 - flow_data().2) * 100.0,
                            colors::BEAR
                        )
                    />
                </div>

                // Sell volume
                <div class="tfs-side sell">
                    <span class="label">"Sell"</span>
                    <span class="value" style=format!("color: {}", colors::BEAR)>
                        {move || format!("{:.4}", flow_data().1)}
                    </span>
                </div>
            </div>
        </div>
    }
}

/// Recent large trades alert
#[component]
pub fn LargeTradesAlert(
    #[prop(into)] market: MarketState,
    #[prop(default = 100_000.0)] threshold: f64,
) -> impl IntoView {
    let trades = market.trades;
    let classifier = ValueThresholdClassifier {
        large_threshold: threshold,
        ..Default::default()
    };

    let large_trades = move || {
        trades.get()
            .into_iter()
            .filter(|t| {
                matches!(
                    classifier.classify(t),
                    TradeClassification::Large | TradeClassification::Whale
                )
            })
            .take(5)
            .collect::<Vec<_>>()
    };

    view! {
        <div class="large-trades-alert">
            <div class="lta-header">
                <span class="icon">"🐋"</span>
                <span class="title">"Large Trades"</span>
            </div>
            <div class="lta-list">
                <For
                    each=large_trades
                    key=|t| t.id.clone()
                    children=move |trade| {
                        let is_whale = trade.value() >= 1_000_000.0;
                        view! {
                            <div class="lta-item" class:whale=is_whale>
                                <span class="time">{trade.time_short()}</span>
                                <span class="side" style=format!("color: {}", trade.side.color())>
                                    {trade.side.arrow()}
                                </span>
                                <span class="value">
                                    {format!("${:.0}K", trade.value() / 1000.0)}
                                </span>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
