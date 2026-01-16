//! Main TUI application logic

use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use parking_lot::RwLock;
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::grpc::AgentState;
use crate::tui::ui;
use crate::protection::ProtectionConfig;
use roea_common::{ConnectionInfo, FileOpInfo, ProcessInfo};

/// Maximum number of events to keep in history
const MAX_EVENTS: usize = 1000;

/// Event severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Alert,
    Critical,
}

impl Severity {
    pub fn color(&self) -> Color {
        match self {
            Severity::Info => Color::Cyan,
            Severity::Warning => Color::Yellow,
            Severity::Alert => Color::LightRed,
            Severity::Critical => Color::Red,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Severity::Info => "●",
            Severity::Warning => "▲",
            Severity::Alert => "◆",
            Severity::Critical => "■",
        }
    }
}

/// A unified event for display
#[derive(Debug, Clone)]
pub struct DisplayEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub severity: Severity,
    pub pid: u32,
    pub process_name: String,
    pub details: String,
    pub is_protected: bool,
}

/// Event type categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    ProcessSpawn,
    ProcessExit,
    Network,
    FileOpen,
    FileRead,
    FileWrite,
    FileCreate,
    FileDelete,
    ProtectedAccess,
}

impl EventType {
    pub fn icon(&self) -> &'static str {
        match self {
            EventType::ProcessSpawn => "🚀",
            EventType::ProcessExit => "💀",
            EventType::Network => "🌐",
            EventType::FileOpen => "📂",
            EventType::FileRead => "📖",
            EventType::FileWrite => "✏️",
            EventType::FileCreate => "🆕",
            EventType::FileDelete => "🗑️",
            EventType::ProtectedAccess => "🔴",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            EventType::ProcessSpawn => "SPAWN",
            EventType::ProcessExit => "EXIT",
            EventType::Network => "NET",
            EventType::FileOpen | EventType::FileRead | EventType::FileWrite |
            EventType::FileCreate | EventType::FileDelete => "FILE",
            EventType::ProtectedAccess => "ALERT",
        }
    }
}

/// Statistics for the dashboard
#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub total_processes: usize,
    pub ai_agents: usize,
    pub network_connections: usize,
    pub file_operations: usize,
    pub protected_alerts: usize,
    pub uptime: Duration,
}

/// Active tab in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Events,
    Processes,
    Network,
    Protected,
}

/// Main application state
pub struct App {
    /// Agent monitoring state
    state: Arc<RwLock<AgentState>>,
    /// Protection configuration
    protection_config: Option<ProtectionConfig>,
    /// Event history
    events: VecDeque<DisplayEvent>,
    /// Tracked AI agent PIDs
    tracked_pids: HashSet<u32>,
    /// Known processes (pid -> name)
    known_processes: HashMap<u32, String>,
    /// Known connections (pid, remote_addr, remote_port)
    known_connections: HashSet<(u32, String, u16)>,
    /// Known files (pid, path)
    known_files: HashSet<(u32, String)>,
    /// Current statistics
    stats: Stats,
    /// Start time
    start_time: Instant,
    /// Selected tab
    active_tab: ActiveTab,
    /// Scroll position for event list
    scroll_offset: usize,
    /// Selected event index
    selected_event: Option<usize>,
    /// Filter by severity
    severity_filter: Option<Severity>,
    /// Show only AI agent events
    agents_only: bool,
    /// Show help overlay
    show_help: bool,
    /// Should quit
    should_quit: bool,
}

impl App {
    pub fn new(state: Arc<RwLock<AgentState>>, protection_config: Option<ProtectionConfig>) -> Self {
        let mut app = Self {
            state,
            protection_config,
            events: VecDeque::with_capacity(MAX_EVENTS),
            tracked_pids: HashSet::new(),
            known_processes: HashMap::new(),
            known_connections: HashSet::new(),
            known_files: HashSet::new(),
            stats: Stats::default(),
            start_time: Instant::now(),
            active_tab: ActiveTab::Events,
            scroll_offset: 0,
            selected_event: None,
            severity_filter: None,
            agents_only: true,
            show_help: false,
            should_quit: false,
        };

        // Initialize with current processes
        app.init_tracking();
        app
    }

    fn init_tracking(&mut self) {
        let state_lock = self.state.read();
        if let Ok(processes) = state_lock.monitor.snapshot() {
            for process in &processes {
                if state_lock.signature_matcher.match_process(process).is_some() {
                    self.tracked_pids.insert(process.pid);
                }
                let display_name = self.get_display_name(process, &state_lock);
                self.known_processes.insert(process.pid, display_name);
            }
        }
    }

