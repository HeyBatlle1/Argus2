//! MCP Client - Connect to Model Context Protocol servers
//! Handles JSON-RPC 2.0 over stdio transport

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    #[serde(skip)]
    pub server_name: String,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<u64>,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    #[allow(dead_code)]
    code: i32,
    message: String,
}

pub struct McpServer {
    pub name: String,
    process: Child,
    pub tools: Vec<McpTool>,
}

impl McpServer {
    pub fn connect(config: &McpServerConfig) -> Result<Self, String> {
        // Spawn the server process
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        
        for (k, v) in &config.env {
            cmd.env(k, v);
        }
        
        let mut process = cmd.spawn()
            .map_err(|e| format!("Failed to spawn MCP server '{}': {}", config.name, e))?;
        
        let mut server = Self {
            name: config.name.clone(),
            process,
            tools: vec![],
        };
        
        // Initialize the connection
        server.initialize()?;
        
        // Get available tools
        server.list_tools()?;
        
        Ok(server)
    }
    
    fn send_request(&mut self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
        let id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        };
        
        let stdin = self.process.stdin.as_mut()
            .ok_or("Failed to get stdin")?;
        
        let request_json = serde_json::to_string(&request)
            .map_err(|e| e.to_string())?;
        
        writeln!(stdin, "{}", request_json)
            .map_err(|e| format!("Failed to write to MCP server: {}", e))?;
        stdin.flush().map_err(|e| e.to_string())?;
        
        // Read response
        let stdout = self.process.stdout.as_mut()
            .ok_or("Failed to get stdout")?;
        
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line)
            .map_err(|e| format!("Failed to read from MCP server: {}", e))?;
        
        let response: JsonRpcResponse = serde_json::from_str(&line)
            .map_err(|e| format!("Invalid JSON-RPC response: {}", e))?;
        
        if let Some(error) = response.error {
            return Err(format!("MCP error: {}", error.message));
        }
        
        response.result.ok_or_else(|| "No result in response".to_string())
    }
    
    fn initialize(&mut self) -> Result<(), String> {
        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "argus",
                "version": "0.1.0"
            }
        });
        
        self.send_request("initialize", Some(params))?;
        
        // Send initialized notification
        let stdin = self.process.stdin.as_mut()
            .ok_or("Failed to get stdin")?;
        
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        
        writeln!(stdin, "{}", notification)
            .map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    fn list_tools(&mut self) -> Result<(), String> {
        let result = self.send_request("tools/list", None)?;
        
        if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
            self.tools = tools.iter()
                .filter_map(|t| {
                    let mut tool: McpTool = serde_json::from_value(t.clone()).ok()?;
                    tool.server_name = self.name.clone();
                    Some(tool)
                })
                .collect();
        }
        
        Ok(())
    }
    
    pub fn call_tool(&mut self, name: &str, arguments: serde_json::Value) -> Result<String, String> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });
        
        let result = self.send_request("tools/call", Some(params))?;
        
        // Extract text content from result
        if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
            let text: Vec<String> = content.iter()
                .filter_map(|c| {
                    if c.get("type")?.as_str()? == "text" {
                        c.get("text")?.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            return Ok(text.join("\n"));
        }
        
        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }
}

impl Drop for McpServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

pub struct McpClient {
    pub servers: Vec<McpServer>,
}

impl McpClient {
    pub fn new() -> Self {
        Self { servers: vec![] }
    }
    
    pub fn load_config() -> Result<Vec<McpServerConfig>, String> {
        let config_path = dirs::home_dir()
            .ok_or("No home directory")?
            .join(".argus")
            .join("mcp.json");
        
        if !config_path.exists() {
            return Ok(vec![]);
        }
        
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| e.to_string())?;
        
        let configs: Vec<McpServerConfig> = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid mcp.json: {}", e))?;
        
        Ok(configs)
    }
    
    pub fn connect_all(&mut self) -> Vec<String> {
        let mut errors = vec![];
        
        match Self::load_config() {
            Ok(configs) => {
                for config in configs {
                    match McpServer::connect(&config) {
                        Ok(server) => {
                            self.servers.push(server);
                        }
                        Err(e) => {
                            errors.push(format!("{}: {}", config.name, e));
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Config error: {}", e));
            }
        }
        
        errors
    }
    
    pub fn all_tools(&self) -> Vec<&McpTool> {
        self.servers.iter()
            .flat_map(|s| s.tools.iter())
            .collect()
    }
    
    pub fn call_tool(&mut self, tool_name: &str, arguments: serde_json::Value) -> Result<String, String> {
        // Find which server has this tool
        for server in &mut self.servers {
            if server.tools.iter().any(|t| t.name == tool_name) {
                return server.call_tool(tool_name, arguments);
            }
        }
        
        Err(format!("Tool '{}' not found in any MCP server", tool_name))
    }
}
