//! Notification Backend Implementations
//!
//! This module contains implementations for all supported notification backends.

use super::{AlertNotification, NotificationBackend, NotificationError, NotificationResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

// ============================================================================
// Slack Backend
// ============================================================================

/// Slack notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Webhook URL
    pub webhook_url: String,
    /// Channel override (optional, uses webhook default if not set)
    #[serde(default)]
    pub channel: Option<String>,
    /// Username override
    #[serde(default = "default_slack_username")]
    pub username: String,
    /// Icon emoji
    #[serde(default = "default_slack_icon")]
    pub icon_emoji: String,
    /// Enable/disable this backend
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_slack_username() -> String {
    "roea-ai".to_string()
}

fn default_slack_icon() -> String {
    ":shield:".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for SlackConfig {
    fn default() -> Self {
        Self {
            webhook_url: String::new(),
            channel: None,
            username: default_slack_username(),
            icon_emoji: default_slack_icon(),
            enabled: false,
        }
    }
}

/// Slack notification backend
pub struct SlackBackend {
    config: SlackConfig,
    client: reqwest::Client,
}

impl SlackBackend {
    pub fn new(config: SlackConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn build_payload(&self, notification: &AlertNotification) -> serde_json::Value {
        let mut payload = serde_json::json!({
            "username": self.config.username,
            "icon_emoji": self.config.icon_emoji,
            "attachments": [{
                "color": notification.severity.color(),
                "title": notification.title,
                "text": notification.message,
                "fields": [
                    {
                        "title": "Severity",
                        "value": notification.severity.to_string().to_uppercase(),
                        "short": true
                    },
                    {
                        "title": "Host",
                        "value": notification.hostname,
                        "short": true
                    }
                ],
                "footer": "roea-ai",
                "ts": notification.timestamp.timestamp()
            }]
        });

        // Add process info if available
        if !notification.process_name.is_empty() {
            let attachments = payload["attachments"].as_array_mut().unwrap();
            let fields = attachments[0]["fields"].as_array_mut().unwrap();
            fields.push(serde_json::json!({
                "title": "Process",
                "value": format!("{} (PID: {})", notification.process_name, notification.pid),
                "short": true
            }));
        }

        // Add path if available
        if let Some(ref path) = notification.path {
            let attachments = payload["attachments"].as_array_mut().unwrap();
            let fields = attachments[0]["fields"].as_array_mut().unwrap();
            fields.push(serde_json::json!({
                "title": "Path",
                "value": path,
                "short": false
            }));
        }

        // Add operation if available
        if let Some(ref op) = notification.operation {
            let attachments = payload["attachments"].as_array_mut().unwrap();
            let fields = attachments[0]["fields"].as_array_mut().unwrap();
            fields.push(serde_json::json!({
                "title": "Operation",
                "value": op,
                "short": true
            }));
        }

        // Add blocked status
        if notification.blocked {
            let attachments = payload["attachments"].as_array_mut().unwrap();
            let fields = attachments[0]["fields"].as_array_mut().unwrap();
            fields.push(serde_json::json!({
                "title": "Status",
                "value": "BLOCKED",
                "short": true
            }));
        }

        // Add channel if specified
        if let Some(ref channel) = self.config.channel {
            payload["channel"] = serde_json::Value::String(channel.clone());
        }

        payload
    }
}

#[async_trait]
impl NotificationBackend for SlackBackend {
    fn name(&self) -> &'static str {
        "slack"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.webhook_url.is_empty()
    }

    async fn send(&self, notification: &AlertNotification) -> NotificationResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let payload = self.build_payload(notification);
        debug!("Sending Slack notification: {:?}", payload);

        let response = self
            .client
            .post(&self.config.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Slack notification sent successfully");
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Slack notification failed: {} - {}", status, body);
            Err(NotificationError::ConfigError(format!(
                "Slack API error: {} - {}",
                status, body
            )))
        }
    }
}

// ============================================================================
// Discord Backend
// ============================================================================

/// Discord notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    /// Webhook URL
    pub webhook_url: String,
    /// Username override
    #[serde(default = "default_discord_username")]
    pub username: String,
    /// Avatar URL
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// Enable/disable this backend
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_discord_username() -> String {
    "roea-ai".to_string()
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            webhook_url: String::new(),
            username: default_discord_username(),
            avatar_url: None,
            enabled: false,
        }
    }
}

