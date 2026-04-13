use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub rust_analyzer: RustAnalyzerConfig,
    #[serde(default)]
    pub lsp: LspConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rust_analyzer: RustAnalyzerConfig::default(),
            lsp: LspConfig::default(),
            cache: CacheConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_from_env() -> Self {
        Config {
            rust_analyzer: RustAnalyzerConfig {
                path: std::env::var("RUST_ANALYZER_PATH").ok(),
                auto_update: std::env::var("RA_MCP_AUTO_UPDATE")
                    .map(|v| v == "true")
                    .unwrap_or(true),
                update_channel: std::env::var("RA_MCP_UPDATE_CHANNEL")
                    .unwrap_or_else(|_| "nightly".to_string()),
            },
            lsp: LspConfig {
                timeout_seconds: std::env::var("RA_MCP_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30),
                max_retries: std::env::var("RA_MCP_MAX_RETRIES")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3),
                crash_recovery: std::env::var("RA_MCP_CRASH_RECOVERY")
                    .map(|v| v == "true")
                    .unwrap_or(true),
            },
            cache: CacheConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig {
                level: std::env::var("RA_MCP_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                format: std::env::var("RA_MCP_LOG_FORMAT").unwrap_or_else(|_| "text".to_string()),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RustAnalyzerConfig {
    pub path: Option<String>,
    #[serde(default = "default_true")]
    pub auto_update: bool,
    pub update_channel: String,
}

impl Default for RustAnalyzerConfig {
    fn default() -> Self {
        Self {
            path: None,
            auto_update: true,
            update_channel: "nightly".to_string(),
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LspConfig {
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_retries")]
    pub max_retries: u32,
    #[serde(default = "default_true")]
    pub crash_recovery: bool,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 3,
            crash_recovery: true,
        }
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    pub disk_cache_dir: Option<PathBuf>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: 300,
            max_entries: 1000,
            disk_cache_dir: None,
        }
    }
}

fn default_ttl() -> u64 {
    300
}

fn default_max_entries() -> usize {
    1000
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub allowed_directories: Vec<String>,
    #[serde(default)]
    pub blocked_paths: Vec<String>,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
    #[serde(default = "default_true")]
    pub prevent_path_traversal: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_directories: vec![],
            blocked_paths: vec![],
            max_file_size_mb: 10,
            prevent_path_traversal: true,
        }
    }
}

fn default_max_file_size() -> u64 {
    10
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "text".to_string(),
        }
    }
}
