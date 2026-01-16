//! File Protection Module
//!
//! This module provides configuration and monitoring for protected files and directories.
//! When AI agents access protected paths, alerts are generated and notifications are sent.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::notification::{AlertNotification, NotificationConfig, NotificationManager};

/// Default protected files that should trigger alerts
pub const DEFAULT_PROTECTED_FILES: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    "/etc/sudoers",
    "/etc/ssh/sshd_config",
    "/etc/hosts",
    "/etc/resolv.conf",
    "/etc/crontab",
    "/root/.ssh/authorized_keys",
    "/root/.ssh/id_rsa",
    "/root/.ssh/id_ed25519",
    "/root/.bashrc",
    "/root/.bash_history",
    "/var/log/auth.log",
    "/var/log/secure",
];

/// Default protected directories (any access under these triggers alert)
pub const DEFAULT_PROTECTED_DIRS: &[&str] = &[
    "/etc/ssh",
    "/root/.ssh",
    "/root/.gnupg",
    "/etc/pam.d",
    "/etc/security",
];

/// Protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    /// List of specific files to protect
    #[serde(default)]
    pub files: Vec<PathBuf>,

    /// List of directories to protect (all files under these are protected)
    #[serde(default)]
    pub directories: Vec<PathBuf>,

    /// Glob patterns for files to protect
    #[serde(default)]
    pub patterns: Vec<String>,

    /// Include default system sensitive files
    #[serde(default = "default_true")]
    pub include_defaults: bool,

    /// Severity level for alerts (info, warning, critical)
    #[serde(default = "default_severity")]
    pub alert_severity: String,

    /// Enable prevention mode (block access instead of just alerting)
    /// Note: Prevention mode requires additional privileges and kernel support
    #[serde(default)]
    pub prevention_mode: bool,

    /// Log all protected file accesses to this file
    #[serde(default)]
    pub log_file: Option<PathBuf>,

    /// Notification configuration
    #[serde(default)]
    pub notifications: Option<NotificationConfig>,

    /// Cached set of protected paths for quick lookup
    #[serde(skip)]
    protected_set: HashSet<PathBuf>,

    /// Cached set of protected directories for prefix matching
    #[serde(skip)]
    protected_dirs_set: HashSet<PathBuf>,
}

fn default_true() -> bool {
    true
}

fn default_severity() -> String {
    "critical".to_string()
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        let mut config = Self {
            files: Vec::new(),
            directories: Vec::new(),
            patterns: Vec::new(),
            include_defaults: true,
            alert_severity: "critical".to_string(),
            prevention_mode: false,
            log_file: None,
            notifications: None,
            protected_set: HashSet::new(),
            protected_dirs_set: HashSet::new(),
        };
        config.rebuild_cache();
        config
    }
}

impl ProtectionConfig {
    /// Create a new protection config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load protection config from a TOML file
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let mut config: ProtectionConfig = toml::from_str(&contents)?;
        config.rebuild_cache();
        Ok(config)
    }

    /// Load protection config from a TOML string
    pub fn from_str(contents: &str) -> anyhow::Result<Self> {
        let mut config: ProtectionConfig = toml::from_str(contents)?;
        config.rebuild_cache();
        Ok(config)
    }

    /// Rebuild the internal cache for quick lookups
    pub fn rebuild_cache(&mut self) {
        self.protected_set.clear();
        self.protected_dirs_set.clear();

        // Add user-specified files
        for file in &self.files {
            self.protected_set.insert(file.clone());
        }

        // Add user-specified directories
        for dir in &self.directories {
            self.protected_dirs_set.insert(dir.clone());
        }

        // Add defaults if enabled
        if self.include_defaults {
            for file in DEFAULT_PROTECTED_FILES {
                self.protected_set.insert(PathBuf::from(file));
            }
            for dir in DEFAULT_PROTECTED_DIRS {
                self.protected_dirs_set.insert(PathBuf::from(dir));
            }
        }
    }

    /// Check if a path is protected
    pub fn is_protected(&self, path: &str) -> bool {
        let path = Path::new(path);

        // Exact file match
        if self.protected_set.contains(path) {
            return true;
        }

        // Directory prefix match
        for dir in &self.protected_dirs_set {
            if path.starts_with(dir) {
                return true;
            }
        }

        // Pattern matching
        for pattern in &self.patterns {
            if let Ok(glob) = glob::Pattern::new(pattern) {
                if glob.matches_path(path) {
                    return true;
                }
            }
        }

        false
    }

    /// Add a file to the protection list
    pub fn add_file(&mut self, path: PathBuf) {
        self.files.push(path.clone());
        self.protected_set.insert(path);
    }

    /// Add a directory to the protection list
    pub fn add_directory(&mut self, path: PathBuf) {
        self.directories.push(path.clone());
        self.protected_dirs_set.insert(path);
    }

    /// Add a pattern to the protection list
    pub fn add_pattern(&mut self, pattern: String) {
        self.patterns.push(pattern);
    }

    /// Get count of protected items
    pub fn protected_count(&self) -> usize {
        self.protected_set.len() + self.protected_dirs_set.len() + self.patterns.len()
    }

    /// Generate example TOML config
    pub fn example_toml() -> &'static str {
        r#"# roea-ai Protection Configuration