/// Discord notification backend
pub struct DiscordBackend {
    config: DiscordConfig,
    client: reqwest::Client,
}

impl DiscordBackend {
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn build_payload(&self, notification: &AlertNotification) -> serde_json::Value {
        let mut fields = vec![
            serde_json::json!({
                "name": "Severity",
                "value": notification.severity.to_string().to_uppercase(),
                "inline": true
            }),
            serde_json::json!({
                "name": "Host",
                "value": notification.hostname,
                "inline": true
            }),
        ];

        if !notification.process_name.is_empty() {
            fields.push(serde_json::json!({
                "name": "Process",
                "value": format!("{} (PID: {})", notification.process_name, notification.pid),
                "inline": true
            }));
        }

        if let Some(ref path) = notification.path {
            fields.push(serde_json::json!({
                "name": "Path",
                "value": format!("`{}`", path),
                "inline": false
            }));
        }

        if let Some(ref op) = notification.operation {
            fields.push(serde_json::json!({
                "name": "Operation",
                "value": op,
                "inline": true
            }));
        }

        if notification.blocked {
            fields.push(serde_json::json!({
                "name": "Status",
                "value": "BLOCKED",
                "inline": true
            }));
        }

        let mut payload = serde_json::json!({
            "username": self.config.username,
            "embeds": [{
                "title": notification.title,
                "description": notification.message,
                "color": notification.severity.color_int(),
                "fields": fields,
                "timestamp": notification.timestamp.to_rfc3339(),
                "footer": {
                    "text": "roea-ai"
                }
            }]
        });

        if let Some(ref avatar_url) = self.config.avatar_url {
            payload["avatar_url"] = serde_json::Value::String(avatar_url.clone());
        }

        payload
    }
}

#[async_trait]
impl NotificationBackend for DiscordBackend {
    fn name(&self) -> &'static str {
        "discord"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.webhook_url.is_empty()
    }

    async fn send(&self, notification: &AlertNotification) -> NotificationResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let payload = self.build_payload(notification);
        debug!("Sending Discord notification: {:?}", payload);

        let response = self
            .client
            .post(&self.config.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Discord notification sent successfully");
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Discord notification failed: {} - {}", status, body);
            Err(NotificationError::ConfigError(format!(
                "Discord API error: {} - {}",
                status, body
            )))
        }
    }
}

// ============================================================================
// ntfy.sh Backend
// ============================================================================

/// ntfy.sh notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtfyConfig {
    /// Server URL (default: https://ntfy.sh)
    #[serde(default = "default_ntfy_server")]
    pub server: String,
    /// Topic name
    pub topic: String,
    /// Authentication token (optional)
    #[serde(default)]
    pub token: Option<String>,
    /// Username for basic auth (optional)
    #[serde(default)]
    pub username: Option<String>,
    /// Password for basic auth (optional)
    #[serde(default)]
    pub password: Option<String>,
    /// Enable/disable this backend
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_ntfy_server() -> String {
    "https://ntfy.sh".to_string()
}

impl Default for NtfyConfig {
    fn default() -> Self {
        Self {
            server: default_ntfy_server(),
            topic: String::new(),
            token: None,
            username: None,
            password: None,
            enabled: false,
        }
    }
}

/// ntfy.sh notification backend
pub struct NtfyBackend {
    config: NtfyConfig,
    client: reqwest::Client,
}

