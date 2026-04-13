use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::lsp::client::LspClient;

pub struct WorkspaceManager {
    ra_path: PathBuf,
    workspaces: Arc<RwLock<HashMap<PathBuf, WorkspaceInfo>>>,
    active_workspace: Arc<RwLock<Option<PathBuf>>>,
}

#[derive(Clone)]
pub struct WorkspaceInfo {
    pub root: PathBuf,
    pub lsp_client: Arc<LspClient>,
    pub crate_names: Vec<String>,
}

impl WorkspaceManager {
    pub fn new(ra_path: PathBuf) -> Self {
        Self {
            ra_path,
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            active_workspace: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn add_workspace(&self, root: PathBuf) -> Result<WorkspaceInfo> {
        let cargo_toml = root.join("Cargo.toml");
        if !cargo_toml.exists() {
            anyhow::bail!("Not a Cargo project: {}", root.display());
        }

        let crate_names = self.parse_crate_names(&cargo_toml).await?;
        let lsp_client = LspClient::start(&self.ra_path, &root).await?;

        let info = WorkspaceInfo {
            root: root.clone(),
            lsp_client: Arc::new(lsp_client),
            crate_names,
        };

        self.workspaces
            .write()
            .await
            .insert(root.clone(), info.clone());

        if self.active_workspace.read().await.is_none() {
            *self.active_workspace.write().await = Some(root);
        }

        Ok(info)
    }

    pub async fn switch_workspace(&self, root: &Path) -> Result<()> {
        let workspaces = self.workspaces.read().await;
        if !workspaces.contains_key(root) {
            anyhow::bail!("Workspace not found: {}", root.display());
        }
        *self.active_workspace.write().await = Some(root.to_path_buf());
        Ok(())
    }

    pub async fn get_active(&self) -> Result<Arc<LspClient>> {
        let active = self.active_workspace.read().await;
        let root = active
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active workspace"))?;
        let workspaces = self.workspaces.read().await;
        let info = workspaces
            .get(root)
            .ok_or_else(|| anyhow::anyhow!("Workspace not found"))?;
        Ok(info.lsp_client.clone())
    }

    pub async fn list_workspaces(&self) -> Vec<(PathBuf, Vec<String>)> {
        self.workspaces
            .read()
            .await
            .iter()
            .map(|(root, info)| (root.clone(), info.crate_names.clone()))
            .collect()
    }

    async fn parse_crate_names(&self, cargo_toml: &Path) -> Result<Vec<String>> {
        let content = tokio::fs::read_to_string(cargo_toml).await?;
        let doc: toml::Value = content.parse()?;

        let mut names = Vec::new();

        if let Some(name) = doc
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
        {
            names.push(name.to_string());
        }

        if let Some(workspace) = doc.get("workspace") {
            if let Some(members) = workspace.get("members") {
                if let Some(arr) = members.as_array() {
                    for member in arr {
                        if let Some(path) = member.as_str() {
                            names.push(path.to_string());
                        }
                    }
                }
            }
        }

        Ok(names)
    }
}
