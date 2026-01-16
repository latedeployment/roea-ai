//! Modern TUI for roea-agent
//!
//! A rich terminal user interface for monitoring AI agents in real-time.

mod app;
mod events;
mod ui;
mod widgets;

pub use app::{App, run_tui};
