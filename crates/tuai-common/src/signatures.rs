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
        let name_lower = process.name.to_lowercase();
        
        // Check process name - use prefix/contains matching since Linux truncates names to 15 chars
        if self
            .signature
            .detection
            .process_names
            .iter()
            .any(|name| {
                let target = name.to_lowercase();
                // Match if process name equals, starts with, or contains the target
                name_lower == target || name_lower.starts_with(&target) || target.starts_with(&name_lower)
            })
        {
            return true;
        }

        // Check command line patterns (case-insensitive)
        if let Some(ref cmdline) = process.cmdline {
            let cmdline_lower = cmdline.to_lowercase();
            if self.command_regexes.iter().any(|re| re.is_match(&cmdline_lower)) {
                return true;
            }
        }

        // Check executable path patterns (case-insensitive)
        if let Some(ref exe_path) = process.exe_path {
            let exe_lower = exe_path.to_lowercase();
            if self.exe_regexes.iter().any(|re| re.is_match(&exe_lower)) {
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
                process_names: vec!["claude".to_string(), "claude-cli".to_string()],
                command_patterns: vec![
                    // Match "claude" as a command (at start or after path separator)
                    CommandPattern {
                        regex: r"(^|/)claude(\s|$)".to_string(),
                    },
                    // Match claude with any subcommand
                    CommandPattern {
                        regex: r"claude\s+".to_string(),
                    },
                    // Match npx claude or similar
                    CommandPattern {
                        regex: r"npx\s+.*claude".to_string(),
                    },
                    // Match Claude shell integration
                    CommandPattern {
                        regex: r"\.claude/".to_string(),
                    },
                    // Match Claude Code node processes
                    CommandPattern {
                        regex: r"@anthropic-ai/claude".to_string(),
                    },
                ],
                exe_patterns: vec![
                    CommandPattern {
                        regex: r"claude".to_string(),
                    },
                ],
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
                command_patterns: vec![
                    // Match cursor-server on Linux
                    CommandPattern {
                        regex: r"\.cursor-server/cursor".to_string(),
                    },
                    // Match cursor CLI commands
                    CommandPattern {
                        regex: r"cursor-server".to_string(),
                    },
                ],
                exe_patterns: vec![
                    CommandPattern {
                        regex: r"Cursor.*\.app".to_string(),
                    },
                    CommandPattern {
                        regex: r"cursor\.exe".to_string(),
                    },
                    CommandPattern {
                        regex: r"\.cursor-server/cursor".to_string(),
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
                    "api.groq.com".to_string(),
                    "api.together.xyz".to_string(),
                    "localhost:11434".to_string(),  // Ollama
                    "127.0.0.1:11434".to_string(),  // Ollama
                    "localhost:1234".to_string(),   // LM Studio
                    "127.0.0.1:1234".to_string(),   // LM Studio
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
            network_endpoints: NetworkEndpoints {
                expected: vec![
                    "api.openai.com".to_string(),
                    "api.anthropic.com".to_string(),
                    "api.groq.com".to_string(),
                    "api.together.xyz".to_string(),
                    "api.mistral.ai".to_string(),
                    "localhost:11434".to_string(),  // Ollama
                    "127.0.0.1:11434".to_string(),  // Ollama
                    "localhost:1234".to_string(),   // LM Studio
                    "127.0.0.1:1234".to_string(),   // LM Studio
                ],
                suspicious_if_not_in_list: false,
            },
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
        AgentSignature {
            name: "ollama".to_string(),
            display_name: "Ollama".to_string(),
            icon: Some("ollama.svg".to_string()),
            detection: DetectionRules {
                process_names: vec![
                    "ollama".to_string(),
                    "ollama_llama_server".to_string(),
                ],
                command_patterns: vec![
                    CommandPattern {
                        regex: r"ollama\s+(serve|run|pull|list|show|create|ps)".to_string(),
                    },
                    CommandPattern {
                        regex: r"ollama-runner".to_string(),
                    },
                    CommandPattern {
                        regex: r"/ollama/".to_string(),
                    },
                ],
                exe_patterns: vec![
                    CommandPattern {
                        regex: r"ollama".to_string(),
                    },
                    CommandPattern {
                        regex: r"ollama_llama_server".to_string(),
                    },
                ],
                parent_hints: vec![
                    "bash".to_string(),
                    "zsh".to_string(),
                    "fish".to_string(),
                    "systemd".to_string(),
                    "launchd".to_string(),
                ],
            },
            child_process_tracking: true,
            network_endpoints: NetworkEndpoints {
                expected: vec![
                    "localhost:11434".to_string(),
                    "127.0.0.1:11434".to_string(),
                    "ollama.ai".to_string(),
                    "registry.ollama.ai".to_string(),
                ],
                suspicious_if_not_in_list: false,
            },
        },
        AgentSignature {
            name: "lm_studio".to_string(),
            display_name: "LM Studio".to_string(),
            icon: Some("lmstudio.svg".to_string()),
            detection: DetectionRules {
                process_names: vec![
                    "LM Studio".to_string(),
                    "lm-studio".to_string(),
                    "lmstudio".to_string(),
                ],
                command_patterns: vec![
                    CommandPattern {
                        regex: r"lm-studio".to_string(),
                    },
                    CommandPattern {
                        regex: r"lmstudio".to_string(),
                    },
                ],
                exe_patterns: vec![
                    CommandPattern {
                        regex: r"LM Studio".to_string(),
                    },
                    CommandPattern {
                        regex: r"lm-studio".to_string(),
                    },
                ],
                parent_hints: vec![],
            },
            child_process_tracking: true,
            network_endpoints: NetworkEndpoints {
                expected: vec![
                    "localhost:1234".to_string(),
                    "127.0.0.1:1234".to_string(),
                ],
                suspicious_if_not_in_list: false,
            },
        },
        AgentSignature {
            name: "localai".to_string(),
            display_name: "LocalAI".to_string(),
            icon: Some("localai.svg".to_string()),
            detection: DetectionRules {
                process_names: vec![
                    "local-ai".to_string(),
                    "localai".to_string(),
                ],
                command_patterns: vec![
                    CommandPattern {
                        regex: r"local-ai".to_string(),
                    },
                    CommandPattern {
                        regex: r"localai".to_string(),
                    },
                ],
                exe_patterns: vec![
                    CommandPattern {
                        regex: r"local-ai".to_string(),
                    },
                ],
                parent_hints: vec![
                    "docker".to_string(),
                    "containerd".to_string(),
                ],
            },
            child_process_tracking: true,
            network_endpoints: NetworkEndpoints {
                expected: vec![
                    "localhost:8080".to_string(),
                    "127.0.0.1:8080".to_string(),
                ],
                suspicious_if_not_in_list: false,
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Test Fixtures
    // ========================================================================

    /// Create a mock process fixture
    fn create_process(pid: u32, name: &str) -> ProcessInfo {
        ProcessInfo::new(pid, name.to_string())
    }

    /// Create a mock process with cmdline
    fn create_process_with_cmdline(pid: u32, name: &str, cmdline: &str) -> ProcessInfo {
        let mut process = ProcessInfo::new(pid, name.to_string());
        process.cmdline = Some(cmdline.to_string());
        process
    }

    /// Create a mock process with exe path
    fn create_process_with_exe(pid: u32, name: &str, exe_path: &str) -> ProcessInfo {
        let mut process = ProcessInfo::new(pid, name.to_string());
        process.exe_path = Some(exe_path.to_string());
        process
    }

    /// Create a full mock process
    fn create_full_process(
        pid: u32,
        name: &str,
        cmdline: Option<&str>,
        exe_path: Option<&str>,
    ) -> ProcessInfo {
        let mut process = ProcessInfo::new(pid, name.to_string());
        process.cmdline = cmdline.map(|s| s.to_string());
        process.exe_path = exe_path.map(|s| s.to_string());
        process
    }

    // ========================================================================
    // Test Module: Basic Signature Matching
    // ========================================================================

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

    // ========================================================================
    // Test Module: Exact Process Name Matching
    // ========================================================================

    #[test]
    fn test_exact_process_name_claude() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process(1000, "claude");
        assert_eq!(matcher.match_process(&process), Some("claude_code"));
    }

    #[test]
    fn test_exact_process_name_case_insensitive() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        // Cursor should match case-insensitively
        let process1 = create_process(1000, "Cursor");
        let process2 = create_process(1001, "cursor");
        let process3 = create_process(1002, "CURSOR");

        assert_eq!(matcher.match_process(&process1), Some("cursor"));
        assert_eq!(matcher.match_process(&process2), Some("cursor"));
        assert_eq!(matcher.match_process(&process3), Some("cursor"));
    }

    #[test]
    fn test_exact_process_name_aider() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process(1000, "aider");
        assert_eq!(matcher.match_process(&process), Some("aider"));
    }

    #[test]
    fn test_exact_process_name_windsurf() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process1 = create_process(1000, "Windsurf");
        let process2 = create_process(1001, "windsurf");

        assert_eq!(matcher.match_process(&process1), Some("windsurf"));
        assert_eq!(matcher.match_process(&process2), Some("windsurf"));
    }

    #[test]
    fn test_cursor_helper_processes() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let helper = create_process(1000, "Cursor Helper");
        let renderer = create_process(1001, "Cursor Helper (Renderer)");

        assert_eq!(matcher.match_process(&helper), Some("cursor"));
        assert_eq!(matcher.match_process(&renderer), Some("cursor"));
    }

    #[test]
    fn test_no_match_for_unknown_process() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let processes = vec![
            create_process(1, "firefox"),
            create_process(2, "chrome"),
            create_process(3, "bash"),
            create_process(4, "node"),
            create_process(5, "python"),
        ];

        for process in processes {
            assert_eq!(
                matcher.match_process(&process),
                None,
                "Should not match: {}",
                process.name
            );
        }
    }

    // ========================================================================
    // Test Module: Regex Pattern Matching
    // ========================================================================

    #[test]
    fn test_command_regex_claude_chat() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_cmdline(1000, "node", "claude chat");
        assert_eq!(matcher.match_process(&process), Some("claude_code"));
    }

    #[test]
    fn test_command_regex_claude_code() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_cmdline(1000, "node", "claude code --help");
        assert_eq!(matcher.match_process(&process), Some("claude_code"));
    }

    #[test]
    fn test_command_regex_claude_with_flags() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_cmdline(1000, "node", "claude --version");
        assert_eq!(matcher.match_process(&process), Some("claude_code"));
    }

    #[test]
    fn test_command_regex_claude_api() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_cmdline(1000, "node", "claude api test");
        assert_eq!(matcher.match_process(&process), Some("claude_code"));
    }

    #[test]
    fn test_command_regex_aider() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_cmdline(1000, "python", "aider --model gpt-4");
        assert_eq!(matcher.match_process(&process), Some("aider"));
    }

    #[test]
    fn test_command_regex_copilot() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_cmdline(1000, "node", "github.copilot extension");
        assert_eq!(matcher.match_process(&process), Some("copilot"));
    }

    #[test]
    fn test_command_regex_continue_dev() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_cmdline(1000, "node", "continue.dev server");
        assert_eq!(matcher.match_process(&process), Some("continue_dev"));
    }

    // ========================================================================
    // Test Module: Executable Path Matching
    // ========================================================================

    #[test]
    fn test_exe_path_cursor_macos() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_exe(
            1000,
            "Cursor",
            "/Applications/Cursor.app/Contents/MacOS/Cursor",
        );
        assert_eq!(matcher.match_process(&process), Some("cursor"));
    }

    #[test]
    fn test_exe_path_cursor_windows() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_exe(
            1000,
            "cursor",
            "C:\\Users\\user\\AppData\\Local\\Programs\\cursor\\cursor.exe",
        );
        assert_eq!(matcher.match_process(&process), Some("cursor"));
    }

    #[test]
    fn test_exe_path_windsurf() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_exe(
            1000,
            "windsurf",
            "/opt/windsurf/Windsurf",
        );
        assert_eq!(matcher.match_process(&process), Some("windsurf"));
    }

    // ========================================================================
    // Test Module: Command Line Argument Parsing
    // ========================================================================

    #[test]
    fn test_cmdline_with_arguments() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let variations = vec![
            "claude chat --help",
            "claude   chat",  // Multiple spaces
            "claude code fix",
            "claude --verbose api",
        ];

        for cmdline in variations {
            let process = create_process_with_cmdline(1000, "node", cmdline);
            assert_eq!(
                matcher.match_process(&process),
                Some("claude_code"),
                "Should match: {}",
                cmdline
            );
        }
    }

    #[test]
    fn test_cmdline_no_match_partial() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        // "claudeX" should not match "claude"
        let process = create_process_with_cmdline(1000, "node", "claudeX chat");
        assert_eq!(matcher.match_process(&process), None);
    }

    #[test]
    fn test_cmdline_empty() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let mut process = create_process(1000, "random");
        process.cmdline = Some("".to_string());
        assert_eq!(matcher.match_process(&process), None);
    }

    #[test]
    fn test_cmdline_none() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process(1000, "random");
        assert_eq!(matcher.match_process(&process), None);
    }

    // ========================================================================
    // Test Module: Version Detection
    // ========================================================================

    #[test]
    fn test_version_in_cmdline() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        // Version flags should still match
        let process = create_process_with_cmdline(1000, "node", "claude --version");
        assert_eq!(matcher.match_process(&process), Some("claude_code"));
    }

    #[test]
    fn test_versioned_binary_name() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        // "aider" exact name should match
        let process = create_process(1000, "aider");
        assert_eq!(matcher.match_process(&process), Some("aider"));
    }

    // ========================================================================
    // Test Module: Child Process Inheritance
    // ========================================================================

    #[test]
    fn test_child_process_tracking_flag() {
        let signatures = default_signatures();

        let claude = signatures.iter().find(|s| s.name == "claude_code").unwrap();
        assert!(claude.child_process_tracking);

        let cursor = signatures.iter().find(|s| s.name == "cursor").unwrap();
        assert!(cursor.child_process_tracking);

        let copilot = signatures.iter().find(|s| s.name == "copilot").unwrap();
        assert!(!copilot.child_process_tracking);
    }

    #[test]
    fn test_parent_hints() {
        let signatures = default_signatures();

        let claude = signatures.iter().find(|s| s.name == "claude_code").unwrap();
        assert!(claude.detection.parent_hints.contains(&"bash".to_string()));
        assert!(claude.detection.parent_hints.contains(&"zsh".to_string()));

        let aider = signatures.iter().find(|s| s.name == "aider").unwrap();
        assert!(aider.detection.parent_hints.contains(&"python".to_string()));
    }

    // ========================================================================
    // Test Module: Network Endpoints
    // ========================================================================

    #[test]
    fn test_network_endpoints_claude() {
        let signatures = default_signatures();
        let claude = signatures.iter().find(|s| s.name == "claude_code").unwrap();

        assert!(claude.network_endpoints.expected.contains(&"api.anthropic.com".to_string()));
        assert!(!claude.network_endpoints.suspicious_if_not_in_list);
    }

    #[test]
    fn test_network_endpoints_cursor() {
        let signatures = default_signatures();
        let cursor = signatures.iter().find(|s| s.name == "cursor").unwrap();

        assert!(cursor.network_endpoints.expected.contains(&"api.cursor.sh".to_string()));
        assert!(cursor.network_endpoints.expected.contains(&"api.openai.com".to_string()));
    }

    #[test]
    fn test_network_endpoints_copilot() {
        let signatures = default_signatures();
        let copilot = signatures.iter().find(|s| s.name == "copilot").unwrap();

        assert!(copilot.network_endpoints.expected.contains(&"api.github.com".to_string()));
    }

    // ========================================================================
    // Test Module: Ollama and Local LLM Support
    // ========================================================================

    #[test]
    fn test_exact_process_name_ollama() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process(1000, "ollama");
        assert_eq!(matcher.match_process(&process), Some("ollama"));
    }

    #[test]
    fn test_command_regex_ollama_serve() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_cmdline(1000, "ollama", "ollama serve");
        assert_eq!(matcher.match_process(&process), Some("ollama"));
    }

    #[test]
    fn test_command_regex_ollama_run() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let process = create_process_with_cmdline(1000, "ollama", "ollama run llama3");
        assert_eq!(matcher.match_process(&process), Some("ollama"));
    }

    #[test]
    fn test_network_endpoints_ollama() {
        let signatures = default_signatures();
        let ollama = signatures.iter().find(|s| s.name == "ollama").unwrap();

        assert!(ollama.network_endpoints.expected.contains(&"localhost:11434".to_string()));
        assert!(ollama.network_endpoints.expected.contains(&"127.0.0.1:11434".to_string()));
        assert!(ollama.network_endpoints.expected.contains(&"registry.ollama.ai".to_string()));
    }

    #[test]
    fn test_network_endpoints_lm_studio() {
        let signatures = default_signatures();
        let lm_studio = signatures.iter().find(|s| s.name == "lm_studio").unwrap();

        assert!(lm_studio.network_endpoints.expected.contains(&"localhost:1234".to_string()));
        assert!(lm_studio.network_endpoints.expected.contains(&"127.0.0.1:1234".to_string()));
    }

    #[test]
    fn test_network_endpoints_localai() {
        let signatures = default_signatures();
        let localai = signatures.iter().find(|s| s.name == "localai").unwrap();

        assert!(localai.network_endpoints.expected.contains(&"localhost:8080".to_string()));
    }

    #[test]
    fn test_aider_supports_ollama_endpoints() {
        let signatures = default_signatures();
        let aider = signatures.iter().find(|s| s.name == "aider").unwrap();

        // Aider should support both cloud and local LLM endpoints
        assert!(aider.network_endpoints.expected.contains(&"api.openai.com".to_string()));
        assert!(aider.network_endpoints.expected.contains(&"api.anthropic.com".to_string()));
        assert!(aider.network_endpoints.expected.contains(&"localhost:11434".to_string()));  // Ollama
        assert!(aider.network_endpoints.expected.contains(&"localhost:1234".to_string()));   // LM Studio
    }

    #[test]
    fn test_continue_dev_supports_local_llm() {
        let signatures = default_signatures();
        let continue_dev = signatures.iter().find(|s| s.name == "continue_dev").unwrap();

        assert!(continue_dev.network_endpoints.expected.contains(&"localhost:11434".to_string()));
    }

    // ========================================================================
    // Test Module: Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_matcher() {
        let matcher = SignatureMatcher::new();
        let process = create_process(1000, "claude");
        assert_eq!(matcher.match_process(&process), None);
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let invalid_sig = AgentSignature {
            name: "invalid".to_string(),
            display_name: "Invalid".to_string(),
            icon: None,
            detection: DetectionRules {
                process_names: vec![],
                command_patterns: vec![CommandPattern {
                    regex: "[invalid(regex".to_string(), // Invalid regex
                }],
                exe_patterns: vec![],
                parent_hints: vec![],
            },
            child_process_tracking: false,
            network_endpoints: NetworkEndpoints::default(),
        };

        let result = CompiledSignature::new(invalid_sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_signature_by_name() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let claude = matcher.get("claude_code");
        assert!(claude.is_some());
        assert_eq!(claude.unwrap().display_name, "Claude Code");

        let nonexistent = matcher.get("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_add_single_signature() {
        let mut matcher = SignatureMatcher::new();

        let custom_sig = AgentSignature {
            name: "custom_agent".to_string(),
            display_name: "Custom Agent".to_string(),
            icon: None,
            detection: DetectionRules {
                process_names: vec!["custom".to_string()],
                command_patterns: vec![],
                exe_patterns: vec![],
                parent_hints: vec![],
            },
            child_process_tracking: false,
            network_endpoints: NetworkEndpoints::default(),
        };

        matcher.add(custom_sig).unwrap();

        let process = create_process(1000, "custom");
        assert_eq!(matcher.match_process(&process), Some("custom_agent"));
    }

    #[test]
    fn test_signatures_iterator() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        let names: Vec<&str> = matcher.signatures().map(|s| s.name.as_str()).collect();

        assert!(names.contains(&"claude_code"));
        assert!(names.contains(&"cursor"));
        assert!(names.contains(&"aider"));
    }

    #[test]
    fn test_renamed_binary() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        // If someone renames "claude" to "my-claude", process name won't match
        // But if cmdline contains the pattern, it should still match
        let process = create_process_with_cmdline(1000, "my-claude", "claude chat");
        assert_eq!(matcher.match_process(&process), Some("claude_code"));
    }

    #[test]
    fn test_wrapped_script() {
        let mut matcher = SignatureMatcher::new();
        matcher.load(default_signatures()).unwrap();

        // Python wrapper script running aider
        let process = create_process_with_cmdline(
            1000,
            "python3",
            "/usr/bin/aider --model anthropic/claude-3",
        );
        assert_eq!(matcher.match_process(&process), Some("aider"));
    }

    #[test]
    fn test_first_match_wins() {
        let mut matcher = SignatureMatcher::new();

        // Add two signatures that could both match
        let sig1 = AgentSignature {
            name: "first".to_string(),
            display_name: "First".to_string(),
            icon: None,
            detection: DetectionRules {
                process_names: vec!["test".to_string()],
                command_patterns: vec![],
                exe_patterns: vec![],
                parent_hints: vec![],
            },
            child_process_tracking: false,
            network_endpoints: NetworkEndpoints::default(),
        };

        let sig2 = AgentSignature {
            name: "second".to_string(),
            display_name: "Second".to_string(),
            icon: None,
            detection: DetectionRules {
                process_names: vec!["test".to_string()],
                command_patterns: vec![],
                exe_patterns: vec![],
                parent_hints: vec![],
            },
            child_process_tracking: false,
            network_endpoints: NetworkEndpoints::default(),
        };

        matcher.add(sig1).unwrap();
        matcher.add(sig2).unwrap();

        let process = create_process(1000, "test");
        // First signature added should win
        assert_eq!(matcher.match_process(&process), Some("first"));
    }

    #[test]
    fn test_display_name_preservation() {
        let signatures = default_signatures();

        let claude = signatures.iter().find(|s| s.name == "claude_code").unwrap();
        assert_eq!(claude.display_name, "Claude Code");

        let cursor = signatures.iter().find(|s| s.name == "cursor").unwrap();
        assert_eq!(cursor.display_name, "Cursor");

        let copilot = signatures.iter().find(|s| s.name == "copilot").unwrap();
        assert_eq!(copilot.display_name, "GitHub Copilot");
    }

    #[test]
    fn test_icon_paths() {
        let signatures = default_signatures();

        for sig in &signatures {
            if let Some(ref icon) = sig.icon {
                assert!(icon.ends_with(".svg"), "Icon should be SVG: {}", icon);
            }
        }
    }

    #[test]
    fn test_default_signatures_count() {
        let signatures = default_signatures();
        // Should have at least the core agents
        assert!(signatures.len() >= 4, "Should have at least 4 default signatures");
    }

    // ========================================================================
    // Test Module: Serialization/Deserialization
    // ========================================================================

    #[test]
    fn test_signature_serialization() {
        let sig = AgentSignature {
            name: "test".to_string(),
            display_name: "Test Agent".to_string(),
            icon: Some("test.svg".to_string()),
            detection: DetectionRules {
                process_names: vec!["test".to_string()],
                command_patterns: vec![CommandPattern {
                    regex: "test.*".to_string(),
                }],
                exe_patterns: vec![],
                parent_hints: vec!["bash".to_string()],
            },
            child_process_tracking: true,
            network_endpoints: NetworkEndpoints {
                expected: vec!["api.test.com".to_string()],
                suspicious_if_not_in_list: true,
            },
        };

        // Should serialize without panic
        let json = serde_json::to_string(&sig).unwrap();
        assert!(json.contains("test"));

        // Should deserialize back
        let deserialized: AgentSignature = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.detection.process_names.len(), 1);
    }

    #[test]
    fn test_minimal_signature_deserialization() {
        let json = r#"{
            "name": "minimal",
            "display_name": "Minimal",
            "detection": {}
        }"#;

        let sig: AgentSignature = serde_json::from_str(json).unwrap();
        assert_eq!(sig.name, "minimal");
        assert!(sig.detection.process_names.is_empty());
        assert!(!sig.child_process_tracking);
    }
}
