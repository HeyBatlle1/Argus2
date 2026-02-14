//! Interactive TUI for Argus
//!
//! Ratatui-based terminal interface. Left pane shows Argus state icon,
//! right pane is the chat. Uses the shared agent loop from argus-core
//! so all tool execution, memory, and MCP are handled uniformly.

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

use argus_core::{AgentConfig, AgentEvent, McpClient, ShellPolicy};
use argus_memory::SqliteMemory;

// ---------------------------------------------------------------------------
// Compact state icons — diamond grid of eyes, 5 lines each.
// Center glyph changes per state. Rendered with Alignment::Center.
// ---------------------------------------------------------------------------

const ARGUS_WATCHING: &str = "\
\n\
◉\n\
◉ ◉ ◉\n\
◉ ◉ ◉ ◉ ◉\n\
◉ ◉ ◉\n\
◉\n\
\n\
ALL EYES OPEN";

const ARGUS_THINKING: &str = "\
\n\
◎\n\
◎ ◎ ◎\n\
◎ ◎ ◎ ◎ ◎\n\
◎ ◎ ◎\n\
◎\n\
\n\
THINKING...";

const ARGUS_EXECUTING: &str = "\
\n\
⊙\n\
⊙ ⊙ ⊙\n\
⊙ ⊙ ⚡ ⊙ ⊙\n\
⊙ ⊙ ⊙\n\
⊙\n\
\n\
EXECUTING";

const ARGUS_COMPLETE: &str = "\
\n\
✦\n\
✦ ✦ ✦\n\
✦ ✦ ✓ ✦ ✦\n\
✦ ✦ ✦\n\
✦\n\
\n\
COMPLETE";

// ---------------------------------------------------------------------------
// State, messages, and application
// ---------------------------------------------------------------------------

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
    messages: Vec<ChatMessage>,
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
    fn new(api_key: String) -> Result<Self, String> {
        let memory = SqliteMemory::open_default()
            .map_err(|e| format!("Failed to open memory: {}", e))?;

        let mut mcp = McpClient::new();
        let mcp_errors = mcp.connect_all();
        for err in &mcp_errors {
            eprintln!("MCP: {}", err);
        }

        Ok(Self {
            messages: vec![],
            input: String::new(),
            scroll: 0,
            config: AgentConfig::new(api_key),
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
        self.messages.push(ChatMessage {
            role: "You".to_string(),
            content: user_msg.clone(),
        });
        self.input.clear();
        self.state = ArgusState::Thinking;

        let mut response_text = String::new();
        let mut tool_log: Vec<String> = Vec::new();

        let result = argus_core::run_agent_turn(
            &self.config,
            &user_msg,
            &self.shell_policy,
            &self.memory,
            &mut self.mcp,
            &self.client,
            |event| match event {
                AgentEvent::ToolCall { name, preview } => {
                    self.state = ArgusState::Executing;
                    let short = if preview.len() > 60 {
                        format!("{}...", &preview[..60])
                    } else {
                        preview
                    };
                    tool_log.push(format!("🔧 {}: {}", name, short));
                }
                AgentEvent::Response(text) => {
                    response_text = text;
                }
                AgentEvent::Error(err) => {
                    response_text = format!("❌ {}", err);
                }
                _ => {}
            },
        )
        .await;

        if let Err(e) = result {
            if response_text.is_empty() {
                response_text = format!("Error: {}", e);
            }
        }

        // Show tool activity
        for entry in tool_log {
            self.messages.push(ChatMessage {
                role: "Grok".to_string(),
                content: entry,
            });
        }

        if !response_text.is_empty() {
            self.messages.push(ChatMessage {
                role: "Grok".to_string(),
                content: response_text,
            });
        }

        self.state = ArgusState::Complete;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// TUI entrypoint
// ---------------------------------------------------------------------------

pub async fn run_tui(api_key: String) -> anyhow::Result<()> {
    let mut app = App::new(api_key)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_event_loop(&mut terminal, &mut app).await;

    // Always restore terminal, even on error
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
                // Reset to watching when user types
                app.state = ArgusState::Watching;
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            app.send_message().await?;
                        }
                    }
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => { app.input.pop(); }
                    KeyCode::Up => app.scroll = app.scroll.saturating_sub(1),
                    KeyCode::Down => app.scroll = app.scroll.saturating_add(1),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn draw_ui(f: &mut ratatui::Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // state icon
            Constraint::Min(40),   // chat area
        ])
        .split(f.size());

    draw_state_icon(f, app, main_chunks[0]);
    draw_chat(f, app, main_chunks[1]);
}

fn draw_state_icon(f: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let (icon, color, title) = match app.state {
        ArgusState::Watching  => (ARGUS_WATCHING,   Color::Cyan,    " Watching "),
        ArgusState::Thinking  => (ARGUS_THINKING,   Color::Yellow,  " Thinking "),
        ArgusState::Executing => (ARGUS_EXECUTING,  Color::Magenta, " Executing "),
        ArgusState::Complete  => (ARGUS_COMPLETE,    Color::Green,   " Complete "),
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
            Constraint::Length(3), // header
            Constraint::Min(10),  // messages
            Constraint::Length(3), // input
            Constraint::Length(1), // status bar
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "ARGUS",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("The Hundred-Eyed Agent", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(header, chunks[0]);

    // Messages
    let mut chat_lines: Vec<Line> = vec![];
    for msg in &app.messages {
        let (color, prefix) = if msg.role == "You" {
            (Color::Green, "► ")
        } else {
            (Color::Cyan, "◉ ")
        };
        chat_lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(color)),
            Span::styled(
                &msg.role,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));
        let text_color = if msg.role == "You" {
            Color::White
        } else {
            Color::Gray
        };
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
                .title(Span::styled(
                    " Messages ",
                    Style::default().fg(Color::Cyan),
                )),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));
    f.render_widget(chat, chunks[1]);

    // Input
    let is_busy = matches!(app.state, ArgusState::Thinking | ArgusState::Executing);
    let (input_border, input_fg, input_title) = if is_busy {
        (Color::DarkGray, Color::DarkGray, " Wait... ")
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

    // Status bar
    let mcp_count = app.mcp.servers.len();
    let mcp_tools: usize = app.mcp.servers.iter().map(|s| s.tools.len()).sum();
    let mcp_status = if mcp_count > 0 {
        format!(" {} MCP ({} tools) ", mcp_count, mcp_tools)
    } else {
        String::new()
    };

    let model_short = app
        .config
        .model
        .rsplit('/')
        .next()
        .unwrap_or(&app.config.model);

    let status = Paragraph::new(Line::from(vec![
        Span::styled(" ESC", Style::default().fg(Color::Yellow)),
        Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
        Span::styled("ENTER", Style::default().fg(Color::Yellow)),
        Span::styled(" send ", Style::default().fg(Color::DarkGray)),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::styled(" scroll ", Style::default().fg(Color::DarkGray)),
        Span::styled("| ", Style::default().fg(Color::DarkGray)),
        Span::styled(&mcp_status, Style::default().fg(Color::Blue)),
        Span::styled("| ", Style::default().fg(Color::DarkGray)),
        Span::styled(model_short, Style::default().fg(Color::Magenta)),
    ]));
    f.render_widget(status, chunks[3]);
}
