// ESNODE | Source Available BUSL-1.1 | Copyright (c) 2024 Estimatedstocks AB
use std::{
    io::{stdout, Stdout},
    path::PathBuf,
    time::{Duration, Instant},
};

use agent_core::state::StatusSnapshot;
use anyhow::{Context, Result};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Row, Table},
    Terminal,
};

use crate::client::AgentClient;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    Overview,
    GpuPower,
    NetworkDisk,
    Efficiency,
    MetricsProfiles,
    AgentStatus,
    Orchestrator,
}

impl Screen {
    fn title(&self) -> &str {
        match self {
            Screen::Overview => "Overview",
            Screen::GpuPower => "GPU & Power",
            Screen::NetworkDisk => "Network & Disk",
            Screen::Efficiency => "Efficiency & MCP",
            Screen::MetricsProfiles => "Metrics Profiles",
            Screen::AgentStatus => "Agent Status",
            Screen::Orchestrator => "Orchestrator",
        }
    }

    fn iterator() -> impl Iterator<Item = Screen> {
        [
            Screen::Overview,
            Screen::GpuPower,
            Screen::NetworkDisk,
            Screen::Efficiency,
            Screen::Orchestrator,
            Screen::MetricsProfiles,
            Screen::AgentStatus,
        ]
        .iter()
        .copied()
    }
}

pub struct AppState {
    screen: Screen,
    last_status: Option<StatusSnapshot>,
    message: Option<String>,
    no_color: bool,

    should_exit: bool,
    config_path: PathBuf,
    config: agent_core::AgentConfig,
}

impl AppState {
    fn new(no_color: bool, config_path: PathBuf, config: agent_core::AgentConfig) -> Self {
        Self {
            screen: Screen::Overview,
            last_status: None,
            message: None,
            no_color,

            should_exit: false,
            config_path,
            config: config.clone(),
        }
    }

    fn set_status(&mut self, status: Option<StatusSnapshot>) {
        self.last_status = status;
    }

    fn next_screen(&mut self) {
        let screens: Vec<Screen> = Screen::iterator().collect();
        let current_pos = screens.iter().position(|&s| s == self.screen).unwrap_or(0);
        let next = (current_pos + 1) % screens.len();
        self.screen = screens[next];
    }

    fn prev_screen(&mut self) {
        let screens: Vec<Screen> = Screen::iterator().collect();
        let current_pos = screens.iter().position(|&s| s == self.screen).unwrap_or(0);
        let prev = if current_pos == 0 {
            screens.len() - 1
        } else {
            current_pos - 1
        };
        self.screen = screens[prev];
    }
}

pub fn run_console(
    client: &AgentClient,
    no_color: bool,
    config_path: PathBuf,
    config: agent_core::AgentConfig,
) -> Result<()> {
    let stdout = prepare_terminal()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.show_cursor()?;

    let mut state = AppState::new(no_color, config_path, config);
    refresh_status(&mut state, client);
    let mut last_refresh = Instant::now();

    loop {
        terminal.draw(|f| render(f, &state))?;

        if state.should_exit {
            break;
        }

        let timeout = Duration::from_millis(200);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if !matches!(key.kind, KeyEventKind::Release) {
                    let refresh_now = handle_key(key.code, &mut state);
                    if state.should_exit {
                        break;
                    }
                    if refresh_now {
                        refresh_status(&mut state, client);
                    }
                }
            }
        }

        if last_refresh.elapsed() > Duration::from_secs(5) {
            refresh_status(&mut state, client);
            last_refresh = Instant::now();
        }
    }

    restore_terminal()?;
    Ok(())
}

fn refresh_status(state: &mut AppState, client: &AgentClient) {
    match client.fetch_status() {
        Ok(snapshot) => {
            state.message = None;
            state.set_status(Some(snapshot));
        }
        Err(err) => {
            state.message = Some(format!(
                "Unable to reach agent at {}: {err}",
                client.base_url()
            ));
            state.set_status(None);
        }
    }
}

