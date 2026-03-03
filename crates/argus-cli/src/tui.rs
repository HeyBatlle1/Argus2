//! Interactive TUI for Argus

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io;

use argus_core::{AgentConfig, AgentEvent, ConversationMessage, McpClient, ShellPolicy};
use argus_memory::SqliteMemory;

const ARGUS_WATCHING: &str = "\
\n\
\u25c9\n\
\u25c9 \u25c9 \u25c9\n\
\u25c9 \u25c9 \u25c9 \u25c9 \u25c9\n\
\u25c9 \u25c9 \u25c9\n\
\u25c9\n\
\n\
ALL EYES OPEN";

const ARGUS_THINKING: &str = "\
\n\
\u25ce\n\
\u25ce \u25ce \u25ce\n\
\u25ce \u25ce \u25ce \u25ce \u25ce\n\
\u25ce \u25ce \u25ce\n\
\u25ce\n\
\n\
THINKING...";

const ARGUS_EXECUTING: &str = "\
\n\
\u2299\n\
\u2299 \u2299 \u2299\n\
\u2299 \u2299 \u26a1 \u2299 \u2299\n\
\u2299 \u2299 \u2299\n\
\u2299\n\
\n\
EXECUTING";

const ARGUS_COMPLETE: &str = "\
\n\
\u2726\n\
\u2726 \u2726 \u2726\n\
\u2726 \u2726 \u2713 \u2726 \u2726\n\
\u2726 \u2726 \u2726\n\
\u2726\n\
\n\
COMPLETE";

#[derive(Clone, Copy, PartialEq)]
enum ArgusState {
    Watching,
    Thinking,
    Executing,
    Complete,
}

struct ChatMessage {
    role: String,
    content: String,
}

struct App {
    chat: Vec<ChatMessage>,
    history: Vec<ConversationMessage>,
    input: String,
    scroll: u16,
    config: AgentConfig,
    client: reqwest::Client,
    state: ArgusState,
    memory: SqliteMemory,
    mcp: McpClient,
    shell_policy: ShellPolicy,
}

impl App {
    fn new(config: AgentConfig) -> Result<Self, String> {
        let memory = SqliteMemory::open_default()
            .map_err(|e| format!("Failed to open memory: {}", e))?;

        let mut mcp = McpClient::new();
        let mcp_errors = mcp.connect_all();
        for err in &mcp_errors {
            eprintln!("MCP: {}", err);
        }

        Ok(Self {
            chat: vec![],
            history: vec![],
            input: String::new(),
            scroll: 0,
            config,
            client: reqwest::Client::new(),
            state: ArgusState::Watching,
            memory,
            mcp,
            shell_policy: ShellPolicy::default(),
        })
    }

    async fn send_message(&mut self) -> anyhow::Result<()> {
        if self.input.trim().is_empty() {
            return Ok(());
        }

        let user_msg = self.input.clone();
        self.chat.push(ChatMessage { role: "You".to_string(), content: user_msg.clone() });
        self.input.clear();
        self.state = ArgusState::Thinking;

        let mut response_text = String::new();
        let mut tool_log: Vec<String> = Vec::new();

        let result = argus_core::run_agent_turn(
            &self.config,
            &user_msg,
            &self.history,
            &self.shell_policy,
            &self.memory,
            &mut self.mcp,
            &self.client,
            |event| match event {
                AgentEvent::ToolCall { name, preview } => {
                    self.state = ArgusState::Executing;
                    let short = if preview.len() > 60 { format!("{}...", &preview[..60]) } else { preview };
                    tool_log.push(format!("\u{1f527} {}: {}", name, short));
                }
                AgentEvent::Response(text) => { response_text = text; }
                AgentEvent::Error(err) => { response_text = format!("\u274c {}", err); }
                _ => {}
            },
        ).await;

        if let Err(e) = result {
            if response_text.is_empty() {
                response_text = format!("Error: {}", e);
            }
        }

        self.history.push(ConversationMessage { role: "user".to_string(), content: user_msg });
        if !response_text.is_empty() {
            self.history.push(ConversationMessage { role: "assistant".to_string(), content: response_text.clone() });
        }

        for entry in tool_log {
            self.chat.push(ChatMessage { role: "Argus".to_string(), content: entry });
        }
        if !response_text.is_empty() {
            self.chat.push(ChatMessage { role: "Argus".to_string(), content: response_text });
        }

        self.state = ArgusState::Complete;
        Ok(())
    }
}