impl NtfyBackend {
    pub fn new(config: NtfyConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationBackend for NtfyBackend {
    fn name(&self) -> &'static str {
        "ntfy"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.topic.is_empty()
    }

    async fn send(&self, notification: &AlertNotification) -> NotificationResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let url = format!("{}/{}", self.config.server.trim_end_matches('/'), self.config.topic);

        let mut request = self
            .client
            .post(&url)
            .header("Title", &notification.title)
            .header("Priority", notification.severity.ntfy_priority().to_string())
            .header("Tags", format!("roea-ai,{}", notification.severity));

        // Add authentication if configured
        if let Some(ref token) = self.config.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        } else if let (Some(ref username), Some(ref password)) =
            (&self.config.username, &self.config.password)
        {
            request = request.basic_auth(username, Some(password));
        }

        // Add action URL for path if available
        if let Some(ref path) = notification.path {
            request = request.header("X-Path", path);
        }

        // Build the message body
        let mut body = notification.message.clone();
        if !notification.process_name.is_empty() {
            body.push_str(&format!(
                "\n\nProcess: {} (PID: {})",
                notification.process_name, notification.pid
            ));
        }
        if let Some(ref path) = notification.path {
            body.push_str(&format!("\nPath: {}", path));
        }
        if notification.blocked {
            body.push_str("\nStatus: BLOCKED");
        }
        body.push_str(&format!("\nHost: {}", notification.hostname));

        debug!("Sending ntfy notification to {}", url);

        let response = request.body(body).send().await?;

        if response.status().is_success() {
            info!("ntfy notification sent successfully");
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("ntfy notification failed: {} - {}", status, body);
            Err(NotificationError::ConfigError(format!(
                "ntfy error: {} - {}",
                status, body
            )))
        }
    }
}

// ============================================================================
// Telegram Backend
// ============================================================================

/// Telegram notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    /// Bot token from @BotFather
    pub bot_token: String,
    /// Chat ID to send messages to
    pub chat_id: String,
    /// Parse mode (HTML or Markdown)
    #[serde(default = "default_telegram_parse_mode")]
    pub parse_mode: String,
    /// Disable notification sound
    #[serde(default)]
    pub disable_notification: bool,
    /// Enable/disable this backend
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_telegram_parse_mode() -> String {
    "HTML".to_string()
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            chat_id: String::new(),
            parse_mode: default_telegram_parse_mode(),
            disable_notification: false,
            enabled: false,
        }
    }
}

/// Telegram notification backend
pub struct TelegramBackend {
    config: TelegramConfig,
    client: reqwest::Client,
}

impl TelegramBackend {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn build_message(&self, notification: &AlertNotification) -> String {
        let mut msg = format!(
            "<b>{} {}</b>\n\n{}",
            notification.severity.emoji(),
            html_escape(&notification.title),
            html_escape(&notification.message)
        );

        if !notification.process_name.is_empty() {
            msg.push_str(&format!(
                "\n\n<b>Process:</b> {} (PID: {})",
                html_escape(&notification.process_name),
                notification.pid
            ));
        }

        if let Some(ref path) = notification.path {
            msg.push_str(&format!("\n<b>Path:</b> <code>{}</code>", html_escape(path)));
        }

        if let Some(ref op) = notification.operation {
            msg.push_str(&format!("\n<b>Operation:</b> {}", html_escape(op)));
        }

        if notification.blocked {
            msg.push_str("\n<b>Status:</b> BLOCKED");
        }