fn prepare_terminal() -> Result<Stdout> {
    enable_raw_mode().context("enabling raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide).context("preparing terminal")?;
    Ok(stdout)
}

fn restore_terminal() -> Result<()> {
    let mut stdout = stdout();
    execute!(stdout, LeaveAlternateScreen, cursor::Show).context("restoring terminal")?;
    disable_raw_mode().context("disabling raw mode")
}

// --- Styles ---

fn style_header(state: &AppState) -> Style {
    if state.no_color {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(Color::Rgb(26, 35, 50)) // Dark navy matching brand
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }
}

fn style_sidebar(state: &AppState) -> Style {
    if state.no_color {
        Style::default()
    } else {
        Style::default()
            .bg(Color::Rgb(20, 27, 40)) // Slightly darker navy
            .fg(Color::Rgb(156, 163, 175)) // Light gray text
    }
}

fn style_sidebar_active(state: &AppState) -> Style {
    if state.no_color {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
            .bg(Color::Rgb(37, 99, 235)) // Bright blue accent
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }
}

fn style_content_block(state: &AppState) -> Style {
    if state.no_color {
        Style::default()
    } else {
        Style::default().fg(Color::Reset)
    }
}

fn style_label(state: &AppState) -> Style {
    if state.no_color {
        Style::default()
    } else {
        Style::default().fg(Color::Rgb(100, 181, 246)) // Light blue
    }
}

fn style_green(state: &AppState) -> Style {
    if state.no_color {
        Style::default()
    } else {
        Style::default().fg(Color::Rgb(76, 175, 80)) // Material green
    }
}

fn style_yellow(state: &AppState) -> Style {
    if state.no_color {
        Style::default()
    } else {
        Style::default().fg(Color::Rgb(255, 193, 7)) // Amber/orange
    }
}

fn style_red(state: &AppState) -> Style {
    if state.no_color {
        Style::default()
    } else {
        Style::default().fg(Color::Rgb(244, 67, 54)) // Material red
    }
}

// --- Render ---

fn render(frame: &mut ratatui::Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Body
            Constraint::Length(1), // Footer
        ])
        .split(frame.size());

    render_header(frame, chunks[0], state);
    render_body(frame, chunks[1], state);
    render_footer(frame, chunks[2], state);
}

fn render_header(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let status_text = if state.last_status.is_some() {
        "● ONLINE"
    } else {
        "● CONNECTING..."
    };
    let status_style = if state.last_status.is_some() {
        style_green(state)
    } else {
        style_yellow(state)
    };

    // ESNODE asterisk logo in ASCII (simplified star symbol)
    let logo = if state.no_color {
        "*"
    } else {
        "✱"
    };

    // Header content with new branding
    let header_text = Line::from(vec![
        Span::styled(
            format!(" {} ", logo),
            if state.no_color {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Rgb(255, 193, 7)) // Orange/amber accent
                    .add_modifier(Modifier::BOLD)
            },
        ),
        Span::styled(
            "ESNODE",
            if state.no_color {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            },
        ),
        Span::raw("  "),
        Span::styled(
            "Power-Aware AI Infrastructure",
            if state.no_color {
                Style::default()
            } else {
                Style::default().fg(Color::Rgb(100, 181, 246)) // Light blue
            },
        ),
        Span::raw("                    "),
        Span::styled(status_text, status_style),
    ]);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .style(style_header(state));

    let p = Paragraph::new(header_text)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(p, area);
}

fn render_body(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25), // Sidebar width
            Constraint::Min(0),     // Content width
        ])
        .split(area);

    render_sidebar(frame, chunks[0], state);
    render_content(frame, chunks[1], state);
}