pub async fn run_tui(config: AgentConfig) -> anyhow::Result<()> {
    let mut app = App::new(config).map_err(|e| anyhow::anyhow!("{}", e))?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_event_loop(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if matches!(app.state, ArgusState::Thinking | ArgusState::Executing) {
                    continue;
                }
                app.state = ArgusState::Watching;
                match key.code {
                    KeyCode::Esc       => break,
                    KeyCode::Enter     => { if !app.input.is_empty() { app.send_message().await?; } }
                    KeyCode::Char(c)   => app.input.push(c),
                    KeyCode::Backspace => { app.input.pop(); }
                    KeyCode::Up        => app.scroll = app.scroll.saturating_sub(1),
                    KeyCode::Down      => app.scroll = app.scroll.saturating_add(1),
                    _                  => {}
                }
            }
        }
    }
    Ok(())
}

fn draw_ui(f: &mut ratatui::Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(40)])
        .split(f.size());
    draw_state_icon(f, app, main_chunks[0]);
    draw_chat(f, app, main_chunks[1]);
}

fn draw_state_icon(f: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let (icon, color, title) = match app.state {
        ArgusState::Watching  => (ARGUS_WATCHING,  Color::Cyan,    " Watching "),
        ArgusState::Thinking  => (ARGUS_THINKING,  Color::Yellow,  " Thinking "),
        ArgusState::Executing => (ARGUS_EXECUTING, Color::Magenta, " Executing "),
        ArgusState::Complete  => (ARGUS_COMPLETE,  Color::Green,   " Complete "),
    };
    let widget = Paragraph::new(icon)
        .style(Style::default().fg(color))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(title, Style::default().fg(color))),
        );
    f.render_widget(widget, area);
}

fn draw_chat(f: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled("ARGUS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("The Hundred-Eyed Agent", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(header, chunks[0]);

    let mut chat_lines: Vec<Line> = vec![];
    for msg in &app.chat {
        let (color, prefix) = if msg.role == "You" {
            (Color::Green, "\u25ba ")
        } else {
            (Color::Cyan, "\u25c9 ")
        };
        chat_lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(color)),
            Span::styled(&msg.role, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ]));
        let text_color = if msg.role == "You" { Color::White } else { Color::Gray };
        for line in msg.content.lines() {
            chat_lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(text_color),
            )));
        }
        chat_lines.push(Line::from(""));
    }

    let chat = Paragraph::new(chat_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(" Messages ", Style::default().fg(Color::Cyan))),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));
    f.render_widget(chat, chunks[1]);

    let is_busy = matches!(app.state, ArgusState::Thinking | ArgusState::Executing);
    let (input_border, input_fg, input_title) = if is_busy {
        (Color::DarkGray, Color::DarkGray, " Working... ")
    } else {
        (Color::Cyan, Color::White, " Message ")
    };
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(input_fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(input_border))
                .title(Span::styled(
                    input_title,
                    Style::default().fg(if is_busy { Color::Yellow } else { Color::Cyan }),
                )),
        );
    f.render_widget(input, chunks[2]);

    let mcp_count = app.mcp.servers.len();
    let mcp_tools: usize = app.mcp.servers.iter().map(|s| s.tools.len()).sum();
    let mcp_status = if mcp_count > 0 {
        format!(" {} MCP ({} tools) ", mcp_count, mcp_tools)
    } else {
        String::new()
    };
    let model_short = app.config.model.rsplit('/').next().unwrap_or(&app.config.model);
    let history_len = app.history.len() / 2;
    let brave_indicator = if app.config.brave_search_key.is_some() { "\u{1f50d}" } else { "\u{1f50d}\u26a0" };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(" ESC", Style::default().fg(Color::Yellow)),
        Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
        Span::styled("ENTER", Style::default().fg(Color::Yellow)),
        Span::styled(" send  ", Style::default().fg(Color::DarkGray)),
        Span::styled(&mcp_status, Style::default().fg(Color::Blue)),
        Span::styled(model_short, Style::default().fg(Color::Magenta)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(brave_indicator, Style::default().fg(Color::DarkGray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} turns", history_len), Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(status, chunks[3]);
}
