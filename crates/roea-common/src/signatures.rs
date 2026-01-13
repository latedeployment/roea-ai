//! AI Agent signature definitions and matching
//!
//! This module provides the signature format for detecting AI coding agents
//! and a matching engine to identify processes.

use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::events::ProcessInfo;

/// Errors that can occur during signature operations
#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(#[from] regex::Error),

    #[error("Failed to parse signature: {0}")]
    ParseError(String),

    #[error("Signature not found: {0}")]
    NotFound(String),
}

/// Network endpoint expectations for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEndpoints {
    /// Expected/known endpoints for this agent
    #[serde(default)]
    pub expected: Vec<String>,

    /// If true, connections to endpoints not in the list are flagged
    #[serde(default)]
    pub suspicious_if_not_in_list: bool,
}

impl Default for NetworkEndpoints {
    fn default() -> Self {
        Self {
            expected: Vec::new(),
            suspicious_if_not_in_list: false,
        }
    }
}

/// Detection rules for identifying an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRules {
    /// Process names to match (exact match)
    #[serde(default)]
    pub process_names: Vec<String>,

    /// Command line patterns (regex)
    #[serde(default)]
    pub command_patterns: Vec<CommandPattern>,

    /// Executable path patterns (regex)
    #[serde(default)]
    pub exe_patterns: Vec<CommandPattern>,

    /// Parent process hints (helps with detection)
    #[serde(default)]
    pub parent_hints: Vec<String>,
}

impl Default for DetectionRules {
    fn default() -> Self {
        Self {
            process_names: Vec::new(),
            command_patterns: Vec::new(),
            exe_patterns: Vec::new(),
            parent_hints: Vec::new(),
        }
    }
}

/// A regex pattern for command/exe matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    pub regex: String,
}

/// Agent signature definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSignature {
    /// Internal name (e.g., "claude_code")
    pub name: String,

    /// Display name (e.g., "Claude Code")
    pub display_name: String,

    /// Icon filename
    #[serde(default)]
    pub icon: Option<String>,

    /// Detection rules
    pub detection: DetectionRules,

    /// Whether to track child processes
    #[serde(default)]
    pub child_process_tracking: bool,

    /// Network endpoint expectations
    #[serde(default)]
    pub network_endpoints: NetworkEndpoints,
}

/// Compiled signature for efficient matching
pub struct CompiledSignature {
    pub signature: AgentSignature,
    command_regexes: Vec<Regex>,
    exe_regexes: Vec<Regex>,
}

impl CompiledSignature {
    /// Compile a signature for matching
    pub fn new(signature: AgentSignature) -> Result<Self, SignatureError> {
        let command_regexes = signature
            .detection
            .command_patterns
            .iter()
            .map(|p| Regex::new(&p.regex))
            .collect::<Result<Vec<_>, _>>()?;

        let exe_regexes = signature
            .detection
            .exe_patterns
            .iter()
            .map(|p| Regex::new(&p.regex))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            signature,
            command_regexes,
            exe_regexes,
        })
    }

    /// Check if a process matches this signature
    pub fn matches(&self, process: &ProcessInfo) -> bool {
        // Check process name
        if self
            .signature
            .detection
            .process_names
            .iter()
            .any(|name| process.name.eq_ignore_ascii_case(name))
        {
            return true;
        }

        // Check command line patterns
        if let Some(ref cmdline) = process.cmdline {
            if self.command_regexes.iter().any(|re| re.is_match(cmdline)) {
                return true;
            }
        }

        // Check executable path patterns
        if let Some(ref exe_path) = process.exe_path {
            if self.exe_regexes.iter().any(|re| re.is_match(exe_path)) {
                return true;
            }
        }

        false
    }
}

/// Signature matcher that holds all compiled signatures
pub struct SignatureMatcher {
    signatures: Vec<CompiledSignature>,
}

impl SignatureMatcher {
    /// Create a new empty matcher
    pub fn new() -> Self {
        Self {
            signatures: Vec::new(),
        }
    }

