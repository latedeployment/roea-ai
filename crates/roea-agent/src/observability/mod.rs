//! Observability module for roea-ai agent
//!
//! Provides integration with external monitoring services:
//! - Sentry for error tracking
//! - Metrics collection for performance monitoring
//! - Health checks for deployment monitoring

mod sentry;
mod metrics;

pub use self::sentry::*;
pub use metrics::*;