fn render_sidebar(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = Screen::iterator()
        .map(|s| {
            let style = if s == state.screen {
                style_sidebar_active(state)
            } else {
                style_sidebar(state)
            };
            let prefix = if s == state.screen { "▶ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, s.title())).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .title(" Navigation "),
        )
        .style(style_sidebar(state));

    frame.render_widget(list, area);
}

fn render_content(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .style(style_content_block(state))
        .padding(ratatui::widgets::Padding::new(2, 2, 1, 1));
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if state.last_status.is_none() {
        let text = Paragraph::new("Waiting for data from agent daemon...")
            .alignment(Alignment::Center)
            .style(style_yellow(state));
        frame.render_widget(text, inner_area);
        return;
    }

    match state.screen {
        Screen::Overview => render_overview(frame, inner_area, state),
        Screen::GpuPower => render_gpu_power(frame, inner_area, state),
        Screen::NetworkDisk => render_network_disk(frame, inner_area, state),
        Screen::Efficiency => render_efficiency(frame, inner_area, state),
        Screen::MetricsProfiles => render_metric_profiles(frame, inner_area, state),
        Screen::AgentStatus => render_agent_status(frame, inner_area, state),
        Screen::Orchestrator => render_orchestrator_panel(frame, inner_area, state),
        _ => render_generic_text(frame, inner_area, state),
    }

    // Overlay message toast if exists
    if let Some(msg) = &state.message {
        let toast_area = Rect::new(area.x + 2, area.y + area.height - 3, area.width - 4, 1);
        let toast = Paragraph::new(format!("! {}", msg))
            .style(style_header(state))
            .alignment(Alignment::Center);
        frame.render_widget(toast, toast_area);
    }
}

fn render_overview(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let status = state.last_status.as_ref().unwrap();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Top gauges
            Constraint::Min(10),   // Details
        ])
        .split(area);

    // Top Gauges Row
    let gauge_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(chunks[0]);

    // CPU Gauge
    let cpu_util = status.cpu_util_percent.unwrap_or(0.0);
    let cpu_gauge = Gauge::default()
        .block(Block::default().title("CPU Usage").borders(Borders::ALL))
        .gauge_style(if cpu_util > 80.0 {
            style_red(state)
        } else {
            style_green(state)
        })
        .percent(cpu_util as u16);
    frame.render_widget(cpu_gauge, gauge_chunks[0]);

    // Memory Gauge
    let mem_total = status.mem_total_bytes.unwrap_or(1);
    let mem_used = status.mem_used_bytes.unwrap_or(0);
    let mem_percent = ((mem_used as f64 / mem_total as f64) * 100.0) as u16;
    let mem_gauge = Gauge::default()
        .block(Block::default().title("Memory Usage").borders(Borders::ALL))
        .gauge_style(style_label(state))
        .percent(mem_percent)
        .label(format!(
            "{}/{} GB",
            mem_used / 1024 / 1024 / 1024,
            mem_total / 1024 / 1024 / 1024
        ));
    frame.render_widget(mem_gauge, gauge_chunks[1]);

    // Disk/Swap Summary
    let swap_txt = if status.swap_degraded {
        "DEGRADED"
    } else {
        "OK"
    };
    let disk_txt = if status.disk_degraded {
        "DEGRADED"
    } else {
        "OK"
    };
    let p = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Disk Health: "),
            Span::styled(
                disk_txt,
                if status.disk_degraded {
                    style_red(state)
                } else {
                    style_green(state)
                },
            ),
        ]),
        Line::from(vec![
            Span::raw("Swap Health: "),
            Span::styled(
                swap_txt,
                if status.swap_degraded {
                    style_red(state)
                } else {
                    style_green(state)
                },
            ),
        ]),
        Line::from(format!("Uptime: {}s", status.uptime_seconds.unwrap_or(0))),
    ])
    .block(
        Block::default()
            .title("System Health")
            .borders(Borders::ALL),
    );
    frame.render_widget(p, gauge_chunks[2]);

    // Details Table
    let l1 = format!("{:.2}", status.load_avg_1m);
    let l5 = format!("{:.2}", status.load_avg_5m.unwrap_or(0.0));
    let l15 = format!("{:.2}", status.load_avg_15m.unwrap_or(0.0));
    let rx = format!(
        "{}/s",
        human_bytes(status.net_rx_bytes_per_sec.unwrap_or(0.0) as u64)
    );
    let tx = format!(
        "{}/s",
        human_bytes(status.net_tx_bytes_per_sec.unwrap_or(0.0) as u64)
    );

    let rows = vec![
        Row::new(vec!["Load Avg (1m)".to_string(), l1]),
        Row::new(vec!["Load Avg (5m)".to_string(), l5]),
        Row::new(vec!["Load Avg (15m)".to_string(), l15]),
        Row::new(vec!["Network Rx".to_string(), rx]),
        Row::new(vec!["Network Tx".to_string(), tx]),
    ];
    let table = Table::new(
        rows,
        [Constraint::Percentage(30), Constraint::Percentage(70)],
    )
    .block(
        Block::default()
            .title("System Details")
            .borders(Borders::ALL),
    );
    frame.render_widget(table, chunks[1]);
}

