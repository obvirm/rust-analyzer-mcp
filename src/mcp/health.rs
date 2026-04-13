use crate::mcp::server::McpServer;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub server_version: String,
    pub uptime_seconds: u64,
    pub rust_analyzer: RaHealth,
    pub lsp_connection: LspHealth,
    pub cache_stats: CacheHealth,
}

#[derive(Debug, Serialize)]
pub struct RaHealth {
    pub path: String,
    pub version: Option<String>,
    pub up_to_date: bool,
    pub auto_update_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct LspHealth {
    pub connected: bool,
    pub workspace_loaded: bool,
    pub active_workspace: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CacheHealth {
    pub enabled: bool,
    pub memory_entries: usize,
}

pub async fn get_health_status(server: &McpServer) -> HealthStatus {
    let ra_version = crate::updater::version::current_version().await.ok();
    let latest = crate::updater::github::get_latest_release_tag().await.ok();

    let lsp = server.lsp_client.read().await;
    let lsp_connected = lsp.is_some();

    let up_to_date = match (&ra_version, &latest) {
        (Some(current), Some(latest)) => current == latest,
        _ => false,
    };

    HealthStatus {
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: server.start_time.elapsed().as_secs(),
        rust_analyzer: RaHealth {
            path: server.ra_path.display().to_string(),
            version: ra_version,
            up_to_date,
            auto_update_enabled: server.config.rust_analyzer.auto_update,
        },
        lsp_connection: LspHealth {
            connected: lsp_connected,
            workspace_loaded: lsp.as_ref().map(|c| c.is_ready()).unwrap_or(false),
            active_workspace: server.project_root.clone(),
        },
        cache_stats: CacheHealth {
            enabled: server.config.cache.enabled,
            memory_entries: 0,
        },
    }
}
