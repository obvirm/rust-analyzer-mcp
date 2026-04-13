pub mod cache;
pub mod config;
pub mod lsp;
pub mod mcp;
pub mod metrics;
pub mod updater;
pub mod utils;
pub mod workspace;

#[cfg(test)]
mod tests {
    use crate::cache::Cache;
    use crate::config::{Config, LspConfig, SecurityConfig};
    use crate::mcp::tools::get_tools;
    use crate::mcp::resources::get_resources;
    use crate::mcp::prompts::get_prompts;
    use crate::utils::platform::{get_platform, get_ra_binary_name};

    #[test]
    fn test_tools_not_empty() {
        let tools = get_tools();
        assert!(!tools.is_empty());
    }

    #[test]
    fn test_tools_have_open_project() {
        let tools = get_tools();
        let has_open_project = tools.iter().any(|t| t.name == "open_project");
        assert!(has_open_project);
    }

    #[test]
    fn test_resources_not_empty() {
        let resources = get_resources();
        assert!(!resources.is_empty());
    }

    #[test]
    fn test_prompts_not_empty() {
        let prompts = get_prompts();
        assert!(!prompts.is_empty());
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.cache.ttl_seconds, 300);
        assert_eq!(config.cache.max_entries, 1000);
    }

    #[test]
    fn test_lsp_config_default() {
        let config = LspConfig::default();
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert_eq!(config.max_file_size_mb, 10);
    }

    #[test]
    fn test_platform_info() {
        let (os, arch) = get_platform();
        assert!(!os.is_empty());
        assert!(!arch.is_empty());
    }

    #[test]
    fn test_ra_binary_name() {
        let name = get_ra_binary_name();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_cache_new() {
        let _cache: Cache<String, String> = Cache::new(
            std::time::Duration::from_secs(60),
            100,
        );
    }
}