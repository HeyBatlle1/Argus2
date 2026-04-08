//! Argus Web Server — axum WebSocket + REST API for the Next.js frontend

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::Method,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tower_http::cors::{Any, CorsLayer};

use argus_core::{AgentConfig, AgentEvent, ConversationMessage, McpClient, MemoryBackend, ShellPolicy, MODEL_HAIKU, MODEL_SONNET, MODEL_OPUS, MODEL_GROK};
use argus_memory::sqlite::SqliteMemory;

// ─── WebSocket message types (mirrors TypeScript protocol) ─────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    UserMessage { content: String },
    SwitchModel { model: String },
    Cancel,
}

/// Memory record serialized to match the frontend Memory type
#[derive(Debug, Serialize, Clone)]
struct MemoryPayload {
    id: String,
    content: String,
    #[serde(rename = "type")]
    memory_type: String,
    importance: f64,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    Thinking,
    ToolCall {
        name: String,
        args: serde_json::Value,
        call_id: String,
    },
    ToolResult {
        name: String,
        result: String,
        success: bool,
        call_id: String,
    },
    ResponseComplete {
        content: String,
    },
    Error {
        message: String,
    },
    Status {
        eye_state: String,
        model: String,
    },
    Connected {
        version: String,
        model: String,
    },
    MemoryUpdate {
        memories: Vec<MemoryPayload>,
    },
}

// ─── Per-connection state ──────────────────────────────────────────────────

struct ConnectionState {
    config: AgentConfig,
    history: Vec<ConversationMessage>,
    client: reqwest::Client,
    memory: SqliteMemory,
    mcp: McpClient,
    shell_policy: ShellPolicy,
}

impl ConnectionState {
    fn new(api_key: String, brave_key: Option<String>) -> anyhow::Result<Self> {
        let mut config = AgentConfig::new(api_key);
        if let Some(k) = brave_key {
            config.brave_search_key = Some(k);
        }

        let memory = SqliteMemory::open_default()
            .map_err(|e| anyhow::anyhow!("Memory init failed: {}", e))?;

        let mut mcp = McpClient::new();
        let _ = mcp.connect_all();

        Ok(Self {
            config,
            history: Vec::new(),
            client: reqwest::Client::new(),
            memory,
            mcp,
            shell_policy: ShellPolicy::default(),
        })
    }

    /// Map frontend model ID alias → OpenRouter model ID
    fn apply_model_switch(&mut self, frontend_id: &str) {
        let openrouter_id = match frontend_id {
            "claude-haiku"  => MODEL_HAIKU,
            "claude-sonnet" => MODEL_SONNET,
            "claude-opus"   => MODEL_OPUS,
            "grok"          => MODEL_GROK,
            "gemini-flash"  => "google/gemini-2.5-flash",
            other           => other, // pass through if already a full ID
        };
        self.config.model = openrouter_id.to_string();
    }

    /// Map OpenRouter model ID → frontend alias
    fn current_frontend_model(&self) -> String {
        match self.config.model.as_str() {
            MODEL_HAIKU  => "claude-haiku".to_string(),
            MODEL_SONNET => "claude-sonnet".to_string(),
            MODEL_OPUS   => "claude-opus".to_string(),
            MODEL_GROK   => "grok".to_string(),
            "google/gemini-2.5-flash" => "gemini-flash".to_string(),
            other => other.to_string(),
        }
    }
}

// ─── Shared app state ──────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    api_key: String,
    brave_key: Option<String>,
}

// ─── Router ────────────────────────────────────────────────────────────────

