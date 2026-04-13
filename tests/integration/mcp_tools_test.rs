use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_definitions() {
        let tools = crate::mcp::tools::get_tools();
        assert!(!tools.is_empty());
        
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"open_project"));
        assert!(tool_names.contains(&"status"));
    }

    #[test]
    fn test_resources_definitions() {
        let resources = crate::mcp::resources::get_resources();
        assert!(!resources.is_empty());
    }

    #[test]
    fn test_prompts_definitions() {
        let prompts = crate::mcp::prompts::get_prompts();
        assert!(!prompts.is_empty());
    }

    #[test]
    fn test_config_defaults() {
        let config = crate::config::Config::default();
        assert_eq!(config.cache.ttl_seconds, 300);
        assert_eq!(config.cache.max_entries, 1000);
        assert!(config.cache.enabled);
    }

    #[test]
    fn test_lsp_config_defaults() {
        let config = crate::config::LspConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_security_config_defaults() {
        let config = crate::config::SecurityConfig::default();
        assert_eq!(config.max_file_size_mb, 10);
        assert!(config.prevent_path_traversal);
    }

    #[test]
    fn test_platform_info() {
        let (os, arch) = crate::utils::platform::get_platform();
        assert!(!os.is_empty());
        assert!(!arch.is_empty());
        
        let name = crate::utils::platform::get_ra_binary_name();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_cache_sync() {
        let cache = crate::cache::Cache::<String, String>::new(
            std::time::Duration::from_secs(60),
            100,
        );
        assert!(cache.is_async());
    }
}

#[test]
fn test_tools_not_empty() {
    let tools = crate::mcp::tools::get_tools();
    assert!(!tools.is_empty());
}

#[test]
fn test_tools_have_open_project() {
    let tools = crate::mcp::tools::get_tools();
    let has_open_project = tools.iter().any(|t| t.name == "open_project");
    assert!(has_open_project);
}

#[test]
fn test_resources_not_empty() {
    let resources = crate::mcp::resources::get_resources();
    assert!(!resources.is_empty());
}

#[test]
fn test_prompts_not_empty() {
    let prompts = crate::mcp::prompts::get_prompts();
    assert!(!prompts.is_empty());
}

#[test]
fn test_config_default() {
    let config = crate::config::Config::default();
    assert_eq!(config.cache.ttl_seconds, 300);
    assert_eq!(config.cache.max_entries, 1000);
}

#[test]
fn test_platform_get_platform() {
    let (os, arch) = crate::utils::platform::get_platform();
    assert!(!os.is_empty());
    assert!(!arch.is_empty());
}

#[test]
fn test_platform_get_ra_binary_name() {
    let name = crate::utils::platform::get_ra_binary_name();
    assert!(!name.is_empty());
}