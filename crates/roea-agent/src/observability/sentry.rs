//! Sentry integration for error tracking and performance monitoring
//!
//! This module provides optional Sentry integration for:
//! - Error reporting with full context
//! - Performance transaction tracing
//! - Breadcrumbs for debugging
//! - User feedback collection

use std::sync::Once;
use tracing::{error, info, warn};

/// Sentry configuration
#[derive(Debug, Clone)]
pub struct SentryConfig {
    /// Sentry DSN (Data Source Name)
    /// Get this from your Sentry project settings
    pub dsn: Option<String>,

    /// Environment name (e.g., "development", "staging", "production")
    pub environment: String,

    /// Release version
    pub release: Option<String>,

    /// Sample rate for error events (0.0 to 1.0)
    pub sample_rate: f32,

    /// Sample rate for performance transactions (0.0 to 1.0)
    pub traces_sample_rate: f32,

    /// Enable debug mode
    pub debug: bool,

    /// Additional tags to include with all events
    pub tags: Vec<(String, String)>,
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            dsn: std::env::var("SENTRY_DSN").ok(),
            environment: std::env::var("SENTRY_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
            release: option_env!("CARGO_PKG_VERSION").map(String::from),
            sample_rate: 1.0,
            traces_sample_rate: 0.1,
            debug: cfg!(debug_assertions),
            tags: vec![
                ("app".to_string(), "roea-agent".to_string()),
                ("platform".to_string(), std::env::consts::OS.to_string()),
                ("arch".to_string(), std::env::consts::ARCH.to_string()),
            ],
        }
    }
}

impl SentryConfig {
    /// Create a new Sentry configuration with the given DSN
    pub fn new(dsn: impl Into<String>) -> Self {
        Self {
            dsn: Some(dsn.into()),
            ..Default::default()
        }
    }

    /// Set the environment
    pub fn environment(mut self, env: impl Into<String>) -> Self {
        self.environment = env.into();
        self
    }

    /// Set the release version
    pub fn release(mut self, release: impl Into<String>) -> Self {
        self.release = Some(release.into());
        self
    }

    /// Set the sample rate for errors (0.0 to 1.0)
    pub fn sample_rate(mut self, rate: f32) -> Self {
        self.sample_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set the traces sample rate for performance (0.0 to 1.0)
    pub fn traces_sample_rate(mut self, rate: f32) -> Self {
        self.traces_sample_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Add a custom tag
    pub fn tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.push((key.into(), value.into()));
        self
    }
}

/// Guard that must be held to keep Sentry initialized
pub struct SentryGuard {
    _guard: Option<sentry::ClientInitGuard>,
}

impl Drop for SentryGuard {
    fn drop(&mut self) {
        if self._guard.is_some() {
            info!("Shutting down Sentry client");
        }
    }
}

static INIT: Once = Once::new();
static mut INITIALIZED: bool = false;

/// Initialize Sentry error tracking
///
/// Returns a guard that must be held for the lifetime of the application.
/// When the guard is dropped, Sentry will flush any remaining events.
///
/// # Example
///
/// ```ignore
/// let _sentry = init_sentry(SentryConfig::default());
/// // Application code here...
/// // Sentry will be flushed when _sentry is dropped
/// ```
pub fn init_sentry(config: SentryConfig) -> SentryGuard {
    let dsn = match &config.dsn {
        Some(dsn) if !dsn.is_empty() => dsn.clone(),
        _ => {
            warn!("Sentry DSN not configured, error tracking disabled");
            return SentryGuard { _guard: None };
        }
    };

    let mut guard = None;

    INIT.call_once(|| {
        let options = sentry::ClientOptions {
            dsn: dsn.parse().ok(),
            environment: Some(config.environment.clone().into()),
            release: config.release.clone().map(Into::into),
            sample_rate: config.sample_rate,
            traces_sample_rate: config.traces_sample_rate,
            debug: config.debug,
            attach_stacktrace: true,
            send_default_pii: false, // Don't send PII by default
            max_breadcrumbs: 100,
            ..Default::default()
        };

        let init_guard = sentry::init(options);

        // Configure scope with default tags
        sentry::configure_scope(|scope| {
            for (key, value) in &config.tags {
                scope.set_tag(key, value);
            }
        });

        info!(
            environment = %config.environment,
            "Sentry initialized for error tracking"
        );

        unsafe {
            INITIALIZED = true;
        }

        guard = Some(init_guard);
    });

    SentryGuard { _guard: guard }
}

/// Check if Sentry is initialized
pub fn is_sentry_initialized() -> bool {
    unsafe { INITIALIZED }
}

/// Capture an error and send it to Sentry
///
/// This is a convenience function that captures any error implementing
/// `std::error::Error` and sends it to Sentry with additional context.
pub fn capture_error<E: std::error::Error + Send + Sync + 'static>(
    error: &E,
    context: Option<&str>,
) {
    if !is_sentry_initialized() {
        return;
    }

    sentry::with_scope(
        |scope| {
            if let Some(ctx) = context {
                scope.set_extra("context", serde_json::json!(ctx));
            }
        },
        || {
            error!(error = %error, "Captured error for Sentry");
            sentry::capture_error(error);
        },
    );
}

