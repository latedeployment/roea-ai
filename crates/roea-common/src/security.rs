//! Security utilities and helpers for roea-ai
//!
//! This module provides security-related utilities including:
//! - Sensitive data sanitization for logging
//! - Input validation helpers
//! - Path sanitization
//! - Secure string handling

use std::path::Path;

/// List of sensitive environment variable names that should be redacted
const SENSITIVE_ENV_VARS: &[&str] = &[
    "API_KEY",
    "APIKEY",
    "API_SECRET",
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "TOKEN",
    "AUTH",
    "CREDENTIAL",
    "PRIVATE_KEY",
    "SSH_KEY",
    "AWS_ACCESS_KEY",
    "AWS_SECRET",
    "GITHUB_TOKEN",
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "DATABASE_URL",
    "DB_PASSWORD",
    "ENCRYPTION_KEY",
];

/// List of file path patterns that should be redacted
const SENSITIVE_PATHS: &[&str] = &[
    ".env",
    ".credentials",
    "credentials.json",
    "secrets.json",
    ".ssh/",
    ".gnupg/",
    ".aws/credentials",
    ".netrc",
    "id_rsa",
    "id_ed25519",
    ".keychain",
];

/// Sanitize a string for safe logging by redacting potential secrets
pub fn sanitize_for_log(input: &str) -> String {
    let mut result = input.to_string();

    // Redact API keys that look like common patterns
    // Anthropic keys: sk-ant-...
    result = regex_lite::Regex::new(r"sk-ant-[a-zA-Z0-9_-]{20,}")
        .unwrap()
        .replace_all(&result, "[REDACTED_ANTHROPIC_KEY]")
        .to_string();

    // OpenAI keys: sk-...
    result = regex_lite::Regex::new(r"sk-[a-zA-Z0-9]{20,}")
        .unwrap()
        .replace_all(&result, "[REDACTED_OPENAI_KEY]")
        .to_string();

    // GitHub tokens: ghp_, gho_, etc.
    result = regex_lite::Regex::new(r"gh[pousr]_[A-Za-z0-9_]{36,}")
        .unwrap()
        .replace_all(&result, "[REDACTED_GITHUB_TOKEN]")
        .to_string();

    // AWS access keys
    result = regex_lite::Regex::new(r"(AKIA|A3T|AGPA|AIDA|AROA|AIPA|ANPA|ANVA|ASIA)[A-Z0-9]{16}")
        .unwrap()
        .replace_all(&result, "[REDACTED_AWS_KEY]")
        .to_string();

    // Generic API key patterns in assignment context
    result = regex_lite::Regex::new(r#"(api[_-]?key|apikey|secret|password|token)\s*[:=]\s*["'][^"']{8,}["']"#)
        .unwrap()
        .replace_all(&result, "$1=[REDACTED]")
        .to_string();

    // Bearer tokens in headers
    result = regex_lite::Regex::new(r"Bearer\s+[a-zA-Z0-9._-]{20,}")
        .unwrap()
        .replace_all(&result, "Bearer [REDACTED]")
        .to_string();

    result
}

/// Sanitize a command line string for logging, redacting potential secrets
pub fn sanitize_cmdline(cmdline: &str) -> String {
    let mut result = cmdline.to_string();

    // Redact common flag patterns with secrets
    let secret_flags = [
        "--api-key",
        "--api_key",
        "--apikey",
        "--token",
        "--password",
        "--secret",
        "--auth",
        "-p", // often password
        "--credentials",
    ];

    for flag in secret_flags {
        // Pattern: --flag=value or --flag value
        let pattern = format!(r"{}[=\s]+\S+", regex_lite::escape(flag));
        if let Ok(re) = regex_lite::Regex::new(&pattern) {
            result = re.replace_all(&result, &format!("{}=[REDACTED]", flag)).to_string();
        }
    }

    // Apply general sanitization
    sanitize_for_log(&result)
}

/// Check if a path is sensitive and should be handled carefully
pub fn is_sensitive_path(path: &str) -> bool {
    let path_lower = path.to_lowercase();

    // Check against known sensitive path patterns
    for pattern in SENSITIVE_PATHS {
        if path_lower.contains(pattern) {
            return true;
        }
    }

    // Check file extensions
    let sensitive_extensions = [".pem", ".key", ".p12", ".pfx", ".jks", ".keystore"];
    for ext in sensitive_extensions {
        if path_lower.ends_with(ext) {
            return true;
        }
    }

    false
}

/// Check if an environment variable name is sensitive
pub fn is_sensitive_env_var(name: &str) -> bool {
    let name_upper = name.to_uppercase();
    SENSITIVE_ENV_VARS.iter().any(|&s| name_upper.contains(s))
}

/// Sanitize environment variables for logging
pub fn sanitize_env_vars(env_vars: &[(String, String)]) -> Vec<(String, String)> {
    env_vars
        .iter()
        .map(|(k, v)| {
            if is_sensitive_env_var(k) {
                (k.clone(), "[REDACTED]".to_string())
            } else {
                (k.clone(), v.clone())
            }
        })
        .collect()
}

/// Validate that a path is safe (no directory traversal)
pub fn is_safe_path(path: &str) -> bool {
    // Check for path traversal attempts
    if path.contains("..") {
        return false;
    }

    // Check for null bytes
    if path.contains('\0') {
        return false;
    }

    // Check for shell metacharacters in dangerous positions
    let dangerous_chars = ['|', ';', '&', '$', '`', '>', '<', '!'];
    for c in dangerous_chars {
        if path.contains(c) {
            return false;
        }
    }

    true
}

/// Normalize a path safely, removing any traversal attempts
pub fn normalize_path(path: &str) -> Option<String> {
    if !is_safe_path(path) {
        return None;
    }

    let path = Path::new(path);

    // Try to canonicalize - this resolves symlinks and normalizes
    // For security tests, we just check the pattern
    Some(
        path.components()
            .filter(|c| !matches!(c, std::path::Component::ParentDir))
            .collect::<std::path::PathBuf>()
            .to_string_lossy()
            .to_string(),
    )
}

/// Mask a string, showing only first and last N characters
pub fn mask_string(s: &str, visible_chars: usize) -> String {
    if s.len() <= visible_chars * 2 {
        return "*".repeat(s.len());
    }

    let start = &s[..visible_chars];
    let end = &s[s.len() - visible_chars..];
    let masked_len = s.len() - visible_chars * 2;

    format!("{}{}{}",start, "*".repeat(masked_len.min(10)), end)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod sanitize_for_log {
        use super::*;

        #[test]
        fn test_redacts_anthropic_key() {
            let input = "Using key sk-ant-api03-abcdefghijklmnopqrstuvwxyz123456";
            let result = sanitize_for_log(input);
            assert!(result.contains("[REDACTED_ANTHROPIC_KEY]"));
            assert!(!result.contains("sk-ant-"));
        }

        #[test]
        fn test_redacts_openai_key() {
            let input = "OPENAI_API_KEY=sk-proj-abcdefghijklmnopqrstuvwxyz12345678901234";
            let result = sanitize_for_log(input);
            assert!(result.contains("[REDACTED_OPENAI_KEY]"));
        }

        #[test]
        fn test_redacts_github_token() {
            let input = "token: ghp_1234567890abcdefghijklmnopqrstuvwxyz1234";
            let result = sanitize_for_log(input);
            assert!(result.contains("[REDACTED_GITHUB_TOKEN]"));
        }

        #[test]
        fn test_redacts_aws_key() {
            let input = "aws_access_key_id = AKIAIOSFODNN7EXAMPLE";
            let result = sanitize_for_log(input);
            assert!(result.contains("[REDACTED_AWS_KEY]"));
        }

        #[test]
        fn test_redacts_bearer_token() {
            let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0";
            let result = sanitize_for_log(input);
            assert!(result.contains("Bearer [REDACTED]"));
        }

        #[test]
        fn test_preserves_normal_text() {
            let input = "Normal log message with no secrets";
            let result = sanitize_for_log(input);
            assert_eq!(input, result);
        }

        #[test]
        fn test_redacts_api_key_assignment() {
            let input = r#"config.api_key = "mysecretkey123456789""#;
            let result = sanitize_for_log(input);
            assert!(result.contains("[REDACTED]"));
            assert!(!result.contains("mysecretkey"));
        }
    }

    mod sanitize_cmdline {
        use super::*;

        #[test]
        fn test_redacts_api_key_flag() {
            let input = "claude --api-key sk-ant-api03-secret123456789";
            let result = sanitize_cmdline(input);
            assert!(result.contains("[REDACTED]"));
            assert!(!result.contains("secret123456789"));
        }

        #[test]
        fn test_redacts_token_flag() {
            let input = "gh auth login --token ghp_secret123456789012345678901234567890";
            let result = sanitize_cmdline(input);
            assert!(result.contains("[REDACTED]"));
        }

        #[test]
        fn test_redacts_password_flag() {
            let input = "mysql -u root --password=mypassword123";
            let result = sanitize_cmdline(input);
            assert!(result.contains("[REDACTED]"));
            assert!(!result.contains("mypassword"));
        }

        #[test]
        fn test_preserves_normal_cmdline() {
            let input = "cargo build --release --features all";
            let result = sanitize_cmdline(input);
            assert_eq!(input, result);
        }
    }

    mod is_sensitive_path {
        use super::*;

        #[test]
        fn test_env_file() {
            assert!(is_sensitive_path("/home/user/project/.env"));
            assert!(is_sensitive_path("/app/.env.local"));
        }

        #[test]
        fn test_ssh_files() {
            assert!(is_sensitive_path("/home/user/.ssh/id_rsa"));
            assert!(is_sensitive_path("/home/user/.ssh/id_ed25519"));
        }

        #[test]
        fn test_credentials_files() {
            assert!(is_sensitive_path("/home/user/.aws/credentials"));
            assert!(is_sensitive_path("/app/credentials.json"));
        }

        #[test]
        fn test_key_files() {
            assert!(is_sensitive_path("/app/server.key"));
            assert!(is_sensitive_path("/certs/private.pem"));
        }

        #[test]
        fn test_normal_files() {
            assert!(!is_sensitive_path("/home/user/code/main.rs"));
            assert!(!is_sensitive_path("/app/config.toml"));
            assert!(!is_sensitive_path("/var/log/app.log"));
        }
    }

    mod is_sensitive_env_var {
        use super::*;

        #[test]
        fn test_api_key_variants() {
            assert!(is_sensitive_env_var("API_KEY"));
            assert!(is_sensitive_env_var("APIKEY"));
            assert!(is_sensitive_env_var("MY_API_KEY"));
        }

        #[test]
        fn test_token_variants() {
            assert!(is_sensitive_env_var("TOKEN"));
            assert!(is_sensitive_env_var("AUTH_TOKEN"));
            assert!(is_sensitive_env_var("GITHUB_TOKEN"));
        }

        #[test]
        fn test_password_variants() {
            assert!(is_sensitive_env_var("PASSWORD"));
            assert!(is_sensitive_env_var("DB_PASSWORD"));
            assert!(is_sensitive_env_var("PASSWD"));
        }

        #[test]
        fn test_normal_vars() {
            assert!(!is_sensitive_env_var("PATH"));
            assert!(!is_sensitive_env_var("HOME"));
            assert!(!is_sensitive_env_var("RUST_LOG"));
        }
    }

    mod is_safe_path {
        use super::*;

        #[test]
        fn test_normal_paths() {
            assert!(is_safe_path("/home/user/project/file.rs"));
            assert!(is_safe_path("relative/path/file.txt"));
            assert!(is_safe_path("./local/file"));
        }

        #[test]
        fn test_traversal_attack() {
            assert!(!is_safe_path("/home/user/../../../etc/passwd"));
            assert!(!is_safe_path("../../../etc/shadow"));
            assert!(!is_safe_path("foo/../../bar"));
        }

        #[test]
        fn test_null_byte() {
            assert!(!is_safe_path("/home/user/file.txt\0.jpg"));
        }

        #[test]
        fn test_shell_metacharacters() {
            assert!(!is_safe_path("/home/user/$(whoami)"));
            assert!(!is_safe_path("/home/user/file; rm -rf /"));
            assert!(!is_safe_path("/home/user/file | cat"));
        }
    }

    mod mask_string {
        use super::*;

        #[test]
        fn test_masks_middle() {
            let result = mask_string("1234567890", 2);
            assert_eq!(result, "12******90");
        }

        #[test]
        fn test_short_string() {
            let result = mask_string("abc", 2);
            assert_eq!(result, "***");
        }

        #[test]
        fn test_exact_length() {
            let result = mask_string("1234", 2);
            assert_eq!(result, "****");
        }
    }

    mod sanitize_env_vars {
        use super::*;

        #[test]
        fn test_redacts_sensitive() {
            let vars = vec![
                ("API_KEY".to_string(), "secret123".to_string()),
                ("PATH".to_string(), "/usr/bin".to_string()),
            ];
            let result = sanitize_env_vars(&vars);

            assert_eq!(result[0].1, "[REDACTED]");
            assert_eq!(result[1].1, "/usr/bin");
        }
    }
}
