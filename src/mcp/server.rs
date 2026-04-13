use anyhow::{Context, Result};
use serde_json::json;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;

use crate::cache::Cache;
use crate::config::Config;
use crate::lsp::client::LspClient;
use crate::mcp::health;
use crate::mcp::resources;
use crate::mcp::tools;
use crate::metrics::Metrics;
use crate::updater;
use crate::workspace::manager::WorkspaceManager;

pub struct McpServer {
    pub ra_path: PathBuf,
    pub project_root: Option<String>,
    pub config: Config,
    pub lsp_client: Arc<RwLock<Option<Arc<LspClient>>>>,
    pub workspace_manager: Arc<WorkspaceManager>,
    #[allow(dead_code)]
    cache: Arc<Cache<String, serde_json::Value>>,
    pub metrics: Arc<Metrics>,
    pub start_time: Instant,
}

impl McpServer {
    pub async fn new(
        ra_path: PathBuf,
        project_root: Option<String>,
        config: Config,
    ) -> Result<Self> {
        let cache = Arc::new(Cache::new(
            std::time::Duration::from_secs(config.cache.ttl_seconds),
            config.cache.max_entries,
        ));

        let metrics = Arc::new(Metrics::new());

        let workspace_manager = Arc::new(WorkspaceManager::new(ra_path.clone()));

        let server = Self {
            ra_path,
            project_root,
            config,
            lsp_client: Arc::new(RwLock::new(None)),
            workspace_manager,
            cache,
            metrics,
            start_time: Instant::now(),
        };

        if let Some(root) = &server.project_root {
            let root_path = PathBuf::from(root);
            if root_path.exists() {
                server.open_workspace(&root_path).await?;
            }
        }

        Ok(server)
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let server = self.clone();

        tokio::spawn(async move {
            if let Err(e) = server.read_stdin_loop().await {
                log::error!("Server error: {}", e);
            }
        });

        tokio::signal::ctrl_c().await?;

        log::info!("Shutting down MCP server");
        Ok(())
    }

    async fn read_stdin_loop(&self) -> Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let mut stdin = BufReader::new(tokio::io::stdin());
        let mut stdout = tokio::io::stdout();

        loop {
            let mut header_line = String::new();
            match stdin.read_line(&mut header_line).await {
                Ok(0) => break,
                Ok(_) => {},
                Err(e) => return Err(anyhow::anyhow!("Failed to read header: {}", e)),
            }

            if header_line.is_empty() {
                continue;
            }

            if !header_line.starts_with("Content-Length:") {
                continue;
            }

            let len: usize = header_line
                .trim()
                .strip_prefix("Content-Length:")
                .unwrap_or("")
                .trim()
                .parse()
                .context("Invalid content length")?;

            let mut empty_line = String::new();
            stdin.read_line(&mut empty_line).await?;

            let mut body = vec![0u8; len];
            stdin.read_exact(&mut body).await?;

            let message: Value = serde_json::from_slice(&body).context("Failed to parse JSON")?;

            let start = Instant::now();
            let response = self.handle_message(message).await?;
            let duration = start.elapsed();

            let method = response
                .get("method")
                .and_then(|m| m.as_str())
                .unwrap_or("response")
                .to_string();

            self.metrics.record_request(&method, duration).await;

            let response_str = serde_json::to_string(&response)?;
            let response_header = format!("Content-Length: {}\r\n\r\n", response_str.len());

            stdout.write_all(response_header.as_bytes()).await?;
            stdout.write_all(response_str.as_bytes()).await?;
            stdout.flush().await?;
        }

