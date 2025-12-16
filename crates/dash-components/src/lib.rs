//! # dash-components
//!
//! Leptos UI components for the BTC Exchange Dashboard.
//!
//! ## Components
//!
//! - `order` - Order book ladder display
//! - `trade_history` - Recent trades tape
//! - `ticker_bar` - Header ticker with price/stats
//! - `dashboard` - Main dashboard layout

pub mod dashboard;
pub mod order;
pub mod ticker_bar;
pub mod trade_history;

pub use dashboard::*;
pub use order::*;
pub use ticker_bar::*;
pub use trade_history::*;