pub async fn run_web_server(port: u16, config: AgentConfig) -> anyhow::Result<()> {
    let state = Arc::new(AppState {
        api_key: config.api_key,
        brave_key: config.brave_search_key,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(health))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("[+] Argus web server listening on http://localhost:{}", port);
    println!("[+] WebSocket endpoint: ws://localhost:{}/ws", port);
    println!("[+] Frontend: set NEXT_PUBLIC_WS_URL=ws://localhost:{}/ws", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "Argus online. The hundred eyes are open."
}

/// Load all memories from SQLite and emit a MemoryUpdate message.
fn build_memory_update(memory: &SqliteMemory) -> ServerMessage {
    let records = memory.recall(None, None, 100).unwrap_or_default();
    let payloads = records
        .into_iter()
        .map(|r| MemoryPayload {
            id: r.id.to_string(),
            content: r.content,
            memory_type: r.memory_type,
            importance: r.importance,
            created_at: r.created_at.unwrap_or_else(|| "unknown".to_string()),
        })
        .collect();
    ServerMessage::MemoryUpdate { memories: payloads }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

// ─── WebSocket connection handler ──────────────────────────────────────────

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (ws_tx, ws_rx) = socket.split();
    let ws_tx = Arc::new(Mutex::new(ws_tx));

    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    let ws_tx_writer = Arc::clone(&ws_tx);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("[ws] serialization error: {}", e);
                    continue;
                }
            };
            let mut sink = ws_tx_writer.lock().await;
            if sink.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let conn = match ConnectionState::new(state.api_key.clone(), state.brave_key.clone()) {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(ServerMessage::Error { message: e.to_string() });
            return;
        }
    };
    let conn = Arc::new(Mutex::new(conn));

    {
        let c = conn.lock().await;
        let _ = tx.send(ServerMessage::Connected {
            version: "0.1.0".to_string(),
            model: c.current_frontend_model(),
        });
        let _ = tx.send(build_memory_update(&c.memory));
    }

    let mut ws_rx = ws_rx;
    while let Some(Ok(msg)) = ws_rx.next().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(ServerMessage::Error {
                    message: format!("Invalid message: {}", e),
                });
                continue;
            }
        };

        match client_msg {
            ClientMessage::UserMessage { content } => {
                handle_user_message(content, Arc::clone(&conn), tx.clone()).await;
            }
            ClientMessage::SwitchModel { model } => {
                let mut c = conn.lock().await;
                c.apply_model_switch(&model);
                let new_model = c.current_frontend_model();
                let _ = tx.send(ServerMessage::Status {
                    eye_state: "watching".to_string(),
                    model: new_model,
                });
            }
            ClientMessage::Cancel => {
                let _ = tx.send(ServerMessage::Status {
                    eye_state: "watching".to_string(),
                    model: {
                        let c = conn.lock().await;
                        c.current_frontend_model()
                    },
                });
            }
        }
    }
}

// ─── Message handler ──────────────────────────────────────────────────────

async fn handle_user_message(
    user_msg: String,
    conn: Arc<Mutex<ConnectionState>>,
    tx: mpsc::UnboundedSender<ServerMessage>,
) {
    let _ = tx.send(ServerMessage::Thinking);

    let (config_snapshot, history_snapshot) = {
        let c = conn.lock().await;
        (
            (c.config.api_key.clone(), c.config.model.clone(), c.config.api_url.clone(),
             c.config.temperature, c.config.brave_search_key.clone()),
            c.history.clone(),
        )
    };

    let mut agent_config = AgentConfig::new(config_snapshot.0);
    agent_config.model = config_snapshot.1;
    agent_config.api_url = config_snapshot.2;
    agent_config.temperature = config_snapshot.3;
    agent_config.brave_search_key = config_snapshot.4;

    let tx_clone = tx.clone();

    let result: Result<String, String> = {
        let mut c = conn.lock().await;
        let mut response_text = String::new();

        let ConnectionState {
            ref shell_policy,
            ref memory,
            ref mut mcp,
            ref client,
            ..
        } = *c;
        let mem: &dyn argus_core::MemoryBackend = memory;

        let r = argus_core::run_agent_turn(
            &agent_config,
            &user_msg,
            &history_snapshot,
            shell_policy,
            mem,
            mcp,
            client,
            |event| {
                match event {
                    AgentEvent::Thinking => {
                        let _ = tx_clone.send(ServerMessage::Thinking);
                    }
                    AgentEvent::ToolCall { id, name, args, .. } => {
                        let _ = tx_clone.send(ServerMessage::ToolCall {
                            name,
                            args,
                            call_id: id,
                        });
                    }
                    AgentEvent::ToolResult { id, name, result, success, .. } => {
                        let _ = tx_clone.send(ServerMessage::ToolResult {
                            name,
                            result,
                            success,
                            call_id: id,
                        });
                    }
                    AgentEvent::Response(text) => {
                        response_text = text;
                    }
                    AgentEvent::Error(err) => {
                        let _ = tx_clone.send(ServerMessage::Error { message: err });
                    }
                }
            },
        ).await;

        match r {
            Ok(text) => {
                c.history.push(ConversationMessage {
                    role: "user".to_string(),
                    content: user_msg.clone(),
                });
                c.history.push(ConversationMessage {
                    role: "assistant".to_string(),
                    content: text.clone(),
                });
                if c.history.len() > 40 {
                    let drain = c.history.len() - 40;
                    c.history.drain(0..drain);
                }
                Ok(text)
            }
            Err(e) => {
                if response_text.is_empty() {
                    Err(e)
                } else {
                    Ok(response_text)
                }
            }
        }
    };

    match result {
        Ok(content) => {
            let _ = tx.send(ServerMessage::ResponseComplete { content });
            let (frontend_model, memory_update) = {
                let c = conn.lock().await;
                (c.current_frontend_model(), build_memory_update(&c.memory))
            };
            let _ = tx.send(memory_update);
            let _ = tx.send(ServerMessage::Status {
                eye_state: "watching".to_string(),
                model: frontend_model,
            });
        }
        Err(err) => {
            let _ = tx.send(ServerMessage::Error { message: err });
            let _ = tx.send(ServerMessage::Status {
                eye_state: "watching".to_string(),
                model: {
                    let c = conn.lock().await;
                    c.current_frontend_model()
                },
            });
        }
    }
}
