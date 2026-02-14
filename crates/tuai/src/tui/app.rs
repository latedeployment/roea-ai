use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use parking_lot::RwLock;
use ratatui::prelude::*;

use crate::grpc::AgentState;
use crate::protection::ProtectionConfig;
use crate::tui::ui;
use tuai_common::ProcessInfo;

const MAX_EVENTS: usize = 2000;

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
            Severity::Info => "I",
            Severity::Warning => "W",
            Severity::Alert => "A",
            Severity::Critical => "C",
        }
    }
}

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
    pub fn label(&self) -> &'static str {
        match self {
            EventType::ProcessSpawn => "SPAWN",
            EventType::ProcessExit => "EXIT",
            EventType::Network => "NET",
            EventType::FileOpen | EventType::FileRead | EventType::FileWrite
            | EventType::FileCreate | EventType::FileDelete => "FILE",
            EventType::ProtectedAccess => "ALERT",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            EventType::ProcessSpawn => Color::Green,
            EventType::ProcessExit => Color::Red,
            EventType::Network => Color::Blue,
            EventType::FileOpen | EventType::FileRead => Color::White,
            EventType::FileWrite | EventType::FileCreate => Color::Yellow,
            EventType::FileDelete => Color::Red,
            EventType::ProtectedAccess => Color::LightRed,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub total_processes: usize,
    pub ai_agents: usize,
    pub network_connections: usize,
    pub file_operations: usize,
    pub protected_alerts: usize,
    pub uptime: Duration,
}

/// The active view in the TUI (k9s-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Default: list of tracked AI agent processes
    Agents,
    /// Live event log (all events streaming)
    Events,
    /// Network connections for tracked agents
    Network,
    /// Protected file alerts
    Alerts,
}

impl View {
    pub fn label(&self) -> &'static str {
        match self {
            View::Agents => "Agents",
            View::Events => "Events",
            View::Network => "Network",
            View::Alerts => "Alerts",
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            View::Agents => "1",
            View::Events => "2",
            View::Network => "3",
            View::Alerts => "4",
        }
    }
}

pub struct App {
    state: Arc<RwLock<AgentState>>,
    protection_config: Option<ProtectionConfig>,
    pub(crate) events: VecDeque<DisplayEvent>,
    pub(crate) tracked_pids: HashSet<u32>,
    pub(crate) known_processes: HashMap<u32, ProcessInfo>,
    known_connections: HashSet<(u32, String, u16)>,
    known_files: HashSet<(u32, String)>,
    pub(crate) stats: Stats,
    start_time: Instant,
    pub(crate) view: View,
    pub(crate) scroll_offset: usize,
    pub(crate) severity_filter: Option<Severity>,
    pub(crate) show_help: bool,
    pub(crate) search_query: String,
    pub(crate) searching: bool,
    should_quit: bool,
}

