#[test]
fn test_tools_not_empty() {
    let tools = crate::mcp::tools::get_tools();
    assert!(!tools.is_empty(), "Tools should not be empty");
}

#[test]
fn test_tools_have_required_fields() {
    let tools = crate::mcp::tools::get_tools();
    for tool in tools.iter().take(3) {
        assert!(!tool.name.is_empty(), "Tool name should not be empty");
        assert!(!tool.description.is_empty(), "Tool description should not be empty");
    }
}

#[test]
fn test_tools_have_open_project() {
    let tools = crate::mcp::tools::get_tools();
    let has_open_project = tools.iter().any(|t| t.name == "open_project");
    assert!(has_open_project, "Should have open_project tool");
}

#[test]
fn test_resources_not_empty() {
    let resources = crate::mcp::resources::get_resources();
    assert!(!resources.is_empty(), "Resources should not be empty");
}

#[test]
fn test_prompts_not_empty() {
    let prompts = crate::mcp::prompts::get_prompts();
    assert!(!prompts.is_empty(), "Prompts should not be empty");
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