    fn get_display_name(&self, process: &ProcessInfo, state_lock: &parking_lot::RwLockReadGuard<AgentState>) -> String {
        if let Some(agent) = state_lock.signature_matcher.match_process(process) {
            agent.to_string()
        } else if process.name.chars().all(|c| c.is_ascii_digit() || c == '.') {
            process.cmdline.as_ref()
                .and_then(|c| c.split_whitespace().next())
                .and_then(|s| s.rsplit('/').next())
                .unwrap_or(&process.name)
                .to_string()
        } else {
            process.name.clone()
        }
    }

    pub fn add_event(&mut self, event: DisplayEvent) {
        if self.events.len() >= MAX_EVENTS {
            self.events.pop_back();
        }

        // Update stats
        match event.event_type {
            EventType::ProtectedAccess => self.stats.protected_alerts += 1,
            EventType::Network => self.stats.network_connections += 1,
            EventType::FileOpen | EventType::FileRead | EventType::FileWrite |
            EventType::FileCreate | EventType::FileDelete => self.stats.file_operations += 1,
            _ => {}
        }

        self.events.push_front(event);
    }

    fn is_protected_path(&self, path: &str) -> bool {
        if let Some(config) = &self.protection_config {
            config.is_protected(path)
        } else {
            false
        }
    }

