use chrono::Local;
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::tui::app::{App, Severity, View};

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(1), // Breadcrumb / view selector
            Constraint::Min(10),   // Main content
            Constraint::Length(1), // Footer
        ])
        .split(size);

    draw_header(frame, chunks[0], app);
    draw_breadcrumb(frame, chunks[1], app);
    draw_content(frame, chunks[2], app);
    draw_footer(frame, chunks[3], app);

    if app.show_help {
        draw_help_overlay(frame);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let stats = &app.stats;
    let uptime = stats.uptime.as_secs();
    let h = uptime / 3600;
    let m = (uptime % 3600) / 60;
    let s = uptime % 60;

    let header = Line::from(vec![
        Span::styled(" tuai ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(
            format!("{} agents", stats.ai_agents),
            Style::default().fg(Color::Green),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} procs", stats.total_processes),
            Style::default().fg(Color::White),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} net", stats.network_connections),
            Style::default().fg(Color::Blue),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} files", stats.file_operations),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        if stats.protected_alerts > 0 {
            Span::styled(
                format!("{} alerts", stats.protected_alerts),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("0 alerts", Style::default().fg(Color::DarkGray))
        },
        Span::raw(" "),
        // Right-align uptime
        Span::styled(
            format!(
                "{:>width$}",
                format!("up {:02}:{:02}:{:02}", h, m, s),
                width = area.width as usize
                    - 6  // " tuai "
                    - 1  // space
                    - format!("{} agents", stats.ai_agents).len()
                    - 3  // " | "
                    - format!("{} procs", stats.total_processes).len()
                    - 3
                    - format!("{} net", stats.network_connections).len()
                    - 3
                    - format!("{} files", stats.file_operations).len()
                    - 3
                    - if stats.protected_alerts > 0 {
                        format!("{} alerts", stats.protected_alerts).len()
                    } else {
                        "0 alerts".len()
                    }
                    - 1  // space
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(Paragraph::new(header), area);
}

fn draw_breadcrumb(frame: &mut Frame, area: Rect, app: &App) {
    let views = [View::Agents, View::Events, View::Network, View::Alerts];
    let mut spans = Vec::new();
    spans.push(Span::raw(" "));

    for (i, view) in views.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        }
        let label = format!("[{}] {}", view.key(), view.label());
        if *view == app.view {
            spans.push(Span::styled(
                label,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(label, Style::default().fg(Color::DarkGray)));
        }
    }

    // Search indicator
    if app.searching {
        spans.push(Span::styled("  /", Style::default().fg(Color::Yellow)));
        spans.push(Span::styled(
            &app.search_query,
            Style::default().fg(Color::White),
        ));
        spans.push(Span::styled("_", Style::default().fg(Color::Yellow)));
    } else if !app.search_query.is_empty() {
        spans.push(Span::styled(
            format!("  /{}", app.search_query),
            Style::default().fg(Color::Yellow),
        ));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Rgb(30, 30, 30))),
        area,
    );
}

fn draw_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.view {
        View::Agents => draw_agents_view(frame, area, app),
        View::Events => draw_events_view(frame, area, app),
        View::Network => draw_network_view(frame, area, app),
        View::Alerts => draw_alerts_view(frame, area, app),
    }
}

fn draw_agents_view(frame: &mut Frame, area: Rect, app: &App) {
    let agents = app.agent_processes();

    let header_cells = ["PID", "NAME", "TYPE", "CWD", "CMDLINE"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = agents
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(3) as usize)
        .map(|(_, proc_info)| {
            let agent_type = proc_info.agent_type.as_deref().unwrap_or("-");
            let cwd = proc_info.cwd.as_deref().unwrap_or("-");
            let cmdline = proc_info
                .cmdline
                .as_deref()
                .unwrap_or("-");

            let cells = vec![
                Cell::from(format!("{}", proc_info.pid))
                    .style(Style::default().fg(Color::Yellow)),
                Cell::from(proc_info.name.clone()).style(Style::default().fg(Color::White)),
                Cell::from(agent_type).style(Style::default().fg(Color::Cyan)),
                Cell::from(truncate(cwd, 30)).style(Style::default().fg(Color::DarkGray)),
                Cell::from(truncate(cmdline, 60)).style(Style::default().fg(Color::DarkGray)),
            ];
            Row::new(cells)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(20),
            Constraint::Length(14),
            Constraint::Length(32),
            Constraint::Min(30),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Agents ({}) ", agents.len()))
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .border_type(BorderType::Rounded),
    );

    frame.render_widget(table, area);
}

fn draw_events_view(frame: &mut Frame, area: Rect, app: &App) {
    let events = app.filtered_events();

    let header_cells = ["TIME", "SEV", "TYPE", "PID", "PROCESS", "DETAILS"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = events
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(3) as usize)
        .map(|event| {
            let time = event
                .timestamp
                .with_timezone(&Local)
                .format("%H:%M:%S")
                .to_string();

            let cells = vec![
                Cell::from(time).style(Style::default().fg(Color::DarkGray)),
                Cell::from(event.severity.symbol())
                    .style(Style::default().fg(event.severity.color())),
                Cell::from(event.event_type.label())
                    .style(Style::default().fg(event.event_type.color())),
                Cell::from(format!("{}", event.pid))
                    .style(Style::default().fg(Color::Yellow)),
                Cell::from(event.process_name.clone())
                    .style(Style::default().fg(Color::Cyan)),
                Cell::from(truncate(&event.details, 80)).style(if event.is_protected {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::White)
                }),
            ];
            Row::new(cells)
        })
        .collect();

    let filter_text = match app.severity_filter {
        None => "",
        Some(Severity::Warning) => " [>=W]",
        Some(Severity::Alert) => " [>=A]",
        Some(Severity::Critical) => " [C]",
        Some(Severity::Info) => "",
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(9),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(16),
            Constraint::Min(30),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Events ({}){} ", events.len(), filter_text))
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .border_type(BorderType::Rounded),
    );

    frame.render_widget(table, area);
}

