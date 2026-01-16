//! Notification Configuration
//!
//! This module defines the configuration structures for the notification system.

use super::backends::*;
use super::NotificationSeverity;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main notification configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Global enable/disable for notifications
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Minimum severity level to trigger notifications
    #[serde(default)]
    pub min_severity: NotificationSeverity,

    /// Rate limiting: minimum seconds between notifications
    #[serde(default = "default_rate_limit")]
    pub rate_limit_seconds: u64,

    /// Include test notification on startup
    #[serde(default)]
    pub send_test_on_startup: bool,

    /// Slack configuration
    #[serde(default)]
    pub slack: Option<SlackConfig>,

    /// Discord configuration
    #[serde(default)]
    pub discord: Option<DiscordConfig>,

    /// ntfy.sh configuration
    #[serde(default)]
    pub ntfy: Option<NtfyConfig>,

    /// Telegram configuration
    #[serde(default)]
    pub telegram: Option<TelegramConfig>,

    /// PagerDuty configuration
    #[serde(default)]
    pub pagerduty: Option<PagerDutyConfig>,

    /// Pushover configuration
    #[serde(default)]
    pub pushover: Option<PushoverConfig>,

    /// Syslog configuration
    #[serde(default)]
    pub syslog: Option<SyslogConfig>,

    /// Generic webhook configurations (can have multiple)
    #[serde(default)]
    pub webhooks: Vec<WebhookConfig>,
}

fn default_true() -> bool {
    true
}

fn default_rate_limit() -> u64 {
    5 // 5 seconds between notifications by default
}

impl NotificationConfig {
    /// Create a new empty notification config
    pub fn new() -> Self {
        Self::default()
    }

    /// Load notification config from a TOML file
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_str(&contents)
    }

    /// Load notification config from a TOML string
    pub fn from_str(contents: &str) -> anyhow::Result<Self> {
        let config: NotificationConfig = toml::from_str(contents)?;
        Ok(config)
    }

    /// Check if any notification backend is enabled
    pub fn has_enabled_backends(&self) -> bool {
        if !self.enabled {
            return false;
        }

        let slack_enabled = self.slack.as_ref().map(|c| c.enabled).unwrap_or(false);
        let discord_enabled = self.discord.as_ref().map(|c| c.enabled).unwrap_or(false);
        let ntfy_enabled = self.ntfy.as_ref().map(|c| c.enabled).unwrap_or(false);
        let telegram_enabled = self.telegram.as_ref().map(|c| c.enabled).unwrap_or(false);
        let pagerduty_enabled = self.pagerduty.as_ref().map(|c| c.enabled).unwrap_or(false);
        let pushover_enabled = self.pushover.as_ref().map(|c| c.enabled).unwrap_or(false);
        let syslog_enabled = self.syslog.as_ref().map(|c| c.enabled).unwrap_or(false);
        let webhooks_enabled = self.webhooks.iter().any(|c| c.enabled);

        slack_enabled
            || discord_enabled
            || ntfy_enabled
            || telegram_enabled
            || pagerduty_enabled
            || pushover_enabled
            || syslog_enabled
            || webhooks_enabled
    }

    /// Get list of enabled backend names
    pub fn enabled_backends(&self) -> Vec<&'static str> {
        let mut backends = Vec::new();

        if self.slack.as_ref().map(|c| c.enabled).unwrap_or(false) {
            backends.push("slack");
        }
        if self.discord.as_ref().map(|c| c.enabled).unwrap_or(false) {
            backends.push("discord");
        }
        if self.ntfy.as_ref().map(|c| c.enabled).unwrap_or(false) {
            backends.push("ntfy");
        }
        if self.telegram.as_ref().map(|c| c.enabled).unwrap_or(false) {
            backends.push("telegram");
        }
        if self.pagerduty.as_ref().map(|c| c.enabled).unwrap_or(false) {
            backends.push("pagerduty");
        }
        if self.pushover.as_ref().map(|c| c.enabled).unwrap_or(false) {
            backends.push("pushover");
        }
        if self.syslog.as_ref().map(|c| c.enabled).unwrap_or(false) {
            backends.push("syslog");
        }
        for _ in self.webhooks.iter().filter(|c| c.enabled) {
            backends.push("webhook");
        }

        backends
    }

    /// Generate example TOML config
    pub fn example_toml() -> &'static str {
        r##"# roea-ai Notification Configuration
# =====================================
# Configure alert notifications for protected file access events.
# Enable one or more backends to receive alerts.

# Global notification settings
enabled = true

