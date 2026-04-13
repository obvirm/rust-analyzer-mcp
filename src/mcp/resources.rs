pub struct Resource {
    pub uri: &'static str,
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub mime_type: Option<&'static str>,
}

pub fn get_resources() -> Vec<Resource> {
    vec![
        Resource {
            uri: "health://status",
            name: "Health Status",
            description: Some("Server health and status information"),
            mime_type: Some("application/json"),
        },
        Resource {
            uri: "metrics://server",
            name: "Server Metrics",
            description: Some("Performance metrics for the MCP server"),
            mime_type: Some("application/json"),
        },
        Resource {
            uri: "config://current",
            name: "Current Configuration",
            description: Some("Current server configuration"),
            mime_type: Some("application/json"),
        },
        Resource {
            uri: "version://rust-analyzer",
            name: "rust-analyzer Version",
            description: Some("Current rust-analyzer version information"),
            mime_type: Some("text/plain"),
        },
    ]
}
