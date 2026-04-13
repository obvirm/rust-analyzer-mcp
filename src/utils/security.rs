use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub fn validate_file_path(path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path);

    let abs = if path.is_absolute() {
        path.clone()
    } else {
        std::env::current_dir()?.join(&path)
    };

    let canonical = abs
        .canonicalize()
        .map_err(|_| anyhow!("Invalid path: {}", path.display()))?;

    let allowed =
        std::env::current_dir().map_err(|_| anyhow!("Cannot determine allowed directory"))?;

    if !canonical.starts_with(&allowed) {
        return Err(anyhow!(
            "Path escapes allowed directory: {}",
            path.display()
        ));
    }

    let path_str = canonical.to_str().unwrap_or("");
    if path_str.contains("..") || (path_str.contains('~') && path_str.contains("/.ssh")) {
        return Err(anyhow!("Dangerous path pattern detected"));
    }

    Ok(canonical)
}

pub fn sanitize_command_args(args: &[String]) -> Result<Vec<String>> {
    let dangerous = [
        ';', '|', '&', '$', '`', '(', ')', '{', '}', '[', ']', '<', '>', '\'', '"', '\\',
    ];

    for arg in args {
        if arg.chars().any(|c| dangerous.contains(&c)) {
            return Err(anyhow!("Dangerous character in argument: {}", arg));
        }
    }

    Ok(args.to_vec())
}

pub fn is_blocked_path(path: &PathBuf, blocked: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    for pattern in blocked {
        if path_str.contains(pattern) {
            return true;
        }
    }
    false
}

pub fn check_file_size(path: &PathBuf, max_mb: u64) -> Result<()> {
    if let Ok(metadata) = std::fs::metadata(path) {
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        if size_mb > max_mb as f64 {
            return Err(anyhow!("File too large: {} MB > {} MB", size_mb, max_mb));
        }
    }
    Ok(())
}