impl App {
    pub fn new(
        state: Arc<RwLock<AgentState>>,
        protection_config: Option<ProtectionConfig>,
    ) -> Self {
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
            view: View::Agents,
            scroll_offset: 0,
            severity_filter: None,
            show_help: false,
            search_query: String::new(),
            searching: false,
            should_quit: false,
        };
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
                self.known_processes.insert(process.pid, process.clone());
            }
        }
    }

    fn get_display_name(
        process: &ProcessInfo,
        state_lock: &parking_lot::RwLockReadGuard<AgentState>,
    ) -> String {
        if let Some(agent) = state_lock.signature_matcher.match_process(process) {
            agent.to_string()
        } else if process
            .name
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.')
        {
            process
                .cmdline
                .as_ref()
                .and_then(|c| c.split_whitespace().next())
                .and_then(|s| s.rsplit('/').next())
                .unwrap_or(&process.name)
                .to_string()
        } else {
            process.name.clone()
        }
    }

    fn is_protected_path(&self, path: &str) -> bool {
        self.protection_config
            .as_ref()
            .map_or(false, |cfg| cfg.is_protected(path))
    }

    pub fn poll_events(&mut self) {
        let mut pending_events = Vec::new();

        {
            let state_lock = self.state.read();

            // Poll processes
            if let Ok(processes) = state_lock.monitor.snapshot() {
                let current_pids: HashSet<u32> = processes.iter().map(|p| p.pid).collect();
                self.stats.total_processes = current_pids.len();
                self.stats.ai_agents = self.tracked_pids.len();

                for process in &processes {
                    if !self.known_processes.contains_key(&process.pid) {
                        let is_agent =
                            state_lock.signature_matcher.match_process(process).is_some();
                        if is_agent {
                            self.tracked_pids.insert(process.pid);
                        }

                        let parent_tracked = process
                            .ppid
                            .map(|p| self.tracked_pids.contains(&p))
                            .unwrap_or(false);
                        let display_name = Self::get_display_name(process, &state_lock);

                        if is_agent || parent_tracked || self.tracked_pids.contains(&process.pid) {
                            pending_events.push(DisplayEvent {
                                timestamp: Utc::now(),
                                event_type: EventType::ProcessSpawn,
                                severity: Severity::Info,
                                pid: process.pid,
                                process_name: display_name,
                                details: process.cmdline.clone().unwrap_or_default(),
                                is_protected: false,
                            });
                        }

                        self.known_processes.insert(process.pid, process.clone());
                    }
                }

                // Detect exited processes
                let exited: Vec<(u32, String)> = self
                    .known_processes
                    .iter()
                    .filter(|(pid, _)| !current_pids.contains(pid))
                    .map(|(pid, p)| (*pid, p.name.clone()))
                    .collect();

                for (pid, name) in exited {
                    if self.tracked_pids.contains(&pid) {
                        pending_events.push(DisplayEvent {
                            timestamp: Utc::now(),
                            event_type: EventType::ProcessExit,
                            severity: Severity::Warning,
                            pid,
                            process_name: name,
                            details: String::new(),
                            is_protected: false,
                        });
                        self.tracked_pids.remove(&pid);
                    }
                    self.known_processes.remove(&pid);
                }
            }

            // Poll network
            if let Ok(connections) = state_lock.network_monitor.snapshot() {
                for conn in &connections {
                    if self.tracked_pids.contains(&conn.pid) {
                        let remote_addr = conn.remote_addr.as_deref().unwrap_or("").to_string();
                        let remote_port = conn.remote_port.unwrap_or(0);
                        let key = (conn.pid, remote_addr.clone(), remote_port);

                        if !self.known_connections.contains(&key)
                            && !remote_addr.is_empty()
                            && remote_port > 0
                        {
                            self.known_connections.insert(key);
                            let proc_name = self
                                .known_processes
                                .get(&conn.pid)
                                .map(|p| p.name.clone())
                                .unwrap_or_else(|| "?".to_string());
                            let local_info = format!(
                                "{}:{}",
                                conn.local_addr.as_deref().unwrap_or("?"),
                                conn.local_port
                                    .map(|p| p.to_string())
                                    .unwrap_or_else(|| "?".to_string())
                            );
                            pending_events.push(DisplayEvent {
                                timestamp: Utc::now(),
                                event_type: EventType::Network,
                                severity: Severity::Info,
                                pid: conn.pid,
                                process_name: proc_name,
                                details: format!(
                                    "{:?} {} -> {}:{}",
                                    conn.protocol, local_info, remote_addr, remote_port
                                ),
                                is_protected: false,
                            });
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
                            let proc_name = self
                                .known_processes
                                .get(&file_op.pid)
                                .map(|p| p.name.clone())
                                .unwrap_or_else(|| "?".to_string());
                            let is_protected = self.is_protected_path(&file_op.path);

                            let (event_type, severity) = if is_protected {
                                (EventType::ProtectedAccess, Severity::Critical)
                            } else {
                                let et = match file_op.operation {
                                    tuai_common::FileOperation::Open => EventType::FileOpen,
                                    tuai_common::FileOperation::Read => EventType::FileRead,
                                    tuai_common::FileOperation::Write => EventType::FileWrite,
                                    tuai_common::FileOperation::Create => EventType::FileCreate,
                                    tuai_common::FileOperation::Delete => EventType::FileDelete,
                                    tuai_common::FileOperation::Rename => EventType::FileWrite,
                                };
                                (et, Severity::Info)
                            };

                            pending_events.push(DisplayEvent {
                                timestamp: Utc::now(),
                                event_type,
                                severity,
                                pid: file_op.pid,
                                process_name: proc_name,
                                details: format!("{:?} {}", file_op.operation, file_op.path),
                                is_protected,
                            });
                        }
                    }
                }
            }
        } // state_lock dropped here

        for event in pending_events {
            self.push_event(event);
        }
        self.stats.uptime = self.start_time.elapsed();
    }

    fn push_event(&mut self, event: DisplayEvent) {
        match event.event_type {
            EventType::ProtectedAccess => self.stats.protected_alerts += 1,
            EventType::Network => self.stats.network_connections += 1,
            EventType::FileOpen | EventType::FileRead | EventType::FileWrite
            | EventType::FileCreate | EventType::FileDelete => {
                self.stats.file_operations += 1;
            }
            _ => {}
        }

        if self.events.len() >= MAX_EVENTS {
            self.events.pop_back();
        }
        self.events.push_front(event);
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        // Search mode input
        if self.searching {
            match key {
                KeyCode::Esc => {
                    self.searching = false;
                    self.search_query.clear();
                }
                KeyCode::Enter => {
                    self.searching = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }
                _ => {}
            }
            return;
        }

        if self.show_help {
            self.show_help = false;
            return;
        }

        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('/') => {
                self.searching = true;
                self.search_query.clear();
            }

            // View switching (k9s-style: number keys)
            KeyCode::Char('1') => {
                self.view = View::Agents;
                self.scroll_offset = 0;
            }
            KeyCode::Char('2') => {
                self.view = View::Events;
                self.scroll_offset = 0;
            }
            KeyCode::Char('3') => {
                self.view = View::Network;
                self.scroll_offset = 0;
            }
            KeyCode::Char('4') => {
                self.view = View::Alerts;
                self.scroll_offset = 0;
            }

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(20);
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(20);
            }
            KeyCode::Home | KeyCode::Char('g') => self.scroll_offset = 0,
            KeyCode::End | KeyCode::Char('G') => {
                self.scroll_offset = self.item_count().saturating_sub(1);
            }

            // Filtering
            KeyCode::Char('f') => {
                self.severity_filter = match self.severity_filter {
                    None => Some(Severity::Warning),
                    Some(Severity::Warning) => Some(Severity::Alert),
                    Some(Severity::Alert) => Some(Severity::Critical),
                    Some(Severity::Critical) => None,
                    Some(Severity::Info) => Some(Severity::Warning),
                };
            }

            // Clear
            KeyCode::Char('c') => {
                self.events.clear();
                self.scroll_offset = 0;
            }

            _ => {}
        }
    }

    fn item_count(&self) -> usize {
        match self.view {
            View::Agents => self.agent_processes().len(),
            View::Events => self.filtered_events().len(),
            View::Network => self.network_events().len(),
            View::Alerts => self.protected_events().len(),
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn agent_processes(&self) -> Vec<(&u32, &ProcessInfo)> {
        let mut agents: Vec<_> = self
            .known_processes
            .iter()
            .filter(|(pid, _)| self.tracked_pids.contains(pid))
            .filter(|(_, proc_info)| {
                if self.search_query.is_empty() {
                    true
                } else {
                    let q = self.search_query.to_lowercase();
                    proc_info.name.to_lowercase().contains(&q)
                        || proc_info
                            .cmdline
                            .as_ref()
                            .map_or(false, |c| c.to_lowercase().contains(&q))
                        || proc_info
                            .agent_type
                            .as_ref()
                            .map_or(false, |a| a.to_lowercase().contains(&q))
                }
            })
            .collect();
        agents.sort_by_key(|(pid, _)| **pid);
        agents
    }

    pub fn filtered_events(&self) -> Vec<&DisplayEvent> {
        self.events
            .iter()
            .filter(|e| {
                if let Some(filter) = self.severity_filter {
                    match filter {
                        Severity::Critical => e.severity == Severity::Critical,
                        Severity::Alert => {
                            e.severity == Severity::Alert || e.severity == Severity::Critical
                        }
                        Severity::Warning => e.severity != Severity::Info,
                        Severity::Info => true,
                    }
                } else {
                    true
                }
            })
            .filter(|e| {
                if self.search_query.is_empty() {
                    true
                } else {
                    let q = self.search_query.to_lowercase();
                    e.process_name.to_lowercase().contains(&q)
                        || e.details.to_lowercase().contains(&q)
                }
            })
            .collect()
    }

    pub fn network_events(&self) -> Vec<&DisplayEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type == EventType::Network)
            .collect()
    }

    pub fn protected_events(&self) -> Vec<&DisplayEvent> {
        self.events.iter().filter(|e| e.is_protected).collect()
    }
}

pub async fn run_tui(
    state: Arc<RwLock<AgentState>>,
    protection_config: Option<ProtectionConfig>,
) -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(state, protection_config);
    let mut last_poll = Instant::now();
    let poll_interval = Duration::from_millis(500);

    loop {
        if last_poll.elapsed() >= poll_interval {
            app.poll_events();
            last_poll = Instant::now();
        }

        terminal.draw(|frame| ui::draw(frame, &app))?;

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

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