fn render_gpu_power(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let status = state.last_status.as_ref().unwrap();
    let gpu_count = status.gpus.len();

    if gpu_count == 0 {
        let p = Paragraph::new("No GPUs detected.").block(Block::default().borders(Borders::ALL));
        frame.render_widget(p, area);
        return;
    }

    let mut rows = Vec::new();
    for (i, gpu) in status.gpus.iter().enumerate() {
        let util = gpu.util_percent.unwrap_or(0.0);
        let mem = gpu.memory_used_bytes.unwrap_or(0.0) / 1024.0 / 1024.0;
        let power = gpu.power_watts.unwrap_or(0.0);
        let temp = gpu.temperature_celsius.unwrap_or(0.0);

        let style = if temp > 80.0 {
            style_red(state)
        } else {
            style_content_block(state)
        };

        rows.push(
            Row::new(vec![
                format!("GPU {}", i),
                format!("{:.1}%", util),
                format!("{:.0} MB", mem),
                format!("{:.1} W", power),
                format!("{:.1}°C", temp),
            ])
            .style(style),
        );
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(10),
        ],
    )
    .header(Row::new(vec!["ID", "Util", "Mem Used", "Power", "Temp"]).style(style_label(state)))
    .block(
        Block::default()
            .title("GPU Telemetry")
            .borders(Borders::ALL),
    );

    frame.render_widget(table, area);
}

fn render_orchestrator_panel(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let text = vec![
        Line::from(vec![
            Span::styled("Autonomy Mode: ", style_label(state)),
            Span::styled("ACTIVE", style_green(state)),
        ]),
        Line::from(""),
        Line::from("Orchestrator is running autonomously on this node."),
        Line::from("Power-aware scheduling is enabled."),
    ];

    let p = Paragraph::new(text).block(
        Block::default()
            .title("Orchestrator Status")
            .borders(Borders::ALL),
    );
    frame.render_widget(p, area);
}

fn render_generic_text(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    // Fallback for other screens to at least show something
    let text = format!(
        "Screen: {:?}\n\n(This view is being modernized)",
        state.screen
    );
    let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(p, area);
}

