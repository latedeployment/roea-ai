//! Notification Manager
//!
//! This module provides a manager for coordinating notifications across multiple backends.

use super::{
    backends::create_backends, AlertNotification, NotificationBackend, NotificationConfig,
    NotificationResult, NotificationSeverity,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Notification manager that coordinates sending alerts to multiple backends
pub struct NotificationManager {
    /// Notification configuration
    config: NotificationConfig,
    /// Notification backends
    backends: Vec<Arc<dyn NotificationBackend>>,
    /// Rate limiting: last notification time per severity
    last_notification: RwLock<HashMap<String, Instant>>,
    /// Deduplication: recently sent notification hashes
    recent_notifications: RwLock<HashMap<String, Instant>>,
}

impl NotificationManager {
    /// Create a new notification manager from config
    pub fn new(config: NotificationConfig) -> Self {
        let backends = create_backends(&config);
        let enabled_count = backends.iter().filter(|b| b.is_enabled()).count();

        info!(
            "Notification manager initialized with {} backends ({} enabled)",
            backends.len(),
            enabled_count
        );

        for backend in &backends {
            if backend.is_enabled() {
                info!("Notification backend enabled: {}", backend.name());
            } else {
                debug!("Notification backend disabled: {}", backend.name());
            }
        }

        Self {
            config,
            backends,
            last_notification: RwLock::new(HashMap::new()),
            recent_notifications: RwLock::new(HashMap::new()),
        }
    }

    /// Check if notifications are enabled and configured
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.config.has_enabled_backends()
    }

    /// Get list of enabled backend names
    pub fn enabled_backends(&self) -> Vec<&'static str> {
        self.backends
            .iter()
            .filter(|b| b.is_enabled())
            .map(|b| b.name())
            .collect()
    }

    /// Send a notification to all enabled backends
    pub async fn notify(&self, notification: &AlertNotification) -> NotificationResult<()> {
        if !self.config.enabled {
            debug!("Notifications disabled, skipping");
            return Ok(());
        }

        // Check minimum severity
        if !self.meets_severity_threshold(&notification.severity) {
            debug!(
                "Notification severity {:?} below threshold {:?}, skipping",
                notification.severity, self.config.min_severity
            );
            return Ok(());
        }

        // Check rate limiting
        if !self.check_rate_limit(&notification.severity.to_string()).await {
            debug!("Rate limited, skipping notification");
            return Ok(());
        }

        // Check deduplication
        let dedup_key = self.dedup_key(notification);
        if !self.check_deduplication(&dedup_key).await {
            debug!("Duplicate notification, skipping");
            return Ok(());
        }

        // Send to all enabled backends
        let mut errors = Vec::new();
        let mut success_count = 0;

        for backend in &self.backends {
            if !backend.is_enabled() {
                continue;
            }

            match backend.send(notification).await {
                Ok(()) => {
                    success_count += 1;
                    debug!("Notification sent via {}", backend.name());
                }
                Err(e) => {
                    error!("Failed to send notification via {}: {}", backend.name(), e);
                    errors.push((backend.name(), e));
                }
            }
        }

        // Update rate limiting
        self.update_rate_limit(&notification.severity.to_string()).await;

        // Update deduplication
        self.update_deduplication(&dedup_key).await;

        if success_count > 0 {
            info!(
                "Notification sent to {} backend(s) ({} errors)",
                success_count,
                errors.len()
            );
            Ok(())
        } else if errors.is_empty() {
            warn!("No notification backends enabled");
            Ok(())
        } else {
            // Return the first error if all backends failed
            Err(errors.into_iter().next().unwrap().1)
        }
    }

    /// Send a test notification to verify configuration
    pub async fn send_test(&self) -> NotificationResult<()> {
        info!("Sending test notification to all backends");

        // Bypass rate limiting and deduplication for test
        let mut errors = Vec::new();
        let mut success_count = 0;

        for backend in &self.backends {
            if !backend.is_enabled() {
                continue;
            }

            match backend.test().await {
                Ok(()) => {
                    success_count += 1;
                    info!("Test notification sent via {}", backend.name());
                }
                Err(e) => {
                    error!("Test notification failed via {}: {}", backend.name(), e);
                    errors.push((backend.name(), e));
                }
            }
        }

        if success_count > 0 {
            info!(
                "Test notification sent to {} backend(s) ({} errors)",
                success_count,
                errors.len()
            );
            Ok(())
        } else if errors.is_empty() {
            warn!("No notification backends enabled for testing");
            Ok(())
        } else {
            Err(errors.into_iter().next().unwrap().1)
        }
    }

    /// Check if a severity meets the minimum threshold
    fn meets_severity_threshold(&self, severity: &NotificationSeverity) -> bool {
        let severity_level = match severity {
            NotificationSeverity::Info => 0,
            NotificationSeverity::Warning => 1,
            NotificationSeverity::Alert => 2,
            NotificationSeverity::Critical => 3,
        };

        let threshold_level = match self.config.min_severity {
            NotificationSeverity::Info => 0,
            NotificationSeverity::Warning => 1,
            NotificationSeverity::Alert => 2,
            NotificationSeverity::Critical => 3,
        };

        severity_level >= threshold_level
    }

    /// Check rate limiting
    async fn check_rate_limit(&self, key: &str) -> bool {
        if self.config.rate_limit_seconds == 0 {
            return true;
        }

        let last_notification = self.last_notification.read().await;
        if let Some(last_time) = last_notification.get(key) {
            let elapsed = last_time.elapsed();
            let limit = Duration::from_secs(self.config.rate_limit_seconds);
            elapsed >= limit
        } else {
            true
        }
    }

    /// Update rate limiting timestamp
    async fn update_rate_limit(&self, key: &str) {
        let mut last_notification = self.last_notification.write().await;
        last_notification.insert(key.to_string(), Instant::now());
    }

    /// Generate deduplication key
    fn dedup_key(&self, notification: &AlertNotification) -> String {
        format!(
            "{}:{}:{}:{}",
            notification.pid,
            notification.process_name,
            notification.path.as_deref().unwrap_or(""),
            notification.operation.as_deref().unwrap_or("")
        )
    }

    /// Check deduplication (within 60 second window)
    async fn check_deduplication(&self, key: &str) -> bool {
        let mut recent = self.recent_notifications.write().await;

        // Clean up old entries
        let cutoff = Instant::now() - Duration::from_secs(60);
        recent.retain(|_, time| *time > cutoff);

        // Check if this notification was recently sent
        !recent.contains_key(key)
    }

    /// Update deduplication cache
    async fn update_deduplication(&self, key: &str) {
        let mut recent = self.recent_notifications.write().await;
        recent.insert(key.to_string(), Instant::now());
    }

    /// Clean up old deduplication entries (call periodically)
    pub async fn cleanup(&self) {
        let mut recent = self.recent_notifications.write().await;
        let cutoff = Instant::now() - Duration::from_secs(60);
        let before = recent.len();
        recent.retain(|_, time| *time > cutoff);
        let after = recent.len();
        if before != after {
            debug!(
                "Cleaned up {} deduplication entries ({} remaining)",
                before - after,
                after
            );
        }
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new(NotificationConfig::default())
    }
}

