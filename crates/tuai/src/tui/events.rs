//! Event handling for the TUI
//!
//! This module provides event types and utilities for the TUI.

use std::time::Duration;
use crossterm::event::{self, Event, KeyEvent};

/// Poll for terminal events with timeout
pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

/// Extract key event from terminal event
pub fn extract_key_event(event: Event) -> Option<KeyEvent> {
    match event {
        Event::Key(key) => Some(key),
        _ => None,
    }
}