fn render_network_disk(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    if state.last_status.is_none() {
        let text =
            Paragraph::new("Waiting for metrics...").block(Block::default().borders(Borders::ALL));
        frame.render_widget(text, area);
        return;
    }
    let summary = NodeSummary::from_status(state);

    // Status Gauges
    let disk_health = if summary.disk_degraded {
        "DEGRADED"
    } else {
        "OK"
    };
    let net_health = if summary.network_degraded {
        "DEGRADED"
    } else {
        "OK"
    };
    let swap_health = if summary.swap_degraded {
        "DEGRADED"
    } else {
        "OK"
    };

    let items = vec![
        ListItem::new(Line::from(vec![
            Span::raw("Disk Health: "),
            Span::styled(
                disk_health,
                if summary.disk_degraded {
                    style_red(state)
                } else {
                    style_green(state)
                },
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("Network Health: "),
            Span::styled(
                net_health,
                if summary.network_degraded {
                    style_red(state)
                } else {
                    style_green(state)
                },
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("Swap Health: "),
            Span::styled(
                swap_health,
                if summary.swap_degraded {
                    style_red(state)
                } else {
                    style_green(state)
                },
            ),
        ])),
    ];

    let list = List::new(items).block(
        Block::default()
            .title("Health Status")
            .borders(Borders::ALL),
    );

    // Network Stats
    let net_rows = vec![
        Row::new(vec!["Rx Rate".to_string(), summary.net_rx]),
        Row::new(vec!["Tx Rate".to_string(), summary.net_tx]),
        Row::new(vec!["Drops".to_string(), summary.net_drop]),
    ];
    let net_table = Table::new(
        net_rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .block(
        Block::default()
            .title("Network Interface")
            .borders(Borders::ALL),
    );

    // Disk Stats
    let disk_rows = vec![
        Row::new(vec!["Usage".to_string(), summary.disk_used]),
        Row::new(vec!["Latency".to_string(), summary.disk_latency]),
        Row::new(vec!["Swap Used".to_string(), summary.swap_used]),
    ];
    let disk_table = Table::new(
        disk_rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .block(
        Block::default()
            .title("Storage / Disk")
            .borders(Borders::ALL),
    );

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Min(8),
        ])
        .split(area);

    frame.render_widget(list, layout[0]);
    frame.render_widget(net_table, layout[1]);
    frame.render_widget(disk_table, layout[2]);
}

fn render_efficiency(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    if state.last_status.is_none() {
        let text =
            Paragraph::new("Waiting for metrics...").block(Block::default().borders(Borders::ALL));
        frame.render_widget(text, area);
        return;
    }
    let summary = NodeSummary::from_status(state);

    let rows = vec![
        Row::new(vec![
            "Tokens per Joule".to_string(),
            summary.tokens_per_joule,
        ]),
        Row::new(vec!["Tokens per Watt".to_string(), summary.tokens_per_watt]),
        Row::new(vec!["Node Power Draw".to_string(), summary.node_power]),
        Row::new(vec!["Avg GPU Util".to_string(), summary.avg_gpu_util]),
        Row::new(vec!["Avg GPU Power".to_string(), summary.avg_gpu_power]),
        Row::new(vec!["CPU Util".to_string(), summary.cpu_util]),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .title("Efficiency Metrics")
            .borders(Borders::ALL),
    );

    frame.render_widget(table, area);
}

fn render_metric_profiles(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let summary = MetricToggleState::from_config(&state.config, state.last_status.as_ref());

    let rows = vec![
        Row::new(vec![
            "1. Host / Node Metrics".to_string(),
            format!("[{}]", summary.host),
        ]),
        Row::new(vec![
            "2. GPU Core Metrics".to_string(),
            format!("[{}]", summary.gpu_core),
        ]),
        Row::new(vec![
            "3. GPU Power & Energy".to_string(),
            format!("[{}]", summary.gpu_power),
        ]),
        Row::new(vec![
            "4. MCP Efficiency".to_string(),
            format!("[{}]", summary.mcp),
        ]),
        Row::new(vec![
            "5. App / HTTP Metrics".to_string(),
            format!("[{}]", summary.app),
        ]),
        Row::new(vec![
            "6. Rack Thermals".to_string(),
            format!("[{}]", summary.rack),
        ]),
    ];

    let table = Table::new(rows, [Constraint::Percentage(70), Constraint::Length(5)]).block(
        Block::default()
            .title("Metrics Profiles (Y=Enabled)")
            .borders(Borders::ALL),
    );

    let instructions = Paragraph::new("Press number keys (1-6) to toggle metric sets.")
        .style(style_label(state))
        .alignment(Alignment::Center);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(2)])
        .split(area);

    frame.render_widget(table, layout[0]);
    frame.render_widget(instructions, layout[1]);
}

fn render_agent_status(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    if state.last_status.is_none() {
        let text =
            Paragraph::new("Waiting for status...").block(Block::default().borders(Borders::ALL));
        frame.render_widget(text, area);
        return;
    }

    let s = state.last_status.as_ref().unwrap();
    let healthy = if s.healthy { "YES" } else { "WARN" };

    let status_rows = vec![
        Row::new(vec!["Agent Running".to_string(), healthy.to_string()]),
        Row::new(vec![
            "Last Scrape Time".to_string(),
            s.last_scrape_unix_ms.to_string(),
        ]),
        Row::new(vec![
            "Degradation Score".to_string(),
            s.degradation_score.to_string(),
        ]),
    ];

    let list_items: Vec<ListItem> = if s.last_errors.is_empty() {
        vec![ListItem::new("No recent errors")]
    } else {
        s.last_errors
            .iter()
            .map(|e| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("[{}] ", e.collector), style_label(state)),
                    Span::raw(format!("{} (ts={})", e.message, e.unix_ms)),
                ]))
            })
            .collect()
    };

    let list = List::new(list_items).block(
        Block::default()
            .title(format!("Recent Errors ({})", s.last_errors.len()))
            .borders(Borders::ALL),
    );

    let status_table = Table::new(
        status_rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .block(Block::default().title("Agent Health").borders(Borders::ALL));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(10)])
        .split(area);

    frame.render_widget(status_table, layout[0]);
    frame.render_widget(list, layout[1]);
}