/// Builder for NotificationManager
pub struct NotificationManagerBuilder {
    config: NotificationConfig,
}

impl NotificationManagerBuilder {
    pub fn new() -> Self {
        Self {
            config: NotificationConfig::default(),
        }
    }

    pub fn config(mut self, config: NotificationConfig) -> Self {
        self.config = config;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    pub fn min_severity(mut self, severity: NotificationSeverity) -> Self {
        self.config.min_severity = severity;
        self
    }

    pub fn rate_limit_seconds(mut self, seconds: u64) -> Self {
        self.config.rate_limit_seconds = seconds;
        self
    }

    pub fn build(self) -> NotificationManager {
        NotificationManager::new(self.config)
    }
}

impl Default for NotificationManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_threshold() {
        let config = NotificationConfig {
            min_severity: NotificationSeverity::Warning,
            ..Default::default()
        };
        let manager = NotificationManager::new(config);

        assert!(!manager.meets_severity_threshold(&NotificationSeverity::Info));
        assert!(manager.meets_severity_threshold(&NotificationSeverity::Warning));
        assert!(manager.meets_severity_threshold(&NotificationSeverity::Alert));
        assert!(manager.meets_severity_threshold(&NotificationSeverity::Critical));
    }

    #[test]
    fn test_dedup_key() {
        let manager = NotificationManager::new(NotificationConfig::default());

        let mut notification = AlertNotification::new(
            NotificationSeverity::Alert,
            "Test",
            "Test message",
        );
        notification.pid = 1234;
        notification.process_name = "test_process".to_string();
        notification.path = Some("/etc/passwd".to_string());
        notification.operation = Some("read".to_string());

        let key = manager.dedup_key(&notification);
        assert_eq!(key, "1234:test_process:/etc/passwd:read");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = NotificationConfig {
            rate_limit_seconds: 1,
            ..Default::default()
        };
        let manager = NotificationManager::new(config);

        // First call should pass
        assert!(manager.check_rate_limit("test").await);

        // Update rate limit
        manager.update_rate_limit("test").await;

        // Immediate second call should fail
        assert!(!manager.check_rate_limit("test").await);

        // After waiting, should pass again
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(manager.check_rate_limit("test").await);
    }

    #[tokio::test]
    async fn test_deduplication() {
        let manager = NotificationManager::new(NotificationConfig::default());

        let key = "test_key";

        // First call should pass
        assert!(manager.check_deduplication(key).await);

        // Update dedup cache
        manager.update_deduplication(key).await;

        // Second call should fail
        assert!(!manager.check_deduplication(key).await);
    }
}