    /// Load signatures from a list
    pub fn load(&mut self, signatures: Vec<AgentSignature>) -> Result<(), SignatureError> {
        self.signatures = signatures
            .into_iter()
            .map(CompiledSignature::new)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    /// Add a single signature
    pub fn add(&mut self, signature: AgentSignature) -> Result<(), SignatureError> {
        self.signatures.push(CompiledSignature::new(signature)?);
        Ok(())
    }

    /// Match a process against all signatures
    ///
    /// Returns the name of the first matching signature, or None
    pub fn match_process(&self, process: &ProcessInfo) -> Option<&str> {
        self.signatures
            .iter()
            .find(|sig| sig.matches(process))
            .map(|sig| sig.signature.name.as_str())
    }

    /// Get all signatures
    pub fn signatures(&self) -> impl Iterator<Item = &AgentSignature> {
        self.signatures.iter().map(|s| &s.signature)
    }

    /// Get signature by name
    pub fn get(&self, name: &str) -> Option<&AgentSignature> {
        self.signatures
            .iter()
            .find(|s| s.signature.name == name)
            .map(|s| &s.signature)
    }
}

impl Default for SignatureMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Load default built-in signatures
pub fn default_signatures() -> Vec<AgentSignature> {
    vec![
        AgentSignature {
            name: "claude_code".to_string(),
            display_name: "Claude Code".to_string(),
            icon: Some("claude.svg".to_string()),
            detection: DetectionRules {
                process_names: vec!["claude".to_string()],
                command_patterns: vec![CommandPattern {
                    regex: r"claude\s+(chat|code|--|api)".to_string(),
                }],
                exe_patterns: vec![],
                parent_hints: vec![
                    "bash".to_string(),
                    "zsh".to_string(),
                    "fish".to_string(),
                    "pwsh".to_string(),
                    "cmd.exe".to_string(),
                ],
            },
            child_process_tracking: true,
            network_endpoints: NetworkEndpoints {
                expected: vec![
                    "api.anthropic.com".to_string(),
                    "sentry.io".to_string(),
                    "statsig.anthropic.com".to_string(),
                ],
                suspicious_if_not_in_list: false,
            },
        },
        AgentSignature {
            name: "cursor".to_string(),
            display_name: "Cursor".to_string(),
            icon: Some("cursor.svg".to_string()),
            detection: DetectionRules {
                process_names: vec![
                    "Cursor".to_string(),
                    "cursor".to_string(),
                    "Cursor Helper".to_string(),
                    "Cursor Helper (Renderer)".to_string(),
                ],
                command_patterns: vec![],
                exe_patterns: vec![
                    CommandPattern {
                        regex: r"Cursor.*\.app".to_string(),
                    },
                    CommandPattern {
                        regex: r"cursor\.exe".to_string(),
                    },
                ],
                parent_hints: vec![],
            },
            child_process_tracking: true,
            network_endpoints: NetworkEndpoints {
                expected: vec![
                    "api.cursor.sh".to_string(),
                    "api.openai.com".to_string(),
                    "api.anthropic.com".to_string(),
                ],
                suspicious_if_not_in_list: false,
            },
        },
        AgentSignature {
            name: "aider".to_string(),
            display_name: "Aider".to_string(),
            icon: Some("aider.svg".to_string()),
            detection: DetectionRules {
                process_names: vec!["aider".to_string()],
                command_patterns: vec![CommandPattern {
                    regex: r"aider\s+".to_string(),
                }],
                exe_patterns: vec![],
                parent_hints: vec![
                    "bash".to_string(),
                    "zsh".to_string(),
                    "fish".to_string(),
                    "python".to_string(),
                    "python3".to_string(),
                ],
            },
            child_process_tracking: true,
            network_endpoints: NetworkEndpoints {
                expected: vec![
                    "api.openai.com".to_string(),
                    "api.anthropic.com".to_string(),
                ],
                suspicious_if_not_in_list: false,
            },
        },
        AgentSignature {
            name: "windsurf".to_string(),
            display_name: "Windsurf".to_string(),
            icon: Some("windsurf.svg".to_string()),
            detection: DetectionRules {
                process_names: vec![
                    "Windsurf".to_string(),
                    "windsurf".to_string(),
                ],
                command_patterns: vec![],
                exe_patterns: vec![
                    CommandPattern {
                        regex: r"[Ww]indsurf".to_string(),
                    },
                ],
                parent_hints: vec![],
            },
            child_process_tracking: true,
            network_endpoints: NetworkEndpoints::default(),
        },
        AgentSignature {
            name: "continue_dev".to_string(),
            display_name: "Continue.dev".to_string(),
            icon: Some("continue.svg".to_string()),
            detection: DetectionRules {
                process_names: vec!["continue".to_string()],
                command_patterns: vec![CommandPattern {
                    regex: r"continue\.dev".to_string(),
                }],
                exe_patterns: vec![],
                parent_hints: vec![],
            },
            child_process_tracking: true,
            network_endpoints: NetworkEndpoints::default(),
        },
        AgentSignature {
            name: "copilot".to_string(),
            display_name: "GitHub Copilot".to_string(),
            icon: Some("copilot.svg".to_string()),
            detection: DetectionRules {
                process_names: vec![],
                command_patterns: vec![
                    CommandPattern {
                        regex: r"copilot".to_string(),
                    },
                    CommandPattern {
                        regex: r"github\.copilot".to_string(),
                    },
                ],
                exe_patterns: vec![],
                parent_hints: vec!["code".to_string(), "Code".to_string()],
            },
            child_process_tracking: false,
            network_endpoints: NetworkEndpoints {
                expected: vec![
                    "api.github.com".to_string(),
                    "copilot-proxy.githubusercontent.com".to_string(),
                ],
                suspicious_if_not_in_list: false,
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_matching() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        // Test Claude Code matching
        let mut process = ProcessInfo::new(1234, "claude".to_string());
        assert_eq!(matcher.match_process(&process), Some("claude_code"));

        // Test with command line
        process.name = "node".to_string();
        process.cmdline = Some("claude chat".to_string());
        assert_eq!(matcher.match_process(&process), Some("claude_code"));

        // Test Cursor matching
        let cursor_process = ProcessInfo::new(5678, "Cursor".to_string());
        assert_eq!(matcher.match_process(&cursor_process), Some("cursor"));

        // Test non-matching process
        let random_process = ProcessInfo::new(9999, "firefox".to_string());
        assert_eq!(matcher.match_process(&random_process), None);
    }
}