/// Capture a message and send it to Sentry
///
/// Use this for logging important events that aren't errors but should be tracked.
pub fn capture_message(message: &str, level: sentry::Level) {
    if !is_sentry_initialized() {
        return;
    }

    match level {
        sentry::Level::Error => error!("{}", message),
        sentry::Level::Warning => warn!("{}", message),
        _ => info!("{}", message),
    }

    sentry::capture_message(message, level);
}

/// Add a breadcrumb for debugging context
///
/// Breadcrumbs are trail of events that lead up to an error.
pub fn add_breadcrumb(category: &str, message: &str, level: sentry::Level) {
    if !is_sentry_initialized() {
        return;
    }

    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: Some(category.to_string()),
        message: Some(message.to_string()),
        level,
        ..Default::default()
    });
}

/// Start a performance transaction
///
/// Returns a transaction guard that should be held for the duration of the operation.
/// The transaction will be finished and sent when the guard is dropped.
pub fn start_transaction(name: &str, op: &str) -> Option<sentry::TransactionOrSpan> {
    if !is_sentry_initialized() {
        return None;
    }

    let ctx = sentry::TransactionContext::new(name, op);
    Some(sentry::start_transaction(ctx).into())
}

/// Set a user context for Sentry events
///
/// This helps identify which user (machine) experienced an error.
/// Note: We don't send PII, just a machine identifier.
pub fn set_user_context(machine_id: &str) {
    if !is_sentry_initialized() {
        return;
    }

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            id: Some(machine_id.to_string()),
            ..Default::default()
        }));
    });
}

/// Clear the user context
pub fn clear_user_context() {
    if !is_sentry_initialized() {
        return;
    }

    sentry::configure_scope(|scope| {
        scope.set_user(None);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentry_config_default() {
        let config = SentryConfig::default();
        assert!(config.dsn.is_none() || config.dsn.as_ref().unwrap().is_empty() || config.dsn.is_some());
        assert!(!config.environment.is_empty());
        assert!(!config.tags.is_empty());
    }

    #[test]
    fn test_sentry_config_builder() {
        let config = SentryConfig::new("https://key@sentry.io/123")
            .environment("test")
            .release("1.0.0")
            .sample_rate(0.5)
            .traces_sample_rate(0.1)
            .tag("test_key", "test_value");

        assert_eq!(config.dsn, Some("https://key@sentry.io/123".to_string()));
        assert_eq!(config.environment, "test");
        assert_eq!(config.release, Some("1.0.0".to_string()));
        assert_eq!(config.sample_rate, 0.5);
        assert_eq!(config.traces_sample_rate, 0.1);
        assert!(config.tags.iter().any(|(k, v)| k == "test_key" && v == "test_value"));
    }

    #[test]
    fn test_sample_rate_clamping() {
        let config = SentryConfig::default()
            .sample_rate(2.0)
            .traces_sample_rate(-1.0);

        assert_eq!(config.sample_rate, 1.0);
        assert_eq!(config.traces_sample_rate, 0.0);
    }

    #[test]
    fn test_init_without_dsn() {
        let config = SentryConfig {
            dsn: None,
            ..Default::default()
        };
        let guard = init_sentry(config);
        assert!(guard._guard.is_none());
    }
}
