//! Argus CLI - The Hundred-Eyed Agent

mod memory;

use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io::{self, Write};
use std::path::PathBuf;
use argus_crypto::SecureVault;
use memory::ArgusMemory;

const ARGUS_WATCHING: &str = r#"
        ╭──◉──╮
       ╭┤◉ ◉ ◉├╮
      ◉│ ╭───╮ │◉
      ◉│ │◉ ◉│ │◉
      ◉│ │ ▽ │ │◉
       │ ╰───╯ │
    ◉──┤ ◉ ◉ ◉ ├──◉
   ╭───┤       ├───╮
   │◉◉◉│ ◉ ◉ ◉ │◉◉◉│
   ╰───┤       ├───╯
       │ ◉   ◉ │
       ╰┬─┴─┬─╯
        │   │
       ─┴─ ─┴─
   ═══════════════
    👁 ALL EYES OPEN"#;

const ARGUS_THINKING: &str = r#"
        ╭──◎──╮
       ╭┤◎ ◎ ◎├╮
      ◎│ ╭───╮ │◎
      ◎│ │⊛ ⊛│ │◎
      ◎│ │ ─ │ │◎
       │ ╰───╯ │
    ◎──┤ ◎ ◎ ◎ ├──◎
   ╭───┤ ≋≋≋≋≋ ├───╮
   │◎◎◎│ ◎ ◎ ◎ │◎◎◎│
   ╰───┤       ├───╯
       │ ◎   ◎ │
       ╰┬─┴─┬─╯
        │   │
       ─┴─ ─┴─
   ═══════════════
    ⟳ THINKING..."#;

const ARGUS_ALERT: &str = r#"
        ╭──⊙──╮
       ╭┤⊙ ⊙ ⊙├╮
      ⊙│ ╭───╮ │⊙
      ⊙│ │⊚ ⊚│ │⊙
      ⊙│ │ ! │ │⊙
       │ ╰───╯ │
    ⊙──┤ ⊙ ⊙ ⊙ ├──⊙
   ╭───┤ ▓▓▓▓▓ ├───╮
   │⊙⊙⊙│ ⊙ ⊙ ⊙ │⊙⊙⊙│
   ╰───┤       ├───╯
       │ ⊙   ⊙ │
       ╰┬─┴─┬─╯
        │   │
       ─┴─ ─┴─
   ═══════════════
    ⚡ EXECUTING"#;

const ARGUS_SUCCESS: &str = r#"
        ╭──✦──╮
       ╭┤✦ ✦ ✦├╮
      ✦│ ╭───╮ │✦
      ✦│ │◉ ◉│ │✦
      ✦│ │ ◡ │ │✦
       │ ╰───╯ │
    ✦──┤ ✦ ✦ ✦ ├──✦
   ╭───┤ ░░░░░ ├───╮
   │✦✦✦│ ✦ ✦ ✦ │✦✦✦│
   ╰───┤       ├───╯
       │ ✦   ✦ │
       ╰┬─┴─┬─╯
        │   │
       ─┴─ ─┴─
   ═══════════════
    ✓ COMPLETE"#;

const LOGO: &str = r#"
    ___    ____  ______  __  _______
   /   |  / __ \/ ____/ / / / / ___/
  / /| | / /_/ / / __  / / / /\__ \ 
 / ___ |/ _, _/ /_/ / / /_/ /___/ / 
/_/  |_/_/ |_|\____/  \____//____/  
"#;

#[derive(Parser)]
#[command(name = "argus", version, about = "The hundred-eyed agent runtime")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Run,
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
}

#[derive(Subcommand)]
enum VaultAction {
    Set { key: String },
    Get { key: String },
    List,
    Delete { key: String },
}

fn vault_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".argus").join("vault.enc")
}

#[derive(Clone, Copy, PartialEq)]
enum ArgusState {
    Watching,
    Thinking,
    ToolActive,
    Success,
}

struct ChatMessage {
    role: String,
    content: String,
}

struct App {
    messages: Vec<ChatMessage>,
    input: String,
    scroll: u16,
    api_key: String,
    client: reqwest::Client,
    state: ArgusState,
    memory: ArgusMemory,
}