        msg.push_str(&format!("\n<b>Host:</b> {}", html_escape(&notification.hostname)));
        msg.push_str(&format!(
            "\n<i>{}</i>",
            notification.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        msg
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[async_trait]
impl NotificationBackend for TelegramBackend {
    fn name(&self) -> &'static str {
        "telegram"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.bot_token.is_empty() && !self.config.chat_id.is_empty()
    }

    async fn send(&self, notification: &AlertNotification) -> NotificationResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.config.bot_token
        );

        let message = self.build_message(notification);

        let payload = serde_json::json!({
            "chat_id": self.config.chat_id,
            "text": message,
            "parse_mode": self.config.parse_mode,
            "disable_notification": self.config.disable_notification
        });

        debug!("Sending Telegram notification");

        let response = self.client.post(&url).json(&payload).send().await?;

        if response.status().is_success() {
            info!("Telegram notification sent successfully");
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Telegram notification failed: {} - {}", status, body);
            Err(NotificationError::ConfigError(format!(
                "Telegram API error: {} - {}",
                status, body
            )))
        }
    }
}

// ============================================================================
// PagerDuty Backend
// ============================================================================

/// PagerDuty notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyConfig {
    /// Integration/routing key
    pub routing_key: String,
    /// Service name
    #[serde(default = "default_pagerduty_source")]
    pub source: String,
    /// Component name
    #[serde(default)]
    pub component: Option<String>,
    /// Class/type of event
    #[serde(default = "default_pagerduty_class")]
    pub class: String,
    /// Enable/disable this backend
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_pagerduty_source() -> String {
    "roea-ai".to_string()
}

fn default_pagerduty_class() -> String {
    "security".to_string()
}

impl Default for PagerDutyConfig {
    fn default() -> Self {
        Self {
            routing_key: String::new(),
            source: default_pagerduty_source(),
            component: None,
            class: default_pagerduty_class(),
            enabled: false,
        }
    }
}

/// PagerDuty notification backend
pub struct PagerDutyBackend {
    config: PagerDutyConfig,
    client: reqwest::Client,
}

impl PagerDutyBackend {
    pub fn new(config: PagerDutyConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationBackend for PagerDutyBackend {
    fn name(&self) -> &'static str {
        "pagerduty"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.routing_key.is_empty()
    }

    async fn send(&self, notification: &AlertNotification) -> NotificationResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let url = "https://events.pagerduty.com/v2/enqueue";

        let mut custom_details = serde_json::json!({
            "pid": notification.pid,
            "process_name": notification.process_name,
            "blocked": notification.blocked,
            "hostname": notification.hostname
        });

        if let Some(ref path) = notification.path {
            custom_details["path"] = serde_json::Value::String(path.clone());
        }

        if let Some(ref op) = notification.operation {
            custom_details["operation"] = serde_json::Value::String(op.clone());
        }

        for (key, value) in &notification.metadata {
            custom_details[key] = serde_json::Value::String(value.clone());
        }

        let mut payload = serde_json::json!({
            "routing_key": self.config.routing_key,
            "event_action": "trigger",
            "dedup_key": notification.id,
            "payload": {
                "summary": notification.title,
                "source": self.config.source,
                "severity": notification.severity.pagerduty_severity(),
                "timestamp": notification.timestamp.to_rfc3339(),
                "class": self.config.class,
                "custom_details": custom_details
            }
        });

        if let Some(ref component) = self.config.component {
            payload["payload"]["component"] = serde_json::Value::String(component.clone());
        }

        debug!("Sending PagerDuty notification");

        let response = self.client.post(url).json(&payload).send().await?;

        if response.status().is_success() {
            info!("PagerDuty notification sent successfully");
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("PagerDuty notification failed: {} - {}", status, body);
            Err(NotificationError::ConfigError(format!(
                "PagerDuty API error: {} - {}",
                status, body
            )))
        }
    }
}

// ============================================================================
// Pushover Backend
// ============================================================================

/// Pushover notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushoverConfig {
    /// Application API token
    pub token: String,
    /// User/group key
    pub user_key: String,
    /// Device name (optional, sends to all devices if not set)
    #[serde(default)]
    pub device: Option<String>,
    /// Notification sound
    #[serde(default)]
    pub sound: Option<String>,
    /// Enable/disable this backend
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for PushoverConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            user_key: String::new(),
            device: None,
            sound: None,
            enabled: false,
        }
    }
}

/// Pushover notification backend
pub struct PushoverBackend {
    config: PushoverConfig,
    client: reqwest::Client,
}

impl PushoverBackend {
    pub fn new(config: PushoverConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn get_priority(&self, notification: &AlertNotification) -> i8 {
        match notification.severity {
            super::NotificationSeverity::Info => -1,
            super::NotificationSeverity::Warning => 0,
            super::NotificationSeverity::Alert => 1,
            super::NotificationSeverity::Critical => 2,
        }
    }
}

#[async_trait]
impl NotificationBackend for PushoverBackend {
    fn name(&self) -> &'static str {
        "pushover"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.token.is_empty() && !self.config.user_key.is_empty()
    }

