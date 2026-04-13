use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

pub fn get_platform() -> (&'static str, &'static str) {
    (std::env::consts::OS, std::env::consts::ARCH)
}

pub fn get_ra_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "rust-analyzer.exe"
    } else {
        "rust-analyzer"
    }
}

pub fn find_ra_in_path() -> Option<PathBuf> {
    let binary_name = get_ra_binary_name();

    if let Ok(output) = Command::new("which").arg(binary_name).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout);
            return Some(PathBuf::from(path.trim()));
        }
    }

    if cfg!(target_os = "windows") {
        if let Ok(output) = Command::new("where").arg(binary_name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                return Some(PathBuf::from(path.lines().next().unwrap_or("").trim()));
            }
        }
    }

    None
}

pub fn get_default_install_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Program Files"))
            .join("rust-analyzer-mcp")
            .join("bin")
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/usr/local"))
            .join(".rust-analyzer-mcp")
            .join("bin")
    }
}

pub async fn ensure_ra_binary(custom_path: Option<String>, auto_update: bool) -> Result<PathBuf> {
    if let Some(path) = custom_path {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
        anyhow::bail!("Custom rust-analyzer path does not exist: {}", path);
    }

    if let Some(path) = find_ra_in_path() {
        log::info!("Found rust-analyzer in PATH: {}", path.display());
        return Ok(path);
    }

    let install_dir = get_default_install_dir();
    let binary_path = install_dir.join(get_ra_binary_name());

    if binary_path.exists() {
        log::info!("Found installed rust-analyzer: {}", binary_path.display());
        return Ok(binary_path);
    }

    if auto_update {
        log::info!("No rust-analyzer found, downloading latest...");
        let latest = crate::updater::github::get_latest_release_tag().await?;
        crate::updater::binary::download_and_install(&latest).await?;

        if binary_path.exists() {
            return Ok(binary_path);
        }
    }

    anyhow::bail!(
        "rust-analyzer not found. Please either:\n\
         1. Install rust-analyzer in your PATH\n\
         2. Provide custom path with --ra-path\n\
         3. Run with --no-auto-update and install manually"
    )
}