impl App {
    fn new(api_key: String) -> Self {
        Self {
            messages: vec![],
            input: String::new(),
            scroll: 0,
            api_key,
            client: reqwest::Client::new(),
            state: ArgusState::Watching,
            memory: ArgusMemory::new("default"),
        }
    }

    async fn execute_tool(&self, name: &str, args: &serde_json::Value) -> String {
        match name {
            "read_file" => {
                let path = args["path"].as_str().unwrap_or("");
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        if content.len() > 2000 {
                            format!("{}...\n[truncated, {} bytes total]", &content[..2000], content.len())
                        } else {
                            content
                        }
                    }
                    Err(e) => format!("Error reading file: {}", e),
                }
            }
            "list_directory" => {
                let path = args["path"].as_str().unwrap_or(".");
                match std::fs::read_dir(path) {
                    Ok(entries) => {
                        let mut result = String::new();
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                            result.push_str(&format!("{}{}\n", if is_dir { "📁 " } else { "📄 " }, name));
                        }
                        result
                    }
                    Err(e) => format!("Error listing directory: {}", e),
                }
            }
            "write_file" => {
                let path = args["path"].as_str().unwrap_or("");
                let content = args["content"].as_str().unwrap_or("");
                match std::fs::write(path, content) {
                    Ok(_) => format!("✅ Written {} bytes to {}", content.len(), path),
                    Err(e) => format!("Error writing file: {}", e),
                }
            }
            "shell" => {
                let command = args["command"].as_str().unwrap_or("");
                // Safety: limit dangerous commands
                let dangerous = ["rm -rf /", "sudo", "mkfs", "dd if=", "> /dev/"];
                if dangerous.iter().any(|d| command.contains(d)) {
                    return "⛔ Command blocked for safety".to_string();
                }
                match std::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if output.status.success() {
                            if stdout.len() > 2000 {
                                format!("{}...\n[truncated]", &stdout[..2000])
                            } else {
                                stdout.to_string()
                            }
                        } else {
                            format!("Exit {}: {}", output.status.code().unwrap_or(-1), stderr)
                        }
                    }
                    Err(e) => format!("Error executing: {}", e),
                }
            }
            "web_search" => {
                let query = args["query"].as_str().unwrap_or("");
                // Use Google search with a realistic user agent
                let url = format!(
                    "https://www.google.com/search?q={}&num=10",
                    urlencoding::encode(query)
                );
                let client = reqwest::Client::builder()
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                    .build()
                    .unwrap_or_default();
                    
                match client.get(&url).send().await {
                    Ok(resp) => {
                        match resp.text().await {
                            Ok(html) => {
                                let mut results = Vec::new();
                                // Parse Google results - look for data-sncf divs or BNeawe class
                                for part in html.split("<div class=\"BNeawe") {
                                    if results.len() >= 8 { break; }
                                    // Find text content
                                    if let Some(start) = part.find('>') {
                                        let after_tag = &part[start+1..];
                                        if let Some(end) = after_tag.find('<') {
                                            let text = &after_tag[..end];
                                            let clean: String = text
                                                .replace("&quot;", "\"")
                                                .replace("&amp;", "&")
                                                .replace("&lt;", "<")
                                                .replace("&gt;", ">")
                                                .replace("&#39;", "'")
                                                .chars()
                                                .filter(|c| !c.is_control())
                                                .collect();
                                            let trimmed = clean.trim();
                                            // Filter out short/junk results
                                            if trimmed.len() > 40 && !trimmed.contains("http") {
                                                results.push(format!("• {}", trimmed));
                                            }
                                        }
                                    }
                                }
                                if results.is_empty() {
                                    // Fallback: try DuckDuckGo
                                    let ddg_url = format!(
                                        "https://html.duckduckgo.com/html/?q={}",
                                        urlencoding::encode(query)
                                    );
                                    if let Ok(ddg_resp) = reqwest::get(&ddg_url).await {
                                        if let Ok(ddg_html) = ddg_resp.text().await {
                                            for (i, chunk) in ddg_html.split("result__snippet").enumerate() {
                                                if i == 0 || i > 5 { continue; }
                                                if let Some(start) = chunk.find('>') {
                                                    if let Some(end) = chunk[start..].find('<') {
                                                        let snippet = &chunk[start+1..start+end];
                                                        let clean: String = snippet
                                                            .replace("&quot;", "\"")
                                                            .replace("&amp;", "&")
                                                            .chars()
                                                            .filter(|c| !c.is_control())
                                                            .collect();
                                                        if clean.len() > 20 {
                                                            results.push(format!("• {}", clean.trim()));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if results.is_empty() {
                                    "No results found - try rephrasing your search".to_string()
                                } else {
                                    format!("🔍 Search results for '{}':\n\n{}", query, results.join("\n\n"))
                                }
                            }
                            Err(e) => format!("Error reading response: {}", e),
                        }
                    }
                    Err(e) => format!("Error searching: {}", e),
                }
            }
            "remember" => {
                let content = args["content"].as_str().unwrap_or("");
                let memory_type = args["type"].as_str().unwrap_or("fact");
                let importance = args["importance"].as_f64().unwrap_or(5.0);
                let reasoning = args["reasoning"].as_str();
                
                match self.memory.remember(memory_type, content, reasoning, importance, None).await {
                    Ok(msg) => msg,
                    Err(e) => format!("❌ Memory error: {}", e),
                }
            }
            "recall" => {
                let query = args["query"].as_str();
                let memory_type = args["type"].as_str();
                let limit = args["limit"].as_u64().unwrap_or(10) as usize;
                
                match self.memory.recall(query, memory_type, limit).await {
                    Ok(memories) => {
                        if memories.is_empty() {
                            "No memories found.".to_string()
                        } else {
                            let mut result = String::from("🧠 Recalled memories:\n\n");
                            for mem in memories {
                                result.push_str(&format!(
                                    "• [{}] (importance: {:.1}): {}\n",
                                    mem.memory_type, mem.importance, mem.content
                                ));
                            }
                            result
                        }
                    }
                    Err(e) => format!("❌ Recall error: {}", e),
                }
            }
            "forget" => {
                let content_match = args["content_match"].as_str().unwrap_or("");
                match self.memory.forget(content_match).await {
                    Ok(msg) => msg,
                    Err(e) => format!("❌ Forget error: {}", e),
                }
            }
            _ => format!("Unknown tool: {}", name),
        }
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

        let tools = serde_json::json!([
            {
                "type": "function",
                "function": {
                    "name": "read_file",
                    "description": "Read the contents of a file from the filesystem",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The path to the file to read"
                            }
                        },
                        "required": ["path"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "list_directory",
                    "description": "List files and directories in a given path",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The directory path to list"
                            }
                        },
                        "required": ["path"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "write_file",
                    "description": "Write content to a file",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The path to write to"
                            },
                            "content": {
                                "type": "string",
                                "description": "The content to write"
                            }
                        },
                        "required": ["path", "content"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "shell",
                    "description": "Execute a shell command and return output. Use for system tasks.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The shell command to execute"
                            }
                        },
                        "required": ["command"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "web_search",
                    "description": "Search Google for current information, news, facts, or anything you don't know. Always use this for questions about recent events, people, or things that may have changed.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "The search query"
                            }
                        },
                        "required": ["query"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "remember",
                    "description": "Store information in persistent memory. Use for facts, preferences, important details about the user, or anything worth remembering across sessions.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "The information to remember"
                            },
                            "type": {
                                "type": "string",
                                "enum": ["fact", "preference", "task", "learning", "relationship"],
                                "description": "Category of memory"
                            },
                            "importance": {
                                "type": "number",
                                "description": "Importance score 1-10"
                            },
                            "reasoning": {
                                "type": "string",
                                "description": "Why this is worth remembering"
                            }
                        },
                        "required": ["content", "type"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "recall",
                    "description": "Search and retrieve memories. Use at the start of conversations or when you need context about the user.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search term to find relevant memories"
                            },
                            "type": {
                                "type": "string",
                                "enum": ["fact", "preference", "task", "learning", "relationship"],
                                "description": "Filter by memory type"
                            },
                            "limit": {
                                "type": "number",
                                "description": "Max memories to return (default 10)"
                            }
                        }
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "forget",
                    "description": "Delete memories matching a search term. Use when user asks to forget something or information is outdated.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "content_match": {
                                "type": "string",
                                "description": "Text to match for deletion"
                            }
                        },
                        "required": ["content_match"]
                    }
                }
            }
        ]);

        let resp = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "x-ai/grok-4-fast",
                "messages": [
                    {
                        "role": "system", 
                        "content": "You are ARGUS - The Hundred-Eyed Agent. Named after Argus Panoptes, the all-seeing giant of Greek mythology, you are a secure, tool-equipped AI agent built in Rust.

Your capabilities via tools:
• read_file: Read any file's contents
• write_file: Create or modify files  
• list_directory: See what's in folders
• shell: Execute system commands (safely)
• web_search: Search Google for current info, news, facts
• remember: Store facts, preferences, learnings in persistent memory
• recall: Retrieve memories - use this to remember context about users
• forget: Delete outdated or unwanted memories

CRITICAL RULES:
1. ALWAYS use tools when relevant - you are an AGENT, not just a chatbot
2. For ANY question about current events, news, prices, weather, or recent info → use web_search
3. For file/system questions → use the appropriate file/shell tool
4. Use 'recall' at conversation start to load relevant context about the user
5. Use 'remember' to store important facts, preferences, or learnings
6. Never say 'I cannot' if you have a tool that can help
7. Be direct, efficient, and action-oriented

You run locally with encrypted secrets, hardware keychain integration, memory-safe Rust, and Supabase-backed persistent memory. You remember users across sessions."
                    },
                    {"role": "user", "content": user_msg}
                ],
                "tools": tools,
                "tool_choice": "auto"
            }))
            .send()
            .await?;

        let json: serde_json::Value = resp.json().await?;
        
        // Check for tool calls
        if let Some(tool_calls) = json["choices"][0]["message"]["tool_calls"].as_array() {
            for tool_call in tool_calls {
                let name = tool_call["function"]["name"].as_str().unwrap_or("");
                let args: serde_json::Value = serde_json::from_str(
                    tool_call["function"]["arguments"].as_str().unwrap_or("{}")
                ).unwrap_or(serde_json::json!({}));
                
                self.state = ArgusState::ToolActive;
                let result = self.execute_tool(name, &args).await;
                
                self.messages.push(ChatMessage {
                    role: "Argus".to_string(),
                    content: format!("🔧 Tool: {}\n📤 Result:\n{}", name, result),
                });
            }
            self.state = ArgusState::Success;
        } else {
            let content = json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("Error: No response")
                .to_string();

            self.messages.push(ChatMessage {
                role: "Argus".to_string(),
                content,
            });
            self.state = ArgusState::Success;
        }
        // State will reset to Watching on next input

        Ok(())
    }
}