# Minimum severity level to trigger notifications
# Options: "info", "warning", "alert", "critical"
min_severity = "alert"

# Rate limiting: minimum seconds between notifications (prevents spam)
rate_limit_seconds = 5

# Send a test notification when the agent starts
send_test_on_startup = false

# ============================================================================
# Slack
# ============================================================================
# Webhook-based notifications with rich formatting
# Get a webhook URL from: https://api.slack.com/messaging/webhooks

[slack]
enabled = false
webhook_url = "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
# Optional: Override channel (uses webhook default if not set)
# channel = "#security-alerts"
username = "roea-ai"
icon_emoji = ":shield:"

# ============================================================================
# Discord
# ============================================================================
# Webhook-based notifications with embeds
# Create a webhook in: Server Settings > Integrations > Webhooks

[discord]
enabled = false
webhook_url = "https://discord.com/api/webhooks/YOUR/WEBHOOK/URL"
username = "roea-ai"
# Optional: Custom avatar URL
# avatar_url = "https://example.com/avatar.png"

# ============================================================================
# ntfy.sh
# ============================================================================
# Simple HTTP push notifications
# Use the public server (ntfy.sh) or self-host your own
# Docs: https://ntfy.sh/docs/

[ntfy]
enabled = false
server = "https://ntfy.sh"
topic = "roea-ai-alerts"
# Optional: Authentication for private topics
# token = "tk_your_token_here"
# Or use basic auth:
# username = "your_username"
# password = "your_password"

# ============================================================================
# Telegram
# ============================================================================
# Bot API notifications
# Create a bot with @BotFather and get the token
# Get your chat_id by messaging @userinfobot

[telegram]
enabled = false
bot_token = "YOUR_BOT_TOKEN"
chat_id = "YOUR_CHAT_ID"
parse_mode = "HTML"
# Disable notification sound for less urgent alerts
disable_notification = false

# ============================================================================
# PagerDuty
# ============================================================================
# Incident management integration
# Get a routing key from: Services > Service Directory > [Service] > Integrations

[pagerduty]
enabled = false
routing_key = "YOUR_ROUTING_KEY"
source = "roea-ai"
# Optional: Component and class for categorization
# component = "file-protection"
class = "security"

# ============================================================================
# Pushover
# ============================================================================
# Mobile push notifications
# Get tokens from: https://pushover.net/

[pushover]
enabled = false
token = "YOUR_APP_TOKEN"
user_key = "YOUR_USER_KEY"
# Optional: Send to specific device
# device = "my-phone"
# Optional: Custom notification sound
# sound = "siren"

# ============================================================================
# Syslog
# ============================================================================
# System logging (local or remote)
# Useful for SIEM integration

[syslog]
enabled = false
facility = "local0"
app_name = "roea-ai"
# Optional: Remote syslog server (uses local if not set)
# server = "syslog.example.com"
# port = 514
# use_tcp = false

# ============================================================================
# Generic Webhooks
# ============================================================================
# Custom HTTP webhooks for integration with other systems
# Can define multiple webhooks

[[webhooks]]
enabled = false
url = "https://your-webhook-endpoint.com/alerts"
method = "POST"
# Optional: Custom headers
# [webhooks.headers]
# X-Custom-Header = "value"
# Optional: Bearer token authentication
# auth_token = "your_token"

# Example: Second webhook for a different system
# [[webhooks]]
# enabled = false
# url = "https://another-endpoint.com/api/alerts"
# method = "POST"
# auth_token = "another_token"
"##
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NotificationConfig::default();
        assert!(config.enabled);
        assert!(!config.has_enabled_backends());
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
enabled = true
min_severity = "warning"

[slack]
enabled = true
webhook_url = "https://hooks.slack.com/test"
"#;

        let config = NotificationConfig::from_str(toml).unwrap();
        assert!(config.enabled);
        assert!(config.has_enabled_backends());
        assert_eq!(config.enabled_backends(), vec!["slack"]);
    }

    #[test]
    fn test_multiple_backends() {
        let toml = r#"
enabled = true

[slack]
enabled = true
webhook_url = "https://hooks.slack.com/test"

[discord]
enabled = true
webhook_url = "https://discord.com/api/webhooks/test"

[[webhooks]]
enabled = true
url = "https://example.com/webhook"
"#;

        let config = NotificationConfig::from_str(toml).unwrap();
        let backends = config.enabled_backends();
        assert!(backends.contains(&"slack"));
        assert!(backends.contains(&"discord"));
        assert!(backends.contains(&"webhook"));
    }
}