    async fn send(&self, notification: &AlertNotification) -> NotificationResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let url = "https://api.pushover.net/1/messages.json";

        let priority = self.get_priority(notification);

        let mut form = vec![
            ("token", self.config.token.clone()),
            ("user", self.config.user_key.clone()),
            ("title", notification.title.clone()),
            ("message", notification.to_plain_text()),
            ("priority", priority.to_string()),
            ("timestamp", notification.timestamp.timestamp().to_string()),
        ];

        // Add retry and expire for emergency priority
        if priority == 2 {
            form.push(("retry", "60".to_string()));
            form.push(("expire", "3600".to_string()));
        }

        if let Some(ref device) = self.config.device {
            form.push(("device", device.clone()));
        }

        if let Some(ref sound) = self.config.sound {
            form.push(("sound", sound.clone()));
        }

        debug!("Sending Pushover notification");

        let response = self.client.post(url).form(&form).send().await?;

        if response.status().is_success() {
            info!("Pushover notification sent successfully");
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Pushover notification failed: {} - {}", status, body);
            Err(NotificationError::ConfigError(format!(
                "Pushover API error: {} - {}",
                status, body
            )))
        }
    }
}

// ============================================================================
// Syslog Backend
// ============================================================================

/// Syslog notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyslogConfig {
    /// Syslog facility
    #[serde(default = "default_syslog_facility")]
    pub facility: String,
    /// Application name
    #[serde(default = "default_syslog_app_name")]
    pub app_name: String,
    /// Remote syslog server (optional, uses local if not set)
    #[serde(default)]
    pub server: Option<String>,
    /// Remote syslog port
    #[serde(default = "default_syslog_port")]
    pub port: u16,
    /// Use TCP instead of UDP for remote syslog
    #[serde(default)]
    pub use_tcp: bool,
    /// Enable/disable this backend
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_syslog_facility() -> String {
    "local0".to_string()
}

fn default_syslog_app_name() -> String {
    "roea-ai".to_string()
}

fn default_syslog_port() -> u16 {
    514
}

impl Default for SyslogConfig {
    fn default() -> Self {
        Self {
            facility: default_syslog_facility(),
            app_name: default_syslog_app_name(),
            server: None,
            port: default_syslog_port(),
            use_tcp: false,
            enabled: false,
        }
    }
}

/// Syslog notification backend
pub struct SyslogBackend {
    config: SyslogConfig,
    logger: Option<std::sync::Mutex<syslog::Logger<syslog::LoggerBackend, syslog::Formatter3164>>>,
}

impl SyslogBackend {
    pub fn new(config: SyslogConfig) -> Self {
        let logger = if config.enabled {
            Self::create_logger(&config).ok().map(std::sync::Mutex::new)
        } else {
            None
        };

        Self { config, logger }
    }

    fn create_logger(
        config: &SyslogConfig,
    ) -> Result<syslog::Logger<syslog::LoggerBackend, syslog::Formatter3164>, NotificationError>
    {
        let facility = match config.facility.to_lowercase().as_str() {
            "kern" => syslog::Facility::LOG_KERN,
            "user" => syslog::Facility::LOG_USER,
            "mail" => syslog::Facility::LOG_MAIL,
            "daemon" => syslog::Facility::LOG_DAEMON,
            "auth" => syslog::Facility::LOG_AUTH,
            "syslog" => syslog::Facility::LOG_SYSLOG,
            "lpr" => syslog::Facility::LOG_LPR,
            "news" => syslog::Facility::LOG_NEWS,
            "uucp" => syslog::Facility::LOG_UUCP,
            "cron" => syslog::Facility::LOG_CRON,
            "local0" => syslog::Facility::LOG_LOCAL0,
            "local1" => syslog::Facility::LOG_LOCAL1,
            "local2" => syslog::Facility::LOG_LOCAL2,
            "local3" => syslog::Facility::LOG_LOCAL3,
            "local4" => syslog::Facility::LOG_LOCAL4,
            "local5" => syslog::Facility::LOG_LOCAL5,
            "local6" => syslog::Facility::LOG_LOCAL6,
            "local7" => syslog::Facility::LOG_LOCAL7,
            _ => syslog::Facility::LOG_LOCAL0,
        };

        let formatter = syslog::Formatter3164 {
            facility,
            hostname: Some(hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "localhost".to_string())),
            process: config.app_name.clone(),
            pid: std::process::id(),
        };

