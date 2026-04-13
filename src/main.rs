use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;

pub mod cache;
pub mod config;
pub mod lsp;
pub mod mcp;
pub mod metrics;
pub mod updater;
pub mod utils;
pub mod workspace;

use config::Config;

#[derive(Parser, Debug)]
#[command(name = "rust-analyzer-mcp")]
#[command(about = "MCP Server wrapping rust-analyzer LSP", long_about = None)]
struct Args {
    #[arg(
        long,
        env = "RUST_ANALYZER_PATH",
        help = "Path to rust-analyzer binary"
    )]
    ra_path: Option<String>,

    #[arg(long, env = "RUST_PROJECT_ROOT", help = "Rust project root to analyze")]
    project_root: Option<String>,

    #[arg(
        short = 'c',
        long,
        default_value = "config/default.toml",
        help = "Configuration file"
    )]
    config: String,

    #[arg(long, help = "Disable auto-update checking")]
    no_auto_update: bool,

    #[arg(long, default_value = "info", help = "Log level")]
    log_level: String,

    #[arg(long, help = "Enable metrics endpoint")]
    metrics: bool,

    #[arg(long, help = "Health check mode (exit after check)")]
    health_check: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let log_level = args
        .log_level
        .parse::<log::LevelFilter>()
        .unwrap_or(log::LevelFilter::Info);
    env_logger::Builder::new()
        .filter_level(log_level)
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("Starting rust-analyzer-mcp server");

    let config = load_config(&args)?;
    log::debug!("Config loaded: {:?}", config);

    let ra_path =
        utils::platform::ensure_ra_binary(args.ra_path.clone(), !args.no_auto_update).await?;
    log::info!("Using rust-analyzer at: {}", ra_path.display());

    if args.health_check {
        return run_health_check(ra_path, &config).await;
    }

    let server = mcp::server::McpServer::new(ra_path, args.project_root, config).await?;
    Arc::new(server).run().await?;

    Ok(())
}

fn load_config(args: &Args) -> Result<Config> {
    let config_path = PathBuf::from(&args.config);

    if config_path.exists() {
        Config::load(&config_path)
    } else if config_path.to_str() == Some("config/default.toml") {
        log::warn!("Default config not found, using env vars only");
        Ok(Config::load_from_env())
    } else {
        anyhow::bail!("Config file not found: {}", args.config)
    }
}

async fn run_health_check(ra_path: PathBuf, config: &Config) -> Result<()> {
    use std::process::Command;

    let output = Command::new(&ra_path).arg("--version").output()?;

    let version = String::from_utf8_lossy(&output.stdout);

    println!("Health Check:");
    println!("  rust-analyzer: {}", ra_path.display());
    println!("  version: {}", version.trim());
    println!("  auto-update: {}", config.rust_analyzer.auto_update);
    println!("  status: OK");

    Ok(())
}
