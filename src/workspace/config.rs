use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub root: PathBuf,
    pub crate_name: Option<String>,
    pub edition: Option<String>,
    pub rust_version: Option<String>,
    pub dependencies: Vec<String>,
}

impl ProjectConfig {
    pub fn from_cargo_toml(path: &Path) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let doc: toml::Value = content.parse()?;

        let crate_name = doc
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(String::from);

        let edition = doc
            .get("package")
            .and_then(|p| p.get("edition"))
            .and_then(|e| e.as_str())
            .map(String::from);

        let rust_version = doc
            .get("package")
            .and_then(|p| p.get("rust-version"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let mut dependencies = Vec::new();
        if let Some(deps) = doc.get("dependencies").and_then(|d| d.as_table()) {
            for (name, _) in deps {
                dependencies.push(name.clone());
            }
        }

        Ok(Self {
            root: path.parent().map(|p| p.to_path_buf()).unwrap_or_default(),
            crate_name,
            edition,
            rust_version,
            dependencies,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub members: Vec<PathBuf>,
    pub root: PathBuf,
}

impl WorkspaceConfig {
    pub fn from_cargo_toml(path: &Path) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let doc: toml::Value = content.parse()?;

        let root = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();

        let members: Vec<PathBuf> = doc
            .get("workspace")
            .and_then(|w| w.get("members"))
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| root.join(s))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self { members, root })
    }
}