async fn run_tui(api_key: String) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(api_key);

    loop {
        terminal.draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(22),  // Argus avatar
                    Constraint::Min(40),     // Chat area
                ])
                .split(f.size());

            // Left side - Argus avatar
            let (avatar, avatar_color, status_text) = match app.state {
                ArgusState::Watching => (ARGUS_WATCHING, Color::Cyan, " Watching "),
                ArgusState::Thinking => (ARGUS_THINKING, Color::Yellow, " Thinking... "),
                ArgusState::ToolActive => (ARGUS_ALERT, Color::Magenta, " Tool Active "),
                ArgusState::Success => (ARGUS_SUCCESS, Color::Green, " Complete "),
            };
            let avatar_widget = Paragraph::new(avatar)
                .style(Style::default().fg(avatar_color))
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(Span::styled(status_text, Style::default().fg(avatar_color))));
            f.render_widget(avatar_widget, main_chunks[0]);

            // Right side - chat
            let chat_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Min(10),    // Chat
                    Constraint::Length(3),  // Input
                    Constraint::Length(1),  // Status
                ])
                .split(main_chunks[1]);

            // Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled("👁️  ", Style::default().fg(Color::Cyan)),
                Span::styled("ARGUS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                Span::styled("The Hundred-Eyed Agent", Style::default().fg(Color::DarkGray)),
            ]))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(header, chat_chunks[0]);

            // Chat messages
            let mut chat_lines: Vec<Line> = vec![];
            for msg in &app.messages {
                let (color, prefix) = if msg.role == "You" {
                    (Color::Green, "► ")
                } else {
                    (Color::Cyan, "◉ ")
                };
                chat_lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(color)),
                    Span::styled(&msg.role, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                ]));
                for line in msg.content.lines() {
                    chat_lines.push(Line::from(Span::styled(
                        format!("  {}", line),
                        Style::default().fg(if msg.role == "You" { Color::White } else { Color::Gray }),
                    )));
                }
                chat_lines.push(Line::from(""));
            }

            let chat = Paragraph::new(chat_lines)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(Span::styled(" Messages ", Style::default().fg(Color::Cyan))))
                .wrap(Wrap { trim: false })
                .scroll((app.scroll, 0));
            f.render_widget(chat, chat_chunks[1]);

            // Input - always visible
            let is_busy = matches!(app.state, ArgusState::Thinking | ArgusState::ToolActive);
            let input_style = if is_busy {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };
            let input = Paragraph::new(app.input.as_str())
                .style(input_style)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if is_busy { Color::DarkGray } else { Color::Cyan }))
                    .title(Span::styled(
                        if is_busy { " Wait... " } else { " Message " },
                        Style::default().fg(if is_busy { Color::Yellow } else { Color::Cyan })
                    )));
            f.render_widget(input, chat_chunks[2]);

            // Status bar
            let status = Paragraph::new(Line::from(vec![
                Span::styled(" ESC", Style::default().fg(Color::Yellow)),
                Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
                Span::styled("ENTER", Style::default().fg(Color::Yellow)),
                Span::styled(" send ", Style::default().fg(Color::DarkGray)),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::styled(" scroll ", Style::default().fg(Color::DarkGray)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("🔐 ", Style::default().fg(Color::Green)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("arcee-ai/trinity-mini", Style::default().fg(Color::Magenta)),
            ]));
            f.render_widget(status, chat_chunks[3]);
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if matches!(app.state, ArgusState::Thinking | ArgusState::ToolActive) {
                    continue; // Ignore input while busy
                }
                // Reset to watching when user starts typing
                app.state = ArgusState::Watching;
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            app.send_message().await?;
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Up => {
                        app.scroll = app.scroll.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        app.scroll = app.scroll.saturating_add(1);
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("{}", LOGO);
            println!("👁️  Initializing Argus...\n");
            let path = vault_path();
            if path.exists() {
                println!("✅ Vault already exists at {:?}", path);
                return Ok(());
            }
            SecureVault::init(path)?;
            println!("✅ Vault created.");
            println!("✅ Master key stored in system keychain.");
            println!("\n🔐 Your secrets are encrypted. Not plaintext. Not ever.\n");
            println!("Next: argus vault set OPENROUTER_KEY <your-key>");
        }
        Commands::Run => {
            let mut vault = SecureVault::new(vault_path());
            vault.unlock()?;
            
            let api_key = vault.retrieve("OPENROUTER_KEY").map_err(|_| {
                anyhow::anyhow!("No OPENROUTER_KEY found. Run: argus vault set OPENROUTER_KEY")
            })?;

            run_tui(api_key).await?;
        }
        Commands::Vault { action } => {
            let mut vault = SecureVault::new(vault_path());
            vault.unlock()?;
            
            match action {
                VaultAction::Set { key } => {
                    print!("Enter secret value: ");
                    io::stdout().flush()?;
                    let mut value = String::new();
                    io::stdin().read_line(&mut value)?;
                    vault.store(&key, value.trim())?;
                    println!("✅ Stored: {}", key);
                }
                VaultAction::Get { key } => {
                    match vault.retrieve(&key) {
                        Ok(v) => println!("{}", v),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                VaultAction::List => {
                    println!("\n🔐 Stored secrets:");
                    for k in vault.list_keys() {
                        println!("   • {}", k);
                    }
                    println!();
                }
                VaultAction::Delete { key } => {
                    vault.delete(&key)?;
                    println!("✅ Deleted: {}", key);
                }
            }
        }
    }
    Ok(())
}
