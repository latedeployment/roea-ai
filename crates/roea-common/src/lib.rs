//! Common types and traits for roea-ai
//!
//! This crate provides shared data structures and abstractions used across
//! the roea-ai monitoring system.

pub mod events;
pub mod platform;
pub mod security;
pub mod signatures;

pub use events::*;
pub use platform::*;
pub use security::*;
pub use signatures::*;
