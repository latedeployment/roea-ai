//! Alert Notification Module
//!
//! This module provides a pluggable notification system for sending alerts
//! when protected files are accessed or other security events occur.
//!
//! Supported notification backends:
//! - **Slack**: Webhook-based notifications with rich formatting
//! - **Discord**: Webhook-based notifications with embeds
//! - **ntfy.sh**: Simple HTTP push notifications (self-hosted or cloud)
//! - **Telegram**: Bot API notifications
//! - **PagerDuty**: Incident management integration
//! - **Email (SMTP)**: Traditional email alerts
//! - **Pushover**: Mobile push notifications
//! - **Syslog**: System logging (local or remote)
//! - **Webhook**: Generic HTTP webhooks for custom integrations

mod backends;
mod config;
mod manager;

pub use backends::*;
pub use config::*;
pub use manager::*;

use crate::protection::ProtectionEvent;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Notification severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotificationSeverity {
    /// Informational - low priority
    Info,
    /// Warning - medium priority
    Warning,
    /// Alert - high priority
    #[default]
    Alert,
    /// Critical - immediate attention required
    Critical,
}

impl NotificationSeverity {
    /// Get emoji representation for the severity
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Info => "ℹ️",
            Self::Warning => "⚠️",
            Self::Alert => "🚨",
            Self::Critical => "🔴",
        }
    }

    /// Get color code (hex) for the severity
    pub fn color(&self) -> &'static str {
        match self {
            Self::Info => "#36a64f",    // Green
            Self::Warning => "#ffcc00", // Yellow
            Self::Alert => "#ff6600",   // Orange
            Self::Critical => "#ff0000", // Red
        }
    }

    /// Get color as integer for Discord embeds
    pub fn color_int(&self) -> u32 {
        match self {
            Self::Info => 0x36a64f,
            Self::Warning => 0xffcc00,
            Self::Alert => 0xff6600,
            Self::Critical => 0xff0000,
        }
    }

    /// Get PagerDuty severity string
    pub fn pagerduty_severity(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Alert => "error",
            Self::Critical => "critical",
        }
    }

    /// Get ntfy.sh priority level (1-5)
    pub fn ntfy_priority(&self) -> u8 {
        match self {
            Self::Info => 2,
            Self::Warning => 3,
            Self::Alert => 4,
            Self::Critical => 5,
        }
    }
}

impl fmt::Display for NotificationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Alert => write!(f, "alert"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for NotificationSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Self::Info),
            "warning" | "warn" => Ok(Self::Warning),
            "alert" | "error" => Ok(Self::Alert),
            "critical" | "crit" => Ok(Self::Critical),
            _ => Err(format!("Unknown severity: {}", s)),
        }
    }
}

/// Alert notification payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    /// Unique alert ID
    pub id: String,
    /// Timestamp of the alert
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Alert severity
    pub severity: NotificationSeverity,
    /// Short title/summary
    pub title: String,
    /// Detailed message
    pub message: String,
    /// Process ID that triggered the alert
    pub pid: u32,
    /// Process name
    pub process_name: String,
    /// File path that was accessed (if applicable)
    pub path: Option<String>,
    /// Operation type (read, write, etc.)
    pub operation: Option<String>,
    /// Whether the access was blocked
    pub blocked: bool,
    /// Hostname where the alert originated
    pub hostname: String,
    /// Additional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl AlertNotification {
    /// Create a new alert notification
    pub fn new(
        severity: NotificationSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            severity,
            title: title.into(),
            message: message.into(),
            pid: 0,
            process_name: String::new(),
            path: None,
            operation: None,
            blocked: false,
            hostname: gethostname(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create from a ProtectionEvent
    pub fn from_protection_event(event: &ProtectionEvent) -> Self {
        let severity = match event.severity.to_lowercase().as_str() {
            "info" => NotificationSeverity::Info,
            "warning" => NotificationSeverity::Warning,
            "alert" => NotificationSeverity::Alert,
            "critical" => NotificationSeverity::Critical,
            _ => NotificationSeverity::Alert,
        };

        let status = if event.blocked { "BLOCKED" } else { "DETECTED" };
        let title = format!(
            "{} Protected File Access {}",
            severity.emoji(),
            status
        );

        let message = format!(
            "Process '{}' (PID: {}) attempted to {} protected file: {}",
            event.process_name,
            event.pid,
            event.operation.to_lowercase(),
            event.path
        );

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: event.timestamp,
            severity,
            title,
            message,
            pid: event.pid,
            process_name: event.process_name.clone(),
            path: Some(event.path.clone()),
            operation: Some(event.operation.clone()),
            blocked: event.blocked,
            hostname: gethostname(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add metadata to the notification
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Format as a plain text message
    pub fn to_plain_text(&self) -> String {
        let mut text = format!(
            "[{}] {} - {}\n\n{}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.severity.to_string().to_uppercase(),
            self.title,
            self.message
        );

        if !self.process_name.is_empty() {
            text.push_str(&format!("\n\nProcess: {} (PID: {})", self.process_name, self.pid));
        }

        if let Some(ref path) = self.path {
            text.push_str(&format!("\nPath: {}", path));
        }

        if let Some(ref op) = self.operation {
            text.push_str(&format!("\nOperation: {}", op));
        }

        if self.blocked {
            text.push_str("\nStatus: BLOCKED");
        }

        text.push_str(&format!("\nHost: {}", self.hostname));

        text
    }
}

/// Get the system hostname
fn gethostname() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Notification backend errors
#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Email error: {0}")]
    EmailError(String),

    #[error("Syslog error: {0}")]
    SyslogError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Backend not available: {0}")]
    BackendUnavailable(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for notification operations
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Trait for notification backends
///
/// Each notification backend implements this trait to provide
/// a consistent interface for sending alerts.
#[async_trait]
pub trait NotificationBackend: Send + Sync {
    /// Get the backend name
    fn name(&self) -> &'static str;

    /// Check if the backend is enabled
    fn is_enabled(&self) -> bool;

    /// Send an alert notification
    async fn send(&self, notification: &AlertNotification) -> NotificationResult<()>;

    /// Test the backend configuration
    async fn test(&self) -> NotificationResult<()> {
        let test_notification = AlertNotification::new(
            NotificationSeverity::Info,
            "roea-ai Test Notification",
            "This is a test notification to verify the notification backend is configured correctly.",
        );
        self.send(&test_notification).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_parsing() {
        assert_eq!(
            "info".parse::<NotificationSeverity>().unwrap(),
            NotificationSeverity::Info
        );
        assert_eq!(
            "warning".parse::<NotificationSeverity>().unwrap(),
            NotificationSeverity::Warning
        );
        assert_eq!(
            "critical".parse::<NotificationSeverity>().unwrap(),
            NotificationSeverity::Critical
        );
    }

    #[test]
    fn test_notification_plain_text() {
        let notification = AlertNotification::new(
            NotificationSeverity::Critical,
            "Test Alert",
            "This is a test message",
        );

        let text = notification.to_plain_text();
        assert!(text.contains("CRITICAL"));
        assert!(text.contains("Test Alert"));
        assert!(text.contains("This is a test message"));
    }

    #[test]
    fn test_severity_colors() {
        assert_eq!(NotificationSeverity::Info.color(), "#36a64f");
        assert_eq!(NotificationSeverity::Critical.color(), "#ff0000");
    }

    #[test]
    fn test_ntfy_priority() {
        assert_eq!(NotificationSeverity::Info.ntfy_priority(), 2);
        assert_eq!(NotificationSeverity::Critical.ntfy_priority(), 5);
    }
}
