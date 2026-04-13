use anyhow::{Context, Result};
use lsp_types::*;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

pub struct LspClient {
    #[allow(dead_code)]
    process: Child,
    writer: Mutex<tokio::process::ChildStdin>,
    reader: Mutex<BufReader<tokio::process::ChildStdout>>,
    next_id: Mutex<u32>,
    initialized: Mutex<bool>,
    workspace_root: PathBuf,
}

impl LspClient {
    pub async fn start(ra_path: &Path, workspace_root: &Path) -> Result<Self> {
        let mut process = Command::new(ra_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to start rust-analyzer")?;

        let writer = process.stdin.take().context("No stdin")?;
        let stdout = process.stdout.take().context("No stdout")?;
        let reader = BufReader::new(stdout);

        let client = Self {
            process,
            writer: Mutex::new(writer),
            reader: Mutex::new(reader),
            next_id: Mutex::new(1),
            initialized: Mutex::new(false),
            workspace_root: workspace_root.to_path_buf(),
        };

        client.initialize().await?;
        Ok(client)
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn is_ready(&self) -> bool {
        // Simple check - in real impl use tokio::block_on or runtime
        false
    }

    async fn next_id(&self) -> u32 {
        let mut id = self.next_id.lock().await;
        let current = *id;
        *id += 1;
        current
    }

    pub async fn send_request<R: serde::Serialize>(
        &self,
        method: &str,
        params: R,
    ) -> Result<serde_json::Value> {
        let id = self.next_id().await;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let request_str = serde_json::to_string(&request)?;
        let header = format!("Content-Length: {}\r\n\r\n", request_str.len());

        {
            let mut writer = self.writer.lock().await;
            writer.write_all(header.as_bytes()).await?;
            writer.write_all(request_str.as_bytes()).await?;
            writer.flush().await?;
        }

        let response = self.read_response(id).await?;
        Ok(response)
    }

    pub async fn send_notification<N: serde::Serialize>(
        &self,
        method: &str,
        params: N,
    ) -> Result<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let notification_str = serde_json::to_string(&notification)?;
        let header = format!("Content-Length: {}\r\n\r\n", notification_str.len());

        let mut writer = self.writer.lock().await;
        writer.write_all(header.as_bytes()).await?;
        writer.write_all(notification_str.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }

    async fn read_response(&self, expected_id: u32) -> Result<serde_json::Value> {
        let mut reader = self.reader.lock().await;

        loop {
            let mut header_line = String::new();
            reader.read_line(&mut header_line).await?;

            if !header_line.starts_with("Content-Length:") {
                continue;
            }

            let len: usize = header_line
                .trim()
                .strip_prefix("Content-Length:")
                .unwrap()
                .trim()
                .parse()?;

            let mut empty = String::new();
            reader.read_line(&mut empty).await?;

            let mut body = vec![0u8; len];
            reader.read_exact(&mut body).await?;

            let msg: serde_json::Value = serde_json::from_slice(&body)?;

            if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
                if id == expected_id as u64 {
                    if let Some(error) = msg.get("error") {
                        anyhow::bail!("LSP error: {}", error);
                    }
                    return Ok(msg["result"].clone());
                }
            }
        }
    }

    async fn initialize(&self) -> Result<()> {
        let root_uri = url::Url::from_file_path(&self.workspace_root)
            .map_err(|_| anyhow::anyhow!("Invalid workspace path"))?;

        let init_params = InitializeParams {
            root_uri: Some(root_uri),
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    completion: Some(CompletionClientCapabilities {
                        completion_item: Some(CompletionItemCapability {
                            snippet_support: Some(false),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    hover: Some(HoverClientCapabilities {
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        self.send_request("initialize", init_params).await?;
        self.send_notification("initialized", InitializedParams {})
            .await?;

        *self.initialized.lock().await = true;
        Ok(())
    }

    // High-level LSP methods

    pub async fn goto_definition(
        &self,
        file_path: &Path,
        line: u32,
        column: u32,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url::Url::from_file_path(file_path).unwrap(),
                },
                position: Position::new(line, column),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let result = self.send_request("textDocument/definition", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn hover(&self, file_path: &Path, line: u32, column: u32) -> Result<Option<Hover>> {
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url::Url::from_file_path(file_path).unwrap(),
                },
                position: Position::new(line, column),
            },
            work_done_progress_params: Default::default(),
        };
        let result = self.send_request("textDocument/hover", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn references(
        &self,
        file_path: &Path,
        line: u32,
        column: u32,
        include_declaration: bool,
    ) -> Result<Vec<Location>> {
        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url::Url::from_file_path(file_path).unwrap(),
                },
                position: Position::new(line, column),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: ReferenceContext {
                include_declaration,
            },
        };
        let result = self.send_request("textDocument/references", params).await?;
        let refs: Option<Vec<Location>> = serde_json::from_value(result)?;
        Ok(refs.unwrap_or_default())
    }

    pub async fn completions(
        &self,
        file_path: &Path,
        line: u32,
        column: u32,
        trigger_character: Option<String>,
    ) -> Result<Option<CompletionResponse>> {
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url::Url::from_file_path(file_path).unwrap(),
                },
                position: Position::new(line, column),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: trigger_character.map(|ch| CompletionContext {
                trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
                trigger_character: Some(ch),
            }),
        };
        let result = self.send_request("textDocument/completion", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn diagnostics(&self, file_path: &Path) -> Result<Vec<Diagnostic>> {
        let uri = url::Url::from_file_path(file_path).unwrap();
        let params = DocumentDiagnosticParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            identifier: None,
            previous_result_id: None,
        };
        let result = self.send_request("textDocument/diagnostic", params).await?;
        let report: DocumentDiagnosticReport = serde_json::from_value(result)?;
        match report {
            DocumentDiagnosticReport::Full(full) => Ok(full.full_document_diagnostic_report.items),
            DocumentDiagnosticReport::Unchanged(_) => Ok(vec![]),
        }
    }

    pub async fn rename(
        &self,
        file_path: &Path,
        line: u32,
        column: u32,
        new_name: &str,
    ) -> Result<Option<WorkspaceEdit>> {
        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: url::Url::from_file_path(file_path).unwrap(),
                },
                position: Position::new(line, column),
            },
            new_name: new_name.to_string(),
            work_done_progress_params: Default::default(),
        };
        let result = self.send_request("textDocument/rename", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn code_actions(
        &self,
        file_path: &Path,
        line: u32,
        column: u32,
        kind: Option<String>,
    ) -> Result<Vec<CodeAction>> {
        let uri = url::Url::from_file_path(file_path).unwrap();
        let params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range::new(Position::new(line, column), Position::new(line, column)),
            context: CodeActionContext {
                diagnostics: vec![],
                only: kind.map(|k| vec![CodeActionKind::from(k)]),
                trigger_kind: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let result = self.send_request("textDocument/codeAction", params).await?;
        let response: Option<CodeActionResponse> = serde_json::from_value(result)?;
        Ok(response
            .map(|actions| {
                actions
                    .into_iter()
                    .filter_map(|a| match a {
                        CodeActionOrCommand::CodeAction(ca) => Some(ca),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    pub async fn workspace_symbol(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SymbolInformation>> {
        let params = WorkspaceSymbolParams {
            query: query.to_string(),
            ..Default::default()
        };
        let result = self.send_request("workspace/symbol", params).await?;
        let symbols: Option<Vec<SymbolInformation>> = serde_json::from_value(result)?;
        Ok(symbols
            .unwrap_or_default()
            .into_iter()
            .take(limit)
            .collect())
    }

    pub async fn document_symbol(&self, file_path: &Path) -> Result<Vec<DocumentSymbol>> {
        let uri = url::Url::from_file_path(file_path).unwrap();
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let result = self
            .send_request("textDocument/documentSymbol", params)
            .await?;
        let symbols: Option<Vec<DocumentSymbol>> = serde_json::from_value(result)?;
        Ok(symbols.unwrap_or_default())
    }

    pub async fn inlay_hints(&self, file_path: &Path) -> Result<Vec<InlayHint>> {
        let uri = url::Url::from_file_path(file_path).unwrap();
        let params = InlayHintParams {
            text_document: TextDocumentIdentifier { uri },
            range: Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX)),
            work_done_progress_params: Default::default(),
        };
        let result = self.send_request("textDocument/inlayHint", params).await?;
        let hints: Option<Vec<InlayHint>> = serde_json::from_value(result)?;
        Ok(hints.unwrap_or_default())
    }

    // rust-analyzer specific extensions

    pub async fn expand_macro(
        &self,
        file_path: &Path,
        line: u32,
        column: u32,
    ) -> Result<Option<ExpandedMacro>> {
        let params = serde_json::json!({
            "textDocument": { "uri": url::Url::from_file_path(file_path).unwrap() },
            "position": { "line": line, "character": column }
        });
        let result = self
            .send_request("rust-analyzer/expandMacro", params)
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn runnables(&self, file_path: Option<&Path>) -> Result<Vec<Runnable>> {
        let params = serde_json::json!({
            "textDocument": file_path.map(|p| serde_json::json!({ "uri": url::Url::from_file_path(p).unwrap() }))
        });
        let result = self.send_request("rust-analyzer/runnables", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn view_hir(&self, file_path: &Path, line: u32, column: u32) -> Result<String> {
        let params = serde_json::json!({
            "textDocument": { "uri": url::Url::from_file_path(file_path).unwrap() },
            "position": { "line": line, "character": column }
        });
        let result = self.send_request("rust-analyzer/viewHir", params).await?;
        Ok(result.as_str().unwrap_or_default().to_string())
    }

    pub async fn status(&self) -> Result<String> {
        let result = self
            .send_request("rust-analyzer/status", serde_json::json!(null))
            .await?;
        Ok(result.as_str().unwrap_or_default().to_string())
    }

    pub async fn goto_type_definition(
        &self,
        file_path: &Path,
        line: u32,
        column: u32,
    ) -> Result<Option<Location>> {
        let params = serde_json::json!({
            "textDocument": { "uri": url::Url::from_file_path(file_path).unwrap() },
            "position": { "line": line, "character": column }
        });
        let result = self
            .send_request("textDocument/typeDefinition", params)
            .await?;
        let response: Option<Location> = serde_json::from_value(result)?;
        Ok(response)
    }

    pub async fn goto_implementation(
        &self,
        file_path: &Path,
        line: u32,
        column: u32,
    ) -> Result<Vec<Location>> {
        let params = serde_json::json!({
            "textDocument": { "uri": url::Url::from_file_path(file_path).unwrap() },
            "position": { "line": line, "character": column }
        });
        let result = self
            .send_request("textDocument/implementation", params)
            .await?;
        let locations: Option<Vec<Location>> = serde_json::from_value(result)?;
        Ok(locations.unwrap_or_default())
    }
}

// Custom types for rust-analyzer extensions

#[derive(Debug, serde::Deserialize)]
pub struct ExpandedMacro {
    pub name: String,
    pub expansion: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Runnable {
    pub label: String,
    pub kind: serde_json::Value,
    pub args: serde_json::Value,
}