        // Try to connect to syslog
        if let Some(ref server) = config.server {
            // Remote syslog
            let addr = format!("{}:{}", server, config.port);
            if config.use_tcp {
                syslog::tcp(formatter, &addr)
                    .map_err(|e| NotificationError::SyslogError(e.to_string()))
            } else {
                syslog::udp(formatter, "0.0.0.0:0", &addr)
                    .map_err(|e| NotificationError::SyslogError(e.to_string()))
            }
        } else {
            // Local syslog (Unix socket)
            syslog::unix(formatter).map_err(|e| NotificationError::SyslogError(e.to_string()))
        }
    }

    fn get_severity(&self, notification: &AlertNotification) -> syslog::Severity {
        match notification.severity {
            super::NotificationSeverity::Info => syslog::Severity::LOG_INFO,
            super::NotificationSeverity::Warning => syslog::Severity::LOG_WARNING,
            super::NotificationSeverity::Alert => syslog::Severity::LOG_ERR,
            super::NotificationSeverity::Critical => syslog::Severity::LOG_CRIT,
        }
    }
}

#[async_trait]
impl NotificationBackend for SyslogBackend {
    fn name(&self) -> &'static str {
        "syslog"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled && self.logger.is_some()
    }

    async fn send(&self, notification: &AlertNotification) -> NotificationResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let Some(ref logger_mutex) = self.logger else {
            return Err(NotificationError::SyslogError(
                "Syslog not initialized".to_string(),
            ));
        };

        let severity = self.get_severity(notification);
        let message = format!(
            "{} [{}] pid={} process={} path={} op={} blocked={} host={}",
            notification.title,
            notification.severity.to_string().to_uppercase(),
            notification.pid,
            notification.process_name,
            notification.path.as_deref().unwrap_or("-"),
            notification.operation.as_deref().unwrap_or("-"),
            notification.blocked,
            notification.hostname
        );

        debug!("Sending syslog message: {}", message);

        // Note: syslog crate's log methods are synchronous
        let mut logger = logger_mutex
            .lock()
            .map_err(|e| NotificationError::SyslogError(format!("Failed to lock logger: {}", e)))?;

        let result = match severity {
            syslog::Severity::LOG_CRIT => logger.crit(&message),
            syslog::Severity::LOG_ERR => logger.err(&message),
            syslog::Severity::LOG_WARNING => logger.warning(&message),
            syslog::Severity::LOG_INFO => logger.info(&message),
            _ => logger.info(&message),
        };

        result.map_err(|e| NotificationError::SyslogError(e.to_string()))?;
        info!("Syslog notification sent successfully");
        Ok(())
    }
}

// ============================================================================
// Generic Webhook Backend
// ============================================================================

/// Generic webhook notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Webhook URL
    pub url: String,
    /// HTTP method (POST, PUT, PATCH)
    #[serde(default = "default_webhook_method")]
    pub method: String,
    /// Additional headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    /// Authentication token (sent as Bearer token)
    #[serde(default)]
    pub auth_token: Option<String>,
    /// Custom payload template (uses default JSON if not set)
    #[serde(default)]
    pub payload_template: Option<String>,
    /// Enable/disable this backend
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_webhook_method() -> String {
    "POST".to_string()
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: default_webhook_method(),
            headers: std::collections::HashMap::new(),
            auth_token: None,
            payload_template: None,
            enabled: false,
        }
    }
}