# This file defines which files and directories should trigger alerts
# when accessed by AI coding agents.

# Include default system sensitive files (/etc/passwd, /etc/shadow, etc.)
include_defaults = true

# Alert severity: "info", "warning", or "critical"
alert_severity = "critical"

# Prevention mode (requires additional privileges)
# When enabled, attempts to block access rather than just alerting
prevention_mode = false

# Optional: Log all protected file accesses to a file
# log_file = "/var/log/roea-protection.log"

# Specific files to protect
files = [
    "/home/user/.ssh/id_rsa",
    "/home/user/.ssh/id_ed25519",
    "/home/user/.aws/credentials",
    "/home/user/.config/gh/hosts.yml",
    "~/.npmrc",
    "~/.pypirc",
]

# Directories to protect (all files under these trigger alerts)
directories = [
    "/home/user/.ssh",
    "/home/user/.gnupg",
    "/home/user/.aws",
    "/home/user/secrets",
]

# Glob patterns for protected files
patterns = [
    "**/.env",
    "**/.env.*",
    "**/secrets.yaml",
    "**/secrets.json",
    "**/*.pem",
    "**/*.key",
    "**/id_rsa*",
    "**/id_ed25519*",
    "**/.git-credentials",
]

# ============================================================================
# Notifications (optional)
# ============================================================================
# Configure where to send alerts when protected files are accessed.
# For a full example, see examples/notify.toml

[notifications]
enabled = true
min_severity = "alert"
rate_limit_seconds = 5

# Example: ntfy.sh (mobile push notifications)
# [notifications.ntfy]
# enabled = true
# server = "https://ntfy.sh"
# topic = "roea-ai-alerts"

# Example: Slack
# [notifications.slack]
# enabled = true
# webhook_url = "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

# Example: Discord
# [notifications.discord]
# enabled = true
# webhook_url = "https://discord.com/api/webhooks/YOUR/WEBHOOK/URL"

# Example: Syslog (for SIEM integration)
# [notifications.syslog]
# enabled = true
# facility = "local0"
# app_name = "roea-ai"
"#
    }
}

/// Protection event for logging and alerting
#[derive(Debug, Clone, Serialize)]
pub struct ProtectionEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub pid: u32,
    pub process_name: String,
    pub path: String,
    pub operation: String,
    pub severity: String,
    pub blocked: bool,
}

impl ProtectionEvent {
    pub fn new(
        pid: u32,
        process_name: String,
        path: String,
        operation: String,
        severity: String,
        blocked: bool,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            pid,
            process_name,
            path,
            operation,
            severity,
            blocked,
        }
    }

    /// Format as log line
    pub fn to_log_line(&self) -> String {
        let status = if self.blocked { "BLOCKED" } else { "ALERT" };
        format!(
            "[{}] {} {} PID:{} {} {} {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            status,
            self.severity.to_uppercase(),
            self.pid,
            self.process_name,
            self.operation,
            self.path
        )
    }
}

/// Protection service that handles events and notifications
pub struct ProtectionService {
    /// Protection configuration
    config: ProtectionConfig,
    /// Notification manager
    notification_manager: Option<Arc<NotificationManager>>,
}

impl ProtectionService {
    /// Create a new protection service with the given configuration
    pub fn new(config: ProtectionConfig) -> Self {
        let notification_manager = config.notifications.as_ref().map(|nc| {
            Arc::new(NotificationManager::new(nc.clone()))
        });

        Self {
            config,
            notification_manager,
        }
    }

    /// Create a new protection service with a separate notification manager
    pub fn with_notification_manager(
        config: ProtectionConfig,
        notification_manager: Arc<NotificationManager>,
    ) -> Self {
        Self {
            config,
            notification_manager: Some(notification_manager),
        }
    }

