use anyhow::Result;
use std::path::PathBuf;

const VERSION_FILE: &str = "rust-analyzer-version";

pub async fn current_version() -> Result<String> {
    let version_path = get_version_file()?;

    if version_path.exists() {
        let version = std::fs::read_to_string(&version_path)?;
        Ok(version.trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

pub async fn set_version(version: &str) -> Result<()> {
    let version_path = get_version_file()?;

    if let Some(parent) = version_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&version_path, version)?;
    Ok(())
}

fn get_version_file() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Program Files"));
        Ok(base.join("rust-analyzer-mcp").join(VERSION_FILE))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let base = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/usr/local"));
        Ok(base.join(".rust-analyzer-mcp").join(VERSION_FILE))
    }
}