/// Generic webhook notification backend
pub struct WebhookBackend {
    config: WebhookConfig,
    client: reqwest::Client,
}

impl WebhookBackend {
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationBackend for WebhookBackend {
    fn name(&self) -> &'static str {
        "webhook"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.url.is_empty()
    }

    async fn send(&self, notification: &AlertNotification) -> NotificationResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let method = match self.config.method.to_uppercase().as_str() {
            "PUT" => reqwest::Method::PUT,
            "PATCH" => reqwest::Method::PATCH,
            _ => reqwest::Method::POST,
        };

        let mut request = self.client.request(method, &self.config.url);

        // Add custom headers
        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        // Add auth token if configured
        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        // Build payload
        let payload = serde_json::json!({
            "id": notification.id,
            "timestamp": notification.timestamp.to_rfc3339(),
            "severity": notification.severity.to_string(),
            "title": notification.title,
            "message": notification.message,
            "pid": notification.pid,
            "process_name": notification.process_name,
            "path": notification.path,
            "operation": notification.operation,
            "blocked": notification.blocked,
            "hostname": notification.hostname,
            "metadata": notification.metadata
        });

        debug!("Sending webhook notification to {}", self.config.url);

        let response = request.json(&payload).send().await?;

        if response.status().is_success() {
            info!("Webhook notification sent successfully");
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Webhook notification failed: {} - {}", status, body);
            Err(NotificationError::ConfigError(format!(
                "Webhook error: {} - {}",
                status, body
            )))
        }
    }
}

// ============================================================================
// Backend Factory
// ============================================================================

/// Create all backends from a unified config
pub fn create_backends(config: &super::NotificationConfig) -> Vec<Arc<dyn NotificationBackend>> {
    let mut backends: Vec<Arc<dyn NotificationBackend>> = Vec::new();

    if let Some(ref slack_config) = config.slack {
        backends.push(Arc::new(SlackBackend::new(slack_config.clone())));
    }

    if let Some(ref discord_config) = config.discord {
        backends.push(Arc::new(DiscordBackend::new(discord_config.clone())));
    }

    if let Some(ref ntfy_config) = config.ntfy {
        backends.push(Arc::new(NtfyBackend::new(ntfy_config.clone())));
    }

    if let Some(ref telegram_config) = config.telegram {
        backends.push(Arc::new(TelegramBackend::new(telegram_config.clone())));
    }

    if let Some(ref pagerduty_config) = config.pagerduty {
        backends.push(Arc::new(PagerDutyBackend::new(pagerduty_config.clone())));
    }

    if let Some(ref pushover_config) = config.pushover {
        backends.push(Arc::new(PushoverBackend::new(pushover_config.clone())));
    }

    if let Some(ref syslog_config) = config.syslog {
        backends.push(Arc::new(SyslogBackend::new(syslog_config.clone())));
    }

    for webhook_config in &config.webhooks {
        backends.push(Arc::new(WebhookBackend::new(webhook_config.clone())));
    }

    backends
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_payload_building() {
        let config = SlackConfig {
            webhook_url: "https://hooks.slack.com/test".to_string(),
            enabled: true,
            ..Default::default()
        };

        let backend = SlackBackend::new(config);

        let notification = AlertNotification::new(
            super::super::NotificationSeverity::Critical,
            "Test Alert",
            "This is a test",
        );

        let payload = backend.build_payload(&notification);
        assert_eq!(payload["username"], "roea-ai");
        assert!(payload["attachments"].is_array());
    }

    #[test]
    fn test_discord_payload_building() {
        let config = DiscordConfig {
            webhook_url: "https://discord.com/api/webhooks/test".to_string(),
            enabled: true,
            ..Default::default()
        };

        let backend = DiscordBackend::new(config);

        let notification = AlertNotification::new(
            super::super::NotificationSeverity::Alert,
            "Test Alert",
            "This is a test",
        );

        let payload = backend.build_payload(&notification);
        assert_eq!(payload["username"], "roea-ai");
        assert!(payload["embeds"].is_array());
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }
}
