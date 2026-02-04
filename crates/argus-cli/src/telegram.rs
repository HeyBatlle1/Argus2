//! Telegram Bot for Argus
//! Run Grok via Telegram with full tool access

use teloxide::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::memory::ArgusMemory;
use crate::mcp::McpClient;

pub struct ArgusBot {
    api_key: String,
    client: reqwest::Client,
    memory: ArgusMemory,
    mcp: Arc<Mutex<McpClient>>,
}

impl ArgusBot {
    pub fn new(api_key: String) -> Self {
        let mut mcp = McpClient::new();
        let _ = mcp.connect_all();
        
        Self {
            api_key,
            client: reqwest::Client::new(),
            memory: ArgusMemory::new(),
            mcp: Arc::new(Mutex::new(mcp)),
        }
    }
    
    pub async fn process_message(&self, user_msg: &str) -> String {
        // Build tools list (same as TUI)
        let mut tools = serde_json::json!([
            {
                "type": "function",
                "function": {
                    "name": "web_search",
                    "description": "Search Google for current information, news, facts.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": { "type": "string", "description": "The search query" }
                        },
                        "required": ["query"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "remember",
                    "description": "Store information in persistent memory.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "content": { "type": "string", "description": "The information to remember" },
                            "type": { "type": "string", "enum": ["fact", "preference", "task", "learning"] },
                            "importance": { "type": "number", "description": "1-10 importance" }
                        },
                        "required": ["content"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "recall",
                    "description": "Retrieve memories.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": { "type": "string", "description": "Search term" },
                            "limit": { "type": "number", "description": "Max results" }
                        }
                    }
                }
            }
        ]);

        // Add MCP tools
        {
            let mcp = self.mcp.lock().await;
            if let Some(arr) = tools.as_array_mut() {
                for mcp_tool in mcp.all_tools() {
                    arr.push(serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": mcp_tool.name,
                            "description": mcp_tool.description.clone().unwrap_or_default(),
                            "parameters": mcp_tool.input_schema
                        }
                    }));
                }
            }
        }

        // Call Grok
        let resp = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "x-ai/grok-4.1-fast",
                "messages": [
                    {
                        "role": "system",
                        "content": "You are Grok, made by xAI. You're chatting via Telegram through ARGUS.

IDENTITY: You are Grok (xAI). ARGUS is just the framework.

TOOLS: web_search, remember, recall, plus any MCP tools.

Keep responses concise for mobile. Use tools when helpful."
                    },
                    {"role": "user", "content": user_msg}
                ],
                "tools": tools,
                "tool_choice": "auto",
                "temperature": 0.7
            }))
            .send()
            .await;

        let json: serde_json::Value = match resp {
            Ok(r) => match r.json().await {
                Ok(j) => j,
                Err(e) => return format!("❌ Error: {}", e),
            },
            Err(e) => return format!("❌ Error: {}", e),
        };

        // Handle tool calls
        if let Some(tool_calls) = json["choices"][0]["message"]["tool_calls"].as_array() {
            let mut results = Vec::new();
            
            for tool_call in tool_calls {
                let name = tool_call["function"]["name"].as_str().unwrap_or("");
                let args: serde_json::Value = serde_json::from_str(
                    tool_call["function"]["arguments"].as_str().unwrap_or("{}")
                ).unwrap_or(serde_json::json!({}));
                
                let result = self.execute_tool(name, &args).await;
                results.push((name.to_string(), result));
            }
            
            // Get follow-up response
            let tool_results: Vec<serde_json::Value> = tool_calls.iter().zip(results.iter())
                .map(|(tc, (_, result))| {
                    serde_json::json!({
                        "role": "tool",
                        "tool_call_id": tc["id"].as_str().unwrap_or(""),
                        "content": result
                    })
                })
                .collect();

            let mut messages = vec![
                serde_json::json!({"role": "system", "content": "You are Grok (xAI) via Telegram. Be concise."}),
                serde_json::json!({"role": "user", "content": user_msg}),
                json["choices"][0]["message"].clone(),
            ];
            messages.extend(tool_results);

            let follow_up = self.client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "model": "x-ai/grok-4.1-fast",
                    "messages": messages,
                    "temperature": 0.7
                }))
                .send()
                .await;

            if let Ok(r) = follow_up {
                if let Ok(j) = r.json::<serde_json::Value>().await {
                    return j["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("Done.")
                        .to_string();
                }
            }
            
            return "Tool executed.".to_string();
        }

        // Direct response
        json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("...")
            .to_string()
    }

    async fn execute_tool(&self, name: &str, args: &serde_json::Value) -> String {
        match name {
            "web_search" => {
                let query = args["query"].as_str().unwrap_or("");
                let encoded = urlencoding::encode(query);
                let url = format!("https://html.duckduckgo.com/html/?q={}", encoded);
                
                match self.client.get(&url)
                    .header("User-Agent", "Mozilla/5.0")
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if let Ok(html) = resp.text().await {
                            let mut results = Vec::new();
                            for (i, chunk) in html.split("result__snippet").enumerate() {
                                if i == 0 || i > 3 { continue; }
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
                                            results.push(clean.trim().to_string());
                                        }
                                    }
                                }
                            }
                            if results.is_empty() {
                                "No results found".to_string()
                            } else {
                                results.join("\n\n")
                            }
                        } else {
                            "Search failed".to_string()
                        }
                    }
                    Err(e) => format!("Error: {}", e),
                }
            }
            "remember" => {
                let content = args["content"].as_str().unwrap_or("");
                let mem_type = args["type"].as_str().unwrap_or("fact");
                let importance = args["importance"].as_f64().unwrap_or(5.0);
                
                match self.memory.remember(mem_type, content, None, importance, None) {
                    Ok(msg) => msg,
                    Err(e) => format!("Memory error: {}", e),
                }
            }
            "recall" => {
                let query = args["query"].as_str();
                let limit = args["limit"].as_u64().unwrap_or(5) as usize;
                
                match self.memory.recall(query, None, limit) {
                    Ok(memories) => {
                        if memories.is_empty() {
                            "No memories found.".to_string()
                        } else {
                            memories.iter()
                                .map(|m| format!("[{}] {}", m.memory_type, m.content))
                                .collect::<Vec<_>>()
                                .join("\n")
                        }
                    }
                    Err(e) => format!("Error: {}", e),
                }
            }
            _ => {
                // Try MCP
                let mut mcp = self.mcp.lock().await;
                match mcp.call_tool(name, args.clone()) {
                    Ok(result) => result,
                    Err(_) => format!("Unknown tool: {}", name),
                }
            }
        }
    }
}

pub async fn run_telegram_bot(token: String, api_key: String) {
    println!("🤖 Argus Telegram bot starting...");
    
    let bot = Bot::new(token);
    let argus = Arc::new(ArgusBot::new(api_key));
    
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let argus = Arc::clone(&argus);
        async move {
            if let Some(text) = msg.text() {
                let response = argus.process_message(text).await;
                
                // Split long messages (Telegram limit is 4096 chars)
                for chunk in response.chars().collect::<Vec<_>>().chunks(4000) {
                    let chunk_str: String = chunk.iter().collect();
                    bot.send_message(msg.chat.id, chunk_str).await?;
                }
            }
            Ok(())
        }
    })
    .await;
}