        Ok(())
    }

    async fn handle_message(&self, message: Value) -> Result<Value> {
        let method = message.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = message.get("id").cloned();

        match method {
            "initialize" => self.handle_initialize(id).await,
            "initialized" => Ok(json!({"jsonrpc": "2.0", "id": id})),
            "tools/list" => self.handle_tools_list(id).await,
            "tools/call" => {
                let name = message
                    .get("params")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let args = message
                    .get("params")
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or(json!({}));
                self.handle_tool_call(id, name, &args).await
            },
            "resources/list" => self.handle_resources_list(id).await,
            "resources/read" => {
                let uri = message
                    .get("params")
                    .and_then(|p| p.get("uri"))
                    .and_then(|u| u.as_str())
                    .unwrap_or("");
                self.handle_resource_read(id, uri).await
            },
            "prompts/list" => self.handle_prompts_list(id).await,
            "prompts/get" => {
                let name = message
                    .get("params")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                self.handle_prompt_get(id, name).await
            },
            "shutdown" => Ok(json!({"jsonrpc": "2.0", "id": id, "result": null})),
            _ => {
                if let Some(id) = id {
                    Ok(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": "Method not found" }
                    }))
                } else {
                    Ok(json!({}))
                }
            },
        }
    }

    async fn handle_initialize(&self, id: Option<Value>) -> Result<Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": true },
                    "resources": { "subscribe": true, "listChanged": true },
                    "prompts": { "listChanged": true }
                },
                "serverInfo": {
                    "name": "rust-analyzer-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        }))
    }

    async fn handle_tools_list(&self, id: Option<Value>) -> Result<Value> {
        let tools_list: Vec<Value> = tools::get_tools()
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            })
            .collect();

        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "tools": tools_list }
        }))
    }

    async fn handle_tool_call(&self, id: Option<Value>, name: &str, args: &Value) -> Result<Value> {
        let start = Instant::now();

        let result = match name {
            "open_project" => self.tool_open_project(args).await?,
            "status" => self.tool_status().await?,
            "goto_definition" => self.tool_goto_definition(args).await?,
            "find_references" => self.tool_find_references(args).await?,
            "hover" => self.tool_hover(args).await?,
            "completions" => self.tool_completions(args).await?,
            "get_diagnostics" => self.tool_diagnostics(args).await?,
            "code_action" => self.tool_code_action(args).await?,
            "rename_symbol" => self.tool_rename(args).await?,
            "workspace_symbol" => self.tool_workspace_symbol(args).await?,
            "file_structure" => self.tool_file_structure(args).await?,
            "format_file" => self.tool_format(args).await?,
            "inlay_hints" => self.tool_inlay_hints(args).await?,
            "expand_macro" => self.tool_expand_macro(args).await?,
            "runnables" => self.tool_runnables(args).await?,
            "view_hir" => self.tool_view_hir(args).await?,
            "check_update" => self.tool_check_update().await?,
            "update_rust_analyzer" => self.tool_update(args).await?,
            "health_check" => self.tool_health_check(args).await?,
            "switch_workspace" => self.tool_switch_workspace(args).await?,
            "list_workspaces" => self.tool_list_workspaces().await?,
            "goto_type_definition" => self.tool_goto_type_definition(args).await?,
            "goto_implementation" => self.tool_goto_implementation(args).await?,
            _ => json!({
                "content": [{ "type": "text", "text": format!("Unknown tool: {}", name) }],
                "isError": true
            }),
        };

        let duration = start.elapsed();
        self.metrics.record_request(name, duration).await;

        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }))
    }

    async fn handle_resources_list(&self, id: Option<Value>) -> Result<Value> {
        let resources_list: Vec<Value> = resources::get_resources()
            .iter()
            .map(|r| {
                json!({
                    "uri": r.uri,
                    "name": r.name,
                    "description": r.description,
                    "mimeType": r.mime_type
                })
            })
            .collect();

        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "resources": resources_list }
        }))
    }

    async fn handle_resource_read(&self, id: Option<Value>, uri: &str) -> Result<Value> {
        let content = match uri {
            "health://status" => {
                let status = health::get_health_status(self).await;
                serde_json::to_string(&status)?
            },
            "metrics://server" => {
                let summary = self.metrics.get_summary().await;
                serde_json::to_string(&summary)?
            },
            "config://current" => serde_json::to_string(&self.config)?,
            "version://rust-analyzer" => updater::version::current_version()
                .await
                .unwrap_or_else(|_| "unknown".to_string()),
            _ => {
                return Ok(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32602, "message": "Resource not found" }
                }))
            },
        };

        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": content
                }]
            }
        }))
    }

    async fn handle_prompts_list(&self, id: Option<Value>) -> Result<Value> {
        let prompts_list: Vec<Value> = crate::mcp::prompts::get_prompts()
            .iter()
            .map(|p| {
                json!({
                    "name": p.name,
                    "description": p.description,
                    "arguments": p.arguments.iter().map(|a| {
                        json!({
                            "name": a.name,
                            "description": a.description,
                            "required": a.required
                        })
                    }).collect::<Vec<_>>()
                })
            })
            .collect();

        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "prompts": prompts_list }
        }))
    }

    async fn handle_prompt_get(&self, id: Option<Value>, name: &str) -> Result<Value> {
        match name {
            "analyze_error" => Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "messages": [{
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": "Analyze this compiler error and suggest fixes:\n\nError: {error_message}\n\nFile: {file_path}"
                        }
                    }]
                }
            })),
            _ => Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": "Prompt not found" }
            })),
        }
    }

    // Tool implementations
    async fn tool_open_project(&self, args: &Value) -> Result<Value> {
        let root = args
            .get("workspace_root")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        let path = PathBuf::from(root);

        self.open_workspace(&path).await?;

        Ok(json!({
            "content": [{ "type": "text", "text": format!("Opened project at: {}", root) }]
        }))
    }

    async fn tool_status(&self) -> Result<Value> {
        let client = self.lsp_client.read().await;

        let ra_version = updater::version::current_version()
            .await
            .unwrap_or_else(|_| "unknown".to_string());

        match client.as_ref() {
            Some(c) => {
                let status = c.status().await.unwrap_or_else(|e| e.to_string());
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!(
                            "rust-analyzer MCP Server v{}\n\nLSP Status:\n{}\nBinary version: {}",
                            env!("CARGO_PKG_VERSION"),
                            status,
                            ra_version
                        )
                    }]
                }))
            },
            None => Ok(json!({
                "content": [{ "type": "text", "text": "No project opened. Use 'open_project' first." }]
            })),
        }
    }

    async fn tool_goto_definition(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;
        let line = (args.get("line").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let column = (args.get("column").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;

        let result = client.goto_definition(&file_path, line, column).await?;

        match result {
            Some(lsp_types::GotoDefinitionResponse::Scalar(loc)) => {
                let text = format_location(&loc);
                Ok(json!({ "content": [{ "type": "text", "text": text }] }))
            },
            Some(lsp_types::GotoDefinitionResponse::Array(locs)) => {
                let text = locs
                    .iter()
                    .map(format_location)
                    .collect::<Vec<_>>()
                    .join("\n\n");
                Ok(json!({ "content": [{ "type": "text", "text": text }] }))
            },
            Some(lsp_types::GotoDefinitionResponse::Link(_)) => {
                Ok(json!({ "content": [{ "type": "text", "text": "Definition found (link)" }] }))
            },
            None => Ok(json!({ "content": [{ "type": "text", "text": "No definition found" }] })),
        }
    }

    async fn tool_find_references(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;
        let line = (args.get("line").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let column = (args.get("column").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let include_declaration = args
            .get("include_declaration")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let refs = client
            .references(&file_path, line, column, include_declaration)
            .await?;

        let locations: Vec<String> = refs
            .iter()
            .filter_map(|loc| {
                let uri = loc.uri.to_string();
                let pos = format!(
                    "{}:{}:{}",
                    uri,
                    loc.range.start.line + 1,
                    loc.range.start.character + 1
                );
                Some(pos)
            })
            .collect();

        Ok(json!({
            "content": [{ "type": "text", "text": locations.join("\n") }]
        }))
    }

    async fn tool_hover(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;
        let line = (args.get("line").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let column = (args.get("column").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;

        let result = client.hover(&file_path, line, column).await?;

        match result {
            Some(hover) => {
                let text = match &hover.contents {
                    lsp_types::HoverContents::Markup(m) => m.value.clone(),
                    _ => format!("{:?}", hover.contents),
                };

                Ok(json!({ "content": [{ "type": "text", "text": text }] }))
            },
            None => Ok(
                json!({ "content": [{ "type": "text", "text": "No hover information available" }] }),
            ),
        }
    }

    async fn tool_completions(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;
        let line = (args.get("line").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let column = (args.get("column").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let trigger = args
            .get("trigger_character")
            .and_then(|v| v.as_str())
            .map(String::from);

        let result = client
            .completions(&file_path, line, column, trigger)
            .await?;

        let items: Vec<String> = match result {
            Some(lsp_types::CompletionResponse::Array(items)) => {
                items.iter().map(|item| item.label.clone()).collect()
            },
            Some(lsp_types::CompletionResponse::List(list)) => {
                list.items.iter().map(|item| item.label.clone()).collect()
            },
            None => vec![],
        };

        Ok(json!({
            "content": [{ "type": "text", "text": items.join("\n") }]
        }))
    }

    async fn tool_diagnostics(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        if let Some(file_path_str) = args.get("file_path").and_then(|v| v.as_str()) {
            let file_path = self.resolve_file(file_path_str)?;
            let diagnostics = client.diagnostics(&file_path).await?;
            let text = format_diagnostics(&diagnostics);
            Ok(json!({ "content": [{ "type": "text", "text": text }] }))
        } else {
            Ok(json!({
                "content": [{ "type": "text", "text": "Use per-file diagnostics for now." }]
            }))
        }
    }

    async fn tool_code_action(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;
        let line = (args.get("line").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let column = (args.get("column").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let kind = args.get("kind").and_then(|v| v.as_str()).map(String::from);

        let actions = client.code_actions(&file_path, line, column, kind).await?;

        let titles: Vec<String> = actions.iter().map(|a| a.title.clone()).collect();

        Ok(json!({
            "content": [{ "type": "text", "text": titles.join("\n") }]
        }))
    }

    async fn tool_rename(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;
        let line = (args.get("line").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let column = (args.get("column").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let new_name = args.get("new_name").and_then(|v| v.as_str()).unwrap_or("");

        let _result = client.rename(&file_path, line, column, new_name).await?;

        Ok(json!({
            "content": [{ "type": "text", "text": format!("Rename to {} completed", new_name) }]
        }))
    }

    async fn tool_workspace_symbol(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

        let symbols = client.workspace_symbol(query, limit).await?;

        let results: Vec<String> = symbols
            .iter()
            .map(|s| {
                format!(
                    "{}:{} - {}",
                    s.location.uri,
                    s.location.range.start.line + 1,
                    s.name
                )
            })
            .collect();

        Ok(json!({
            "content": [{ "type": "text", "text": results.join("\n") }]
        }))
    }

    async fn tool_file_structure(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;

        let symbols = client.document_symbol(&file_path).await?;

        let results: Vec<String> = symbols
            .iter()
            .map(|s| {
                let range = s.selection_range;
                format!(
                    "{}:{} - {} ({:?})",
                    range.start.line + 1,
                    range.start.character + 1,
                    s.name,
                    s.kind
                )
            })
            .collect();

        Ok(json!({
            "content": [{ "type": "text", "text": results.join("\n") }]
        }))
    }

    async fn tool_format(&self, args: &Value) -> Result<Value> {
        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;

        let output = std::process::Command::new("rustfmt")
            .arg(&file_path)
            .output()?;

        if output.status.success() {
            Ok(json!({
                "content": [{ "type": "text", "text": "File formatted successfully" }]
            }))
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Ok(json!({
                "content": [{ "type": "text", "text": format!("Format failed: {}", err) }],
                "isError": true
            }))
        }
    }

    async fn tool_inlay_hints(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;

        let hints = client.inlay_hints(&file_path).await?;

        let results: Vec<String> = hints
            .iter()
            .map(|h| {
                let label = match &h.label {
                    lsp_types::InlayHintLabel::String(s) => s.clone(),
                    lsp_types::InlayHintLabel::LabelParts(parts) => parts
                        .iter()
                        .map(|p| p.value.clone())
                        .collect::<Vec<_>>()
                        .join(""),
                };
                format!(
                    "{}:{}: {}",
                    h.position.line + 1,
                    h.position.character + 1,
                    label
                )
            })
            .collect();

        Ok(json!({
            "content": [{ "type": "text", "text": results.join("\n") }]
        }))
    }

    async fn tool_expand_macro(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;
        let line = (args.get("line").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let column = (args.get("column").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;

        let result = client.expand_macro(&file_path, line, column).await?;

        match result {
            Some(m) => Ok(json!({
                "content": [{ "type": "text", "text": format!("{}\n\n{}", m.name, m.expansion) }]
            })),
            None => Ok(json!({ "content": [{ "type": "text", "text": "No macro at position" }] })),
        }
    }

    async fn tool_runnables(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .map(|p| self.resolve_file(p).ok())
            .flatten();

        let runnables = client.runnables(file_path.as_deref()).await?;

        let results: Vec<String> = runnables
            .iter()
            .map(|r| format!("{}: {:?}", r.label, r.kind))
            .collect();

        Ok(json!({
            "content": [{ "type": "text", "text": results.join("\n") }]
        }))
    }

    async fn tool_view_hir(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;
        let line = (args.get("line").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let column = (args.get("column").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;

        let hir = client.view_hir(&file_path, line, column).await?;

        Ok(json!({
            "content": [{ "type": "text", "text": hir }]
        }))
    }

    async fn tool_check_update(&self) -> Result<Value> {
        let latest = updater::github::get_latest_release_tag().await?;
        let current = updater::version::current_version()
            .await
            .unwrap_or_default();
        let update_available = latest != current;

        let text = if update_available {
            format!(
                "Update available! Current: {} → Latest: {}",
                current, latest
            )
        } else {
            format!("Up to date: {}", current)
        };

        Ok(json!({
            "content": [{ "type": "text", "text": text }]
        }))
    }

    async fn tool_update(&self, args: &Value) -> Result<Value> {
        let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);

        let current = updater::version::current_version()
            .await
            .unwrap_or_default();
        let latest = updater::github::get_latest_release_tag().await?;

        if !force && latest == current {
            return Ok(json!({
                "content": [{ "type": "text", "text": format!("Already on latest version: {}", current) }]
            }));
        }

        updater::binary::download_and_install(&latest).await?;

        if self.lsp_client.read().await.is_some() {
            if let Some(root) = &self.project_root {
                self.open_workspace(&PathBuf::from(root)).await?;
            }
        }

        Ok(json!({
            "content": [{ "type": "text", "text": format!("Updated rust-analyzer: {} → {}", current, latest) }]
        }))
    }

    async fn tool_health_check(&self, args: &Value) -> Result<Value> {
        let include_details = args
            .get("include_details")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let status = health::get_health_status(self).await;

        let text = if include_details {
            serde_json::to_string_pretty(&status)?
        } else {
            format!(
                "Server: v{}\nUptime: {}s\nLSP Connected: {}\nrust-analyzer: {}",
                status.server_version,
                status.uptime_seconds,
                status.lsp_connection.connected,
                status.rust_analyzer.version.unwrap_or_default()
            )
        };

        Ok(json!({
            "content": [{ "type": "text", "text": text }]
        }))
    }

    async fn tool_switch_workspace(&self, args: &Value) -> Result<Value> {
        let root = args
            .get("workspace_root")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let path = PathBuf::from(root);

        self.workspace_manager.switch_workspace(&path).await?;

        Ok(json!({
            "content": [{ "type": "text", "text": format!("Switched to workspace: {}", root) }]
        }))
    }

    async fn tool_list_workspaces(&self) -> Result<Value> {
        let workspaces = self.workspace_manager.list_workspaces().await;

        let text = workspaces
            .iter()
            .map(|(root, crates)| format!("{}: {:?}", root.display(), crates))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(json!({
            "content": [{ "type": "text", "text": text }]
        }))
    }

    async fn tool_goto_type_definition(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;
        let line = (args.get("line").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let column = (args.get("column").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;

        let result = client
            .goto_type_definition(&file_path, line, column)
            .await?;

        match result {
            Some(loc) => Ok(json!({
                "content": [{ "type": "text", "text": format_location(&loc) }]
            })),
            None => {
                Ok(json!({ "content": [{ "type": "text", "text": "No type definition found" }] }))
            },
        }
    }

    async fn tool_goto_implementation(&self, args: &Value) -> Result<Value> {
        let client = self.require_client().await?;

        let file_path =
            self.resolve_file(args.get("file_path").and_then(|v| v.as_str()).unwrap_or(""))?;
        let line = (args.get("line").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;
        let column = (args.get("column").and_then(|v| v.as_u64()).unwrap_or(1) - 1) as u32;

        let result = client.goto_implementation(&file_path, line, column).await?;

        let locations: Vec<String> = result.iter().map(|loc| format_location(loc)).collect();

        Ok(json!({
            "content": [{ "type": "text", "text": locations.join("\n\n") }]
        }))
    }

    async fn require_client(&self) -> Result<Arc<LspClient>> {
        let guard = self.lsp_client.read().await;
        guard
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No project opened. Use 'open_project' first."))
    }

    async fn open_workspace(&self, root: &Path) -> Result<()> {
        let client = LspClient::start(&self.ra_path, root).await?;
        *self.lsp_client.write().await = Some(Arc::new(client));
        self.workspace_manager
            .add_workspace(root.to_path_buf())
            .await?;
        Ok(())
    }

    fn resolve_file(&self, path: &str) -> Result<PathBuf> {
        let p = PathBuf::from(path);
        if p.is_absolute() {
            Ok(p)
        } else if let Some(root) = &self.project_root {
            Ok(PathBuf::from(root).join(p))
        } else {
            Ok(std::env::current_dir()?.join(p))
        }
    }
}

fn format_location(loc: &lsp_types::Location) -> String {
    format!(
        "{}:{}:{}",
        loc.uri,
        loc.range.start.line + 1,
        loc.range.start.character + 1
    )
}

fn format_diagnostics(diagnostics: &[lsp_types::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|d| {
            let severity = match d.severity {
                Some(lsp_types::DiagnosticSeverity::ERROR) => "error",
                Some(lsp_types::DiagnosticSeverity::WARNING) => "warning",
                Some(lsp_types::DiagnosticSeverity::INFORMATION) => "info",
                Some(lsp_types::DiagnosticSeverity::HINT) => "hint",
                _ => "unknown",
            };
            format!(
                "{}:{}: [{}] {}",
                d.range.start.line + 1,
                d.range.start.character + 1,
                severity,
                d.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