    /// Get the protection configuration
    pub fn config(&self) -> &ProtectionConfig {
        &self.config
    }

    /// Check if a path is protected
    pub fn is_protected(&self, path: &str) -> bool {
        self.config.is_protected(path)
    }

    /// Handle a protection event and send notifications
    pub async fn handle_event(&self, event: &ProtectionEvent) -> anyhow::Result<()> {
        // Log the event
        tracing::warn!(
            target: "protection",
            pid = event.pid,
            process = %event.process_name,
            path = %event.path,
            operation = %event.operation,
            severity = %event.severity,
            blocked = event.blocked,
            "Protected file access detected"
        );

        // Write to log file if configured
        if let Some(ref log_path) = self.config.log_file {
            if let Err(e) = self.write_to_log(log_path, event) {
                tracing::error!("Failed to write to protection log: {}", e);
            }
        }

        // Send notifications
        if let Some(ref nm) = self.notification_manager {
            let notification = AlertNotification::from_protection_event(event);
            if let Err(e) = nm.notify(&notification).await {
                tracing::error!("Failed to send notification: {}", e);
            }
        }

        Ok(())
    }

    /// Write event to log file
    fn write_to_log(&self, path: &Path, event: &ProtectionEvent) -> std::io::Result<()> {
        use std::io::Write;

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        writeln!(file, "{}", event.to_log_line())?;
        Ok(())
    }

    /// Send a test notification
    pub async fn send_test_notification(&self) -> anyhow::Result<()> {
        if let Some(ref nm) = self.notification_manager {
            nm.send_test().await.map_err(|e| anyhow::anyhow!(e))
        } else {
            tracing::warn!("No notification manager configured");
            Ok(())
        }
    }

    /// Check if notifications are enabled
    pub fn has_notifications(&self) -> bool {
        self.notification_manager
            .as_ref()
            .map(|nm| nm.is_enabled())
            .unwrap_or(false)
    }

    /// Get list of enabled notification backends
    pub fn notification_backends(&self) -> Vec<&'static str> {
        self.notification_manager
            .as_ref()
            .map(|nm| nm.enabled_backends())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_protection() {
        let config = ProtectionConfig::default();

        // Should protect default files
        assert!(config.is_protected("/etc/passwd"));
        assert!(config.is_protected("/etc/shadow"));
        assert!(config.is_protected("/etc/ssh/sshd_config"));

        // Should protect files under protected directories
        assert!(config.is_protected("/root/.ssh/id_rsa"));
        assert!(config.is_protected("/root/.ssh/known_hosts"));

        // Should not protect random files
        assert!(!config.is_protected("/tmp/test.txt"));
        assert!(!config.is_protected("/home/user/code.rs"));
    }

    #[test]
    fn test_custom_protection() {
        let mut config = ProtectionConfig {
            include_defaults: false,
            ..Default::default()
        };
        config.rebuild_cache();

        // Should not protect defaults when disabled
        assert!(!config.is_protected("/etc/passwd"));

        // Add custom protection
        config.add_file(PathBuf::from("/custom/secret.txt"));
        config.add_directory(PathBuf::from("/custom/secrets"));

        assert!(config.is_protected("/custom/secret.txt"));
        assert!(config.is_protected("/custom/secrets/key.pem"));
        assert!(!config.is_protected("/custom/other.txt"));
    }

    #[test]
    fn test_pattern_protection() {
        let mut config = ProtectionConfig {
            include_defaults: false,
            patterns: vec!["**/.env".to_string(), "**/*.key".to_string()],
            ..Default::default()
        };
        config.rebuild_cache();

        assert!(config.is_protected("/app/.env"));
        assert!(config.is_protected("/home/user/project/.env"));
        assert!(config.is_protected("/secrets/private.key"));
        assert!(!config.is_protected("/app/.envrc"));
    }

    #[test]
    fn test_toml_parsing() {
        let toml = r#"
include_defaults = false
alert_severity = "warning"
files = ["/custom/file.txt"]
directories = ["/custom/dir"]
patterns = ["**/*.secret"]
"#;

        let config = ProtectionConfig::from_str(toml).unwrap();
        assert!(!config.include_defaults);
        assert_eq!(config.alert_severity, "warning");
        assert!(config.is_protected("/custom/file.txt"));
        assert!(config.is_protected("/custom/dir/nested/file"));
        assert!(config.is_protected("/any/path/test.secret"));
    }
}