fn render_footer(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let mode = if state.no_color { "B&W" } else { "Color" };
    let text = Line::from(vec![
        Span::raw(" F5: Refresh | "),
        Span::raw(" Arrow Keys: Navigate | "),
        Span::raw(" Q/F3: Quit | "),
        Span::raw(format!(" Mode: {} ", mode)),
    ]);
    let p = Paragraph::new(text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(Alignment::Center);
    frame.render_widget(p, area);
}

fn handle_key(code: KeyCode, state: &mut AppState) -> bool {
    match code {
        KeyCode::Esc | KeyCode::F(3) | KeyCode::Char('q') => {
            state.should_exit = true;
            false
        }
        KeyCode::Up => {
            state.prev_screen();
            false
        }
        KeyCode::Down => {
            state.next_screen();
            false
        }
        KeyCode::F(5) => true,
        // Legacy Hotkeys
        KeyCode::Char('1') => {
            state.screen = Screen::Overview;
            true
        }
        KeyCode::Char('2') => {
            state.screen = Screen::GpuPower;
            true
        }
        KeyCode::Char('3') => {
            state.screen = Screen::NetworkDisk;
            true
        }
        KeyCode::Char('7') => {
            state.screen = Screen::Orchestrator;
            true
        }
        _ => false,
    }
}

// Helpers
fn human_bytes(v: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;
    let f = v as f64;
    if f >= TB {
        format!("{:.1} TiB", f / TB)
    } else if f >= GB {
        format!("{:.1} GiB", f / GB)
    } else if f >= MB {
        format!("{:.1} MiB", f / MB)
    } else if f >= KB {
        format!("{:.0} KiB", f / KB)
    } else {
        format!("{v} B")
    }
}

fn format_duration(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3600;
    let minutes = (secs % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

// Structs restored for compilation compatibility

struct NodeSummary {
    node_name: String,
    region: String,
    uptime: String,
    cores: String,
    load_1: String,
    load_5: String,
    load_15: String,
    cpu_util: String,
    mem_total: String,
    mem_used: String,
    mem_free: String,
    swap_used: String,
    disk_used: String,
    disk_latency: String,
    net_rx: String,
    net_tx: String,
    net_drop: String,
    node_power: String,
    node_limit: String,
    spikes: String,
    therm_inlet: String,
    therm_exhaust: String,
    therm_hotspot: String,
    gpu_count: usize,
    total_vram: String,
    avg_gpu_util: String,
    avg_gpu_power: String,
    tokens_per_watt: String,
    tokens_per_joule: String,
    disk_degraded: bool,
    network_degraded: bool,
    swap_degraded: bool,
    degradation_score: u64,
}

impl NodeSummary {
    fn from_status(state: &AppState) -> Self {
        let mut summary = Self {
            node_name: "gpu-node-01".to_string(),
            region: "local".to_string(),
            uptime: "n/a".to_string(),
            cores: "n/a".to_string(),
            load_1: "n/a".to_string(),
            load_5: "n/a".to_string(),
            load_15: "n/a".to_string(),
            cpu_util: "n/a".to_string(),
            mem_total: "n/a".to_string(),
            mem_used: "n/a".to_string(),
            mem_free: "n/a".to_string(),
            swap_used: "n/a".to_string(),
            disk_used: "n/a".to_string(),
            disk_latency: "n/a".to_string(),
            net_rx: "n/a".to_string(),
            net_tx: "n/a".to_string(),
            net_drop: "0".to_string(),
            node_power: "n/a".to_string(),
            node_limit: "n/a".to_string(),
            spikes: "n/a".to_string(),
            therm_inlet: "n/a".to_string(),
            therm_exhaust: "n/a".to_string(),
            therm_hotspot: "n/a".to_string(),
            gpu_count: 0,
            total_vram: "0 GiB".to_string(),
            avg_gpu_util: "n/a".to_string(),
            avg_gpu_power: "n/a".to_string(),
            tokens_per_watt: "n/a".to_string(),
            tokens_per_joule: "n/a".to_string(),
            disk_degraded: false,
            network_degraded: false,
            swap_degraded: false,
            degradation_score: 0,
        };

        if let Some(status) = state.last_status.as_ref() {
            summary.load_1 = format!("{:.1}", status.load_avg_1m);
            if let Some(l5) = status.load_avg_5m {
                summary.load_5 = format!("{l5:.1}");
            }
            if let Some(l15) = status.load_avg_15m {
                summary.load_15 = format!("{l15:.1}");
            }
            if let Some(cores) = status.cpu_cores {
                summary.cores = format!("{cores}");
            }
            if let Some(util) = status.cpu_util_percent {
                summary.cpu_util = format!("{util:.0} %");
            }
            if let Some(uptime) = status.uptime_seconds {
                summary.uptime = format_duration(uptime);
            }
            if let (Some(total), Some(used), Some(free)) = (
                status.mem_total_bytes,
                status.mem_used_bytes,
                status.mem_free_bytes,
            ) {
                summary.mem_total = human_bytes(total);
                summary.mem_used = human_bytes(used);
                summary.mem_free = human_bytes(free);
            }
            if let Some(swap) = status.swap_used_bytes {
                summary.swap_used = human_bytes(swap);
            }
            if let (Some(total), Some(used)) =
                (status.disk_root_total_bytes, status.disk_root_used_bytes)
            {
                summary.disk_used = format!("{} / {}", human_bytes(used), human_bytes(total));
            }
            if let Some(io_ms) = status.disk_root_io_time_ms {
                summary.disk_latency = format!("{io_ms} ms");
            }
            if let Some(nic) = status.primary_nic.clone() {
                let rx = status.net_rx_bytes_per_sec.map_or_else(
                    || "n/a".to_string(),
                    |b| format!("{}/s", human_bytes(b as u64)),
                );
                let tx = status.net_tx_bytes_per_sec.map_or_else(
                    || "n/a".to_string(),
                    |b| format!("{}/s", human_bytes(b as u64)),
                );
                let drops = status
                    .net_drops_per_sec
                    .map_or_else(|| "0".to_string(), |d| format!("{d:.1}"));
                summary.net_rx = format!("{rx} ({nic})");
                summary.net_tx = tx;
                summary.net_drop = drops;
            }
            if let Some(power) = status.node_power_watts {
                summary.node_power = format!("{power:.1} W");
            } else {
                let cpu_pkg_avg: Option<f64> = {
                    let vals: Vec<f64> = status
                        .cpu_package_power_watts
                        .iter()
                        .map(|p| p.watts)
                        .collect();
                    if vals.is_empty() {
                        None
                    } else {
                        Some(vals.iter().sum::<f64>() / (vals.len() as f64))
                    }
                };
                let gpu_avg: Option<f64> = {
                    if status.gpus.is_empty() {
                        None
                    } else {
                        Some(
                            status
                                .gpus
                                .iter()
                                .filter_map(|g| g.power_watts)
                                .sum::<f64>()
                                / (status.gpus.len() as f64),
                        )
                    }
                };
                let approx = match (cpu_pkg_avg, gpu_avg) {
                    (Some(c), Some(g)) => Some(c + g),
                    (Some(c), None) => Some(c),
                    (None, Some(g)) => Some(g),
                    _ => None,
                };
                if let Some(v) = approx {
                    summary.node_power = format!("~{v:.1} W");
                }
            }

            if let Some(_tps) = status.app_tokens_per_sec {
                if let Some(tpw) = status.app_tokens_per_watt {
                    summary.tokens_per_watt = format!("{tpw:.2}");
                    summary.tokens_per_joule = format!("{tpw:.2}");
                }
            }

            if !status.cpu_temperatures.is_empty() {
                let mut inlet = None;
                let mut exhaust = None;
                let mut hotspot = None;
                for t in &status.cpu_temperatures {
                    let name = t.sensor.to_lowercase();
                    if inlet.is_none() && (name.contains("inlet") || name.contains("ambient")) {
                        inlet = Some(t.celsius);
                    }
                    if exhaust.is_none() && name.contains("exhaust") {
                        exhaust = Some(t.celsius);
                    }
                    hotspot = Some(match hotspot {
                        Some(h) if h >= t.celsius => h,
                        _ => t.celsius,
                    });
                }
                if let Some(v) = inlet {
                    summary.therm_inlet = format!("{v:.0} C");
                }
                if let Some(v) = exhaust {
                    summary.therm_exhaust = format!("{v:.0} C");
                }
                if let Some(v) = hotspot {
                    summary.therm_hotspot = format!("{v:.0} C");
                }
            }
            if !status.gpus.is_empty() {
                summary.gpu_count = status.gpus.len();
                let total_vram_bytes: f64 = status
                    .gpus
                    .iter()
                    .filter_map(|g| g.memory_total_bytes)
                    .sum();
                if total_vram_bytes > 0.0 {
                    summary.total_vram =
                        format!("{:.0} GiB", total_vram_bytes / 1024.0 / 1024.0 / 1024.0);
                }
                let avg_util: f64 = status
                    .gpus
                    .iter()
                    .filter_map(|g| g.util_percent)
                    .sum::<f64>()
                    / (status.gpus.len() as f64);
                if avg_util > 0.0 {
                    summary.avg_gpu_util = format!("{avg_util:.0} %");
                }
                let avg_power: f64 = status
                    .gpus
                    .iter()
                    .filter_map(|g| g.power_watts)
                    .sum::<f64>()
                    / (status.gpus.len() as f64);
                if avg_power > 0.0 {
                    summary.avg_gpu_power = format!("{avg_power:.0} W/GPU");
                }
                summary.tokens_per_watt = "n/a".to_string();
            }
            if let Some(limit) = state.config.node_power_envelope_watts {
                summary.node_limit = format!("{limit:.0} W");
            }
            summary.disk_degraded = status.disk_degraded;
            summary.network_degraded = status.network_degraded;
            summary.swap_degraded = status.swap_degraded;
            summary.degradation_score = status.degradation_score;
        }

        summary
    }
}

#[derive(Default)]
struct MetricToggleState {
    host: char,
    gpu_core: char,
    gpu_power: char,
    mcp: char,
    app: char,
    rack: char,
}

impl MetricToggleState {
    const fn from_config(
        config: &agent_core::AgentConfig,
        status: Option<&StatusSnapshot>,
    ) -> Self {
        let mut toggles = Self {
            host: if config.enable_cpu
                && config.enable_memory
                && config.enable_disk
                && config.enable_network
            {
                'Y'
            } else {
                'N'
            },
            gpu_core: if config.enable_gpu { 'Y' } else { 'N' },
            gpu_power: if config.enable_power { 'Y' } else { 'N' },
            mcp: if config.enable_mcp { 'Y' } else { 'N' },
            app: if config.enable_app { 'Y' } else { 'N' },
            rack: if config.enable_rack_thermals {
                'Y'
            } else {
                'N'
            },
        };

        if let Some(s) = status {
            if toggles.host == 'N'
                && (s.cpu_cores.is_some()
                    || s.mem_total_bytes.is_some()
                    || s.disk_root_total_bytes.is_some())
            {
                toggles.host = 'Y';
            }
            if toggles.gpu_core == 'N' && !s.gpus.is_empty() {
                toggles.gpu_core = 'Y';
            }
            if toggles.gpu_power == 'N'
                && (s.node_power_watts.is_some() || !s.cpu_package_power_watts.is_empty())
            {
                toggles.gpu_power = 'Y';
            }
        }
        toggles
    }
}