    pub fn poll_events(&mut self) {
        let state_lock = self.state.read();

        // Poll processes
        if let Ok(processes) = state_lock.monitor.snapshot() {
            let current_pids: HashSet<u32> = processes.iter().map(|p| p.pid).collect();
            self.stats.total_processes = current_pids.len();
            self.stats.ai_agents = self.tracked_pids.len();

            // Check for new processes
            for process in &processes {
                if !self.known_processes.contains_key(&process.pid) {
                    let is_agent = state_lock.signature_matcher.match_process(process).is_some();
                    if is_agent {
                        self.tracked_pids.insert(process.pid);
                    }

                    let parent_tracked = process.ppid.map(|p| self.tracked_pids.contains(&p)).unwrap_or(false);
                    let display_name = self.get_display_name(process, &state_lock);

                    if is_agent || parent_tracked || self.tracked_pids.contains(&process.pid) {
                        let event = DisplayEvent {
                            timestamp: Utc::now(),
                            event_type: EventType::ProcessSpawn,
                            severity: if is_agent { Severity::Info } else { Severity::Info },
                            pid: process.pid,
                            process_name: display_name.clone(),
                            details: process.cmdline.clone().unwrap_or_default(),
                            is_protected: false,
                        };
                        self.events.push_front(event);
                    }

                    self.known_processes.insert(process.pid, display_name);
                }
            }

            // Check for exited processes
            let exited: Vec<(u32, String)> = self.known_processes.iter()
                .filter(|(pid, _)| !current_pids.contains(pid))
                .map(|(pid, name)| (*pid, name.clone()))
                .collect();

            for (pid, name) in exited {
                if self.tracked_pids.contains(&pid) {
                    let event = DisplayEvent {
                        timestamp: Utc::now(),
                        event_type: EventType::ProcessExit,
                        severity: Severity::Warning,
                        pid,
                        process_name: name,
                        details: String::new(),
                        is_protected: false,
                    };
                    self.events.push_front(event);
                    self.tracked_pids.remove(&pid);
                }
                self.known_processes.remove(&pid);
            }
        }

        // Poll network connections
        if let Ok(connections) = state_lock.network_monitor.snapshot() {
            for conn in &connections {
                if self.tracked_pids.contains(&conn.pid) {
                    let remote_addr = conn.remote_addr.as_deref().unwrap_or("").to_string();
                    let remote_port = conn.remote_port.unwrap_or(0);
                    let key = (conn.pid, remote_addr.clone(), remote_port);

                    if !self.known_connections.contains(&key) && !remote_addr.is_empty() && remote_port > 0 {
                        self.known_connections.insert(key);
                        let proc_name = self.known_processes.get(&conn.pid).cloned().unwrap_or_else(|| "?".to_string());
                        let local_info = format!("{}:{}",
                            conn.local_addr.as_deref().unwrap_or("?"),
                            conn.local_port.map(|p| p.to_string()).unwrap_or_else(|| "?".to_string())
                        );
                        let event = DisplayEvent {
                            timestamp: Utc::now(),
                            event_type: EventType::Network,
                            severity: Severity::Info,
                            pid: conn.pid,
                            process_name: proc_name,
                            details: format!("{:?} {} → {}:{}", conn.protocol, local_info, remote_addr, remote_port),
                            is_protected: false,
                        };
                        self.events.push_front(event);
                    }
                }
            }
        }

        // Poll file operations
        if let Ok(files) = state_lock.file_monitor.snapshot() {
            for file_op in &files {
                if self.tracked_pids.contains(&file_op.pid) {
                    let key = (file_op.pid, file_op.path.clone());

                    if !self.known_files.contains(&key) {
                        self.known_files.insert(key);
                        let proc_name = self.known_processes.get(&file_op.pid).cloned().unwrap_or_else(|| "?".to_string());
                        let is_protected = self.is_protected_path(&file_op.path);

                        let (event_type, severity) = if is_protected {
                            (EventType::ProtectedAccess, Severity::Critical)
                        } else {
                            let et = match file_op.operation {
                                roea_common::FileOperation::Open => EventType::FileOpen,
                                roea_common::FileOperation::Read => EventType::FileRead,
                                roea_common::FileOperation::Write => EventType::FileWrite,
                                roea_common::FileOperation::Create => EventType::FileCreate,
                                roea_common::FileOperation::Delete => EventType::FileDelete,
                                roea_common::FileOperation::Rename => EventType::FileWrite,
                            };
                            (et, Severity::Info)
                        };

                        let event = DisplayEvent {
                            timestamp: Utc::now(),
                            event_type,
                            severity,
                            pid: file_op.pid,
                            process_name: proc_name,
                            details: format!("{:?} {}", file_op.operation, file_op.path),
                            is_protected,
                        };
                        self.events.push_front(event);
                    }
                }
            }
        }

        drop(state_lock);
        self.stats.uptime = self.start_time.elapsed();
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        if self.show_help {
            self.show_help = false;
            return;
        }

        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('?') | KeyCode::Char('h') => self.show_help = true,
            KeyCode::Tab => {
                self.active_tab = match self.active_tab {
                    ActiveTab::Events => ActiveTab::Processes,
                    ActiveTab::Processes => ActiveTab::Network,
                    ActiveTab::Network => ActiveTab::Protected,
                    ActiveTab::Protected => ActiveTab::Events,
                };
            }
            KeyCode::Char('1') => self.active_tab = ActiveTab::Events,
            KeyCode::Char('2') => self.active_tab = ActiveTab::Processes,
            KeyCode::Char('3') => self.active_tab = ActiveTab::Network,
            KeyCode::Char('4') => self.active_tab = ActiveTab::Protected,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.scroll_offset < self.events.len().saturating_sub(1) {
                    self.scroll_offset += 1;
                }
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(20);
            }
            KeyCode::PageDown => {
                self.scroll_offset = (self.scroll_offset + 20).min(self.events.len().saturating_sub(1));
            }
            KeyCode::Home => self.scroll_offset = 0,
            KeyCode::End => self.scroll_offset = self.events.len().saturating_sub(1),
            KeyCode::Char('a') => self.agents_only = !self.agents_only,
            KeyCode::Char('c') => {
                self.events.clear();
                self.scroll_offset = 0;
            }
            KeyCode::Char('f') => {
                self.severity_filter = match self.severity_filter {
                    None => Some(Severity::Warning),
                    Some(Severity::Warning) => Some(Severity::Alert),
                    Some(Severity::Alert) => Some(Severity::Critical),
                    Some(Severity::Critical) => None,
                    Some(Severity::Info) => Some(Severity::Warning),
                };
            }
            _ => {}
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn filtered_events(&self) -> Vec<&DisplayEvent> {
        self.events
            .iter()
            .filter(|e| {
                if let Some(filter) = self.severity_filter {
                    match filter {
                        Severity::Critical => e.severity == Severity::Critical,
                        Severity::Alert => e.severity == Severity::Alert || e.severity == Severity::Critical,
                        Severity::Warning => e.severity != Severity::Info,
                        Severity::Info => true,
                    }
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn protected_events(&self) -> Vec<&DisplayEvent> {
        self.events
            .iter()
            .filter(|e| e.is_protected)
            .collect()
    }
}

/// Run the TUI application
pub async fn run_tui(state: Arc<RwLock<AgentState>>, protection_config: Option<ProtectionConfig>) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(state, protection_config);
    let mut last_poll = Instant::now();
    let poll_interval = Duration::from_millis(500);

    loop {
        // Poll for new events
        if last_poll.elapsed() >= poll_interval {
            app.poll_events();
            last_poll = Instant::now();
        }

        // Draw UI
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }

        if app.should_quit() {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