fn draw_network_view(frame: &mut Frame, area: Rect, app: &App) {
    let net_events = app.network_events();

    let header_cells = ["TIME", "PID", "PROCESS", "CONNECTION"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = net_events
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(3) as usize)
        .map(|event| {
            let time = event
                .timestamp
                .with_timezone(&Local)
                .format("%H:%M:%S")
                .to_string();
            let cells = vec![
                Cell::from(time).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{}", event.pid))
                    .style(Style::default().fg(Color::Yellow)),
                Cell::from(event.process_name.clone())
                    .style(Style::default().fg(Color::Cyan)),
                Cell::from(event.details.clone()).style(Style::default().fg(Color::Blue)),
            ];
            Row::new(cells)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(9),
            Constraint::Length(8),
            Constraint::Length(16),
            Constraint::Min(40),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Network ({}) ", net_events.len()))
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .border_type(BorderType::Rounded),
    );

    frame.render_widget(table, area);
}

fn draw_alerts_view(frame: &mut Frame, area: Rect, app: &App) {
    let protected = app.protected_events();

    if protected.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No protected file access detected",
                Style::default().fg(Color::Green),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Protected files are monitored for access by AI agents.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  Configure with --protect-config <file.toml>",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .title(" Alerts ")
                .title_style(Style::default().fg(Color::White))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded),
        );
        frame.render_widget(msg, area);
        return;
    }

    let header_cells = ["TIME", "PID", "PROCESS", "OPERATION", "PATH"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = protected
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(3) as usize)
        .map(|event| {
            let time = event
                .timestamp
                .with_timezone(&Local)
                .format("%H:%M:%S")
                .to_string();
            let cells = vec![
                Cell::from(time).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{}", event.pid))
                    .style(Style::default().fg(Color::Yellow)),
                Cell::from(event.process_name.clone())
                    .style(Style::default().fg(Color::Cyan)),
                Cell::from(event.event_type.label())
                    .style(Style::default().fg(Color::Red)),
                Cell::from(event.details.clone()).style(
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
            ];
            Row::new(cells)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(9),
            Constraint::Length(8),
            Constraint::Length(16),
            Constraint::Length(10),
            Constraint::Min(40),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Alerts ({}) ", protected.len()))
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .border_type(BorderType::Rounded),
    );

    frame.render_widget(table, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let filter_text = match app.severity_filter {
        None => "all",
        Some(Severity::Warning) => ">=warn",
        Some(Severity::Alert) => ">=alert",
        Some(Severity::Critical) => "crit",
        Some(Severity::Info) => "all",
    };

    let footer = Line::from(vec![
        Span::styled(" <?> ", Style::default().fg(Color::Cyan)),
        Span::styled("help", Style::default().fg(Color::DarkGray)),
        Span::styled("  </> ", Style::default().fg(Color::Cyan)),
        Span::styled("search", Style::default().fg(Color::DarkGray)),
        Span::styled("  <j/k> ", Style::default().fg(Color::Cyan)),
        Span::styled("scroll", Style::default().fg(Color::DarkGray)),
        Span::styled("  <f> ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("filter:{}", filter_text), Style::default().fg(Color::DarkGray)),
        Span::styled("  <c> ", Style::default().fg(Color::Cyan)),
        Span::styled("clear", Style::default().fg(Color::DarkGray)),
        Span::styled("  <q> ", Style::default().fg(Color::Cyan)),
        Span::styled("quit", Style::default().fg(Color::DarkGray)),
    ]);

    frame.render_widget(
        Paragraph::new(footer).style(Style::default().bg(Color::Rgb(30, 30, 30))),
        area,
    );
}

fn draw_help_overlay(frame: &mut Frame) {
    let area = frame.area();
    let popup = centered_rect(50, 70, area);

    frame.render_widget(Clear, popup);

    let help = vec![
        Line::from(Span::styled(
            "Keybindings",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        help_line("1-4", "Switch view"),
        help_line("j/k", "Scroll down/up"),
        help_line("PgUp/Dn", "Page scroll"),
        help_line("g/G", "Top/bottom"),
        help_line("/", "Search"),
        help_line("f", "Cycle severity filter"),
        help_line("c", "Clear events"),
        help_line("q/Esc", "Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Views",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        help_line("1", "Agents - tracked AI processes"),
        help_line("2", "Events - live event log"),
        help_line("3", "Network - connections"),
        help_line("4", "Alerts - protected file access"),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let widget = Paragraph::new(help)
        .block(
            Block::default()
                .title(" Help ")
                .title_style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Left);

    frame.render_widget(widget, popup);
}

fn help_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:>10}  ", key),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(desc),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
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
        .split(popup[1])[1]
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
