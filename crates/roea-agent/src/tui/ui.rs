//! UI rendering for the TUI

use chrono::Local;
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::tui::app::{ActiveTab, App, DisplayEvent, EventType, Severity};

/// Main UI drawing function
pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Stats bar
            Constraint::Min(10),    // Main content
            Constraint::Length(2),  // Footer
        ])
        .split(size);

    draw_header(frame, chunks[0]);
    draw_stats_bar(frame, chunks[1], app);
    draw_main_content(frame, chunks[2], app);
    draw_footer(frame, chunks[3], app);

    // Draw help overlay if active
    if app.show_help {
        draw_help_overlay(frame);
    }
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("┃", Style::default().fg(Color::Cyan)),
            Span::styled(" roea-ai ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("┃", Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled("AI Agent Monitor", Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled("━━━", Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled("TUI Mode", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
    ])
    .block(Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .border_type(BorderType::Double));

    frame.render_widget(title, area);
}

fn draw_stats_bar(frame: &mut Frame, area: Rect, app: &App) {
    let stats = &app.stats;
    let uptime_secs = stats.uptime.as_secs();
    let hours = uptime_secs / 3600;
    let mins = (uptime_secs % 3600) / 60;
    let secs = uptime_secs % 60;

    let stats_line = Line::from(vec![
        Span::styled(" ▶ ", Style::default().fg(Color::Green)),
        Span::styled("Agents: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", stats.ai_agents), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  │  "),
        Span::styled("Procs: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", stats.total_processes), Style::default().fg(Color::White)),
        Span::raw("  │  "),
        Span::styled("Net: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", stats.network_connections), Style::default().fg(Color::Blue)),
        Span::raw("  │  "),
        Span::styled("Files: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", stats.file_operations), Style::default().fg(Color::Yellow)),
        Span::raw("  │  "),
        Span::styled("🔴 Alerts: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", stats.protected_alerts),
            Style::default().fg(if stats.protected_alerts > 0 { Color::Red } else { Color::Green }).add_modifier(Modifier::BOLD)
        ),
        Span::raw("  │  "),
        Span::styled("Uptime: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:02}:{:02}:{:02}", hours, mins, secs), Style::default().fg(Color::Magenta)),
    ]);

    let stats_widget = Paragraph::new(stats_line)
        .block(Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)));

    frame.render_widget(stats_widget, area);
}

fn draw_main_content(frame: &mut Frame, area: Rect, app: &App) {
    // Tab bar
    let tabs = vec!["[1] Events", "[2] Processes", "[3] Network", "[4] Protected"];
    let tab_titles: Vec<Line> = tabs.iter().enumerate().map(|(i, t)| {
        let is_active = match (i, app.active_tab) {
            (0, ActiveTab::Events) => true,
            (1, ActiveTab::Processes) => true,
            (2, ActiveTab::Network) => true,
            (3, ActiveTab::Protected) => true,
            _ => false,
        };
        if is_active {
            Line::from(Span::styled(*t, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        } else {
            Line::from(Span::styled(*t, Style::default().fg(Color::DarkGray)))
        }
    }).collect();

    let tabs_widget = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)))
        .select(match app.active_tab {
            ActiveTab::Events => 0,
            ActiveTab::Processes => 1,
            ActiveTab::Network => 2,
            ActiveTab::Protected => 3,
        })
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(5)])
        .split(area);

    frame.render_widget(tabs_widget, content_chunks[0]);

    // Main content based on active tab
    match app.active_tab {
        ActiveTab::Events => draw_events_tab(frame, content_chunks[1], app),
        ActiveTab::Processes => draw_processes_tab(frame, content_chunks[1], app),
        ActiveTab::Network => draw_network_tab(frame, content_chunks[1], app),
        ActiveTab::Protected => draw_protected_tab(frame, content_chunks[1], app),
    }
}

fn draw_events_tab(frame: &mut Frame, area: Rect, app: &App) {
    let events = app.filtered_events();

    let header_cells = ["Time", "Type", "Sev", "PID", "Process", "Details"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = events
        .iter()
        .skip(app.scroll_offset)
        .take(area.height as usize - 3)
        .map(|event| {
            let time = event.timestamp.with_timezone(&Local).format("%H:%M:%S").to_string();
            let type_style = if event.is_protected {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let cells = vec![
                Cell::from(time).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{} {}", event.event_type.icon(), event.event_type.label())).style(type_style),
                Cell::from(event.severity.symbol()).style(Style::default().fg(event.severity.color())),
                Cell::from(format!("{}", event.pid)).style(Style::default().fg(Color::Yellow)),
                Cell::from(event.process_name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(truncate_string(&event.details, 60)).style(
                    if event.is_protected {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::White)
                    }
                ),
            ];
            Row::new(cells)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),   // Time
            Constraint::Length(12),   // Type
            Constraint::Length(4),    // Severity
            Constraint::Length(8),    // PID
            Constraint::Length(16),   // Process
            Constraint::Min(30),      // Details
        ],
    )
    .header(header)
    .block(Block::default()
        .title(format!(" Events ({}) ", events.len()))
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray)));

    frame.render_widget(table, area);
}

fn draw_processes_tab(frame: &mut Frame, area: Rect, app: &App) {
    let processes: Vec<(&u32, &String)> = app.known_processes.iter().collect();

    let header_cells = ["PID", "Name", "AI Agent"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = processes
        .iter()
        .map(|(pid, name)| {
            let is_agent = app.tracked_pids.contains(*pid);
            let cells = vec![
                Cell::from(format!("{}", pid)).style(Style::default().fg(Color::Yellow)),
                Cell::from(name.to_string()).style(Style::default().fg(if is_agent { Color::Cyan } else { Color::White })),
                Cell::from(if is_agent { "✓ AI" } else { "" }).style(Style::default().fg(Color::Green)),
            ];
            Row::new(cells)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Min(30),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(Block::default()
        .title(format!(" Tracked Processes ({}) ", processes.len()))
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray)));

    frame.render_widget(table, area);
}

fn draw_network_tab(frame: &mut Frame, area: Rect, app: &App) {
    let net_events: Vec<&DisplayEvent> = app.events
        .iter()
        .filter(|e| e.event_type == EventType::Network)
        .collect();

    let header_cells = ["Time", "PID", "Process", "Connection"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = net_events
        .iter()
        .take(area.height as usize - 3)
        .map(|event| {
            let time = event.timestamp.with_timezone(&Local).format("%H:%M:%S").to_string();
            let cells = vec![
                Cell::from(time).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{}", event.pid)).style(Style::default().fg(Color::Yellow)),
                Cell::from(event.process_name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(event.details.clone()).style(Style::default().fg(Color::Blue)),
            ];
            Row::new(cells)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(16),
            Constraint::Min(40),
        ],
    )
    .header(header)
    .block(Block::default()
        .title(format!(" Network Connections ({}) ", net_events.len()))
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray)));

    frame.render_widget(table, area);
}

fn draw_protected_tab(frame: &mut Frame, area: Rect, app: &App) {
    let protected_events = app.protected_events();

    if protected_events.is_empty() {
        let message = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("  ✓ No protected file access detected", Style::default().fg(Color::Green))),
            Line::from(""),
            Line::from(Span::styled("  Protected files/folders are monitored for access by AI agents.", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled("  Configure with --protect-config <file.toml>", Style::default().fg(Color::DarkGray))),
        ])
        .block(Block::default()
            .title(" 🔒 Protected Files ")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));

        frame.render_widget(message, area);
        return;
    }

    let header_cells = ["Time", "PID", "Process", "Operation", "Path"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = protected_events
        .iter()
        .take(area.height as usize - 3)
        .map(|event| {
            let time = event.timestamp.with_timezone(&Local).format("%H:%M:%S").to_string();
            let cells = vec![
                Cell::from(time).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{}", event.pid)).style(Style::default().fg(Color::Yellow)),
                Cell::from(event.process_name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(event.event_type.icon()).style(Style::default().fg(Color::Red)),
                Cell::from(event.details.clone()).style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            ];
            Row::new(cells)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(16),
            Constraint::Length(10),
            Constraint::Min(40),
        ],
    )
    .header(header)
    .block(Block::default()
        .title(format!(" 🔴 PROTECTED FILE ALERTS ({}) ", protected_events.len()))
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red)));

    frame.render_widget(table, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let filter_text = match app.severity_filter {
        None => "All",
        Some(Severity::Warning) => "≥Warning",
        Some(Severity::Alert) => "≥Alert",
        Some(Severity::Critical) => "Critical",
        Some(Severity::Info) => "All",
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [?] Help ", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(" [Tab] Switch Tab ", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(" [↑↓] Scroll ", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(" [f] Filter: ", Style::default().fg(Color::DarkGray)),
        Span::styled(filter_text, Style::default().fg(Color::Cyan)),
        Span::styled(" │", Style::default().fg(Color::DarkGray)),
        Span::styled(" [c] Clear ", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(" [q] Quit ", Style::default().fg(Color::DarkGray)),
    ]));

    frame.render_widget(footer, area);
}

fn draw_help_overlay(frame: &mut Frame) {
    let area = frame.area();
    let popup_area = centered_rect(60, 60, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled("Keyboard Shortcuts", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tab      ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch between tabs"),
        ]),
        Line::from(vec![
            Span::styled("  1-4      ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump to specific tab"),
        ]),
        Line::from(vec![
            Span::styled("  ↑/k      ", Style::default().fg(Color::Yellow)),
            Span::raw("Scroll up"),
        ]),
        Line::from(vec![
            Span::styled("  ↓/j      ", Style::default().fg(Color::Yellow)),
            Span::raw("Scroll down"),
        ]),
        Line::from(vec![
            Span::styled("  PgUp/Dn  ", Style::default().fg(Color::Yellow)),
            Span::raw("Page up/down"),
        ]),
        Line::from(vec![
            Span::styled("  Home/End ", Style::default().fg(Color::Yellow)),
            Span::raw("Go to top/bottom"),
        ]),
        Line::from(vec![
            Span::styled("  f        ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle severity filter"),
        ]),
        Line::from(vec![
            Span::styled("  c        ", Style::default().fg(Color::Yellow)),
            Span::raw("Clear all events"),
        ]),
        Line::from(vec![
            Span::styled("  a        ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle agents-only filter"),
        ]),
        Line::from(vec![
            Span::styled("  q/Esc    ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit application"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Severity Levels", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ●  ", Style::default().fg(Color::Cyan)),
            Span::raw("Info - Normal operations"),
        ]),
        Line::from(vec![
            Span::styled("  ▲  ", Style::default().fg(Color::Yellow)),
            Span::raw("Warning - Process exits, etc."),
        ]),
        Line::from(vec![
            Span::styled("  ◆  ", Style::default().fg(Color::LightRed)),
            Span::raw("Alert - Requires attention"),
        ]),
        Line::from(vec![
            Span::styled("  ■  ", Style::default().fg(Color::Red)),
            Span::raw("Critical - Protected file access!"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", Style::default().fg(Color::DarkGray))),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .title(" 📖 Help ")
            .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .border_type(BorderType::Rounded))
        .alignment(Alignment::Left);

    frame.render_widget(help, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
