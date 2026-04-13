use serde_json::Value;

pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

fn schema_object(properties: &[(&str, &str, &str)], required: &[&str]) -> Value {
    let mut props = serde_json::Map::new();
    for (key, typ, desc) in properties {
        let mut prop = serde_json::Map::new();
        prop.insert("type".to_string(), Value::String(typ.to_string()));
        prop.insert("description".to_string(), Value::String(desc.to_string()));
        props.insert(key.to_string(), Value::Object(prop));
    }
    let mut obj = serde_json::Map::new();
    obj.insert("type".to_string(), Value::String("object".to_string()));
    obj.insert("properties".to_string(), Value::Object(props));
    obj.insert(
        "required".to_string(),
        Value::Array(
            required
                .iter()
                .map(|s| Value::String(s.to_string()))
                .collect(),
        ),
    );
    Value::Object(obj)
}

fn schema_object_optional(properties: &[(&str, &str, &str)]) -> Value {
    let mut props = serde_json::Map::new();
    for (key, typ, desc) in properties {
        let mut prop = serde_json::Map::new();
        prop.insert("type".to_string(), Value::String(typ.to_string()));
        prop.insert("description".to_string(), Value::String(desc.to_string()));
        props.insert(key.to_string(), Value::Object(prop));
    }
    let mut obj = serde_json::Map::new();
    obj.insert("type".to_string(), Value::String("object".to_string()));
    obj.insert("properties".to_string(), Value::Object(props));
    Value::Object(obj)
}

fn schema_empty() -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("type".to_string(), Value::String("object".to_string()));
    obj.insert(
        "properties".to_string(),
        Value::Object(serde_json::Map::new()),
    );
    Value::Object(obj)
}

pub fn get_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "open_project",
            description: "Open a Rust project workspace for analysis",
            input_schema: schema_object(
                &[(
                    "workspace_root",
                    "string",
                    "Absolute path to the Cargo project root",
                )],
                &["workspace_root"],
            ),
        },
        ToolDefinition {
            name: "status",
            description: "Get rust-analyzer status, version, and workspace info",
            input_schema: schema_empty(),
        },
        ToolDefinition {
            name: "goto_definition",
            description: "Jump to the definition of a symbol at a given position",
            input_schema: schema_object(
                &[
                    ("file_path", "string", "Absolute path to the file"),
                    ("line", "integer", "1-based line number"),
                    ("column", "integer", "1-based column number"),
                ],
                &["file_path", "line", "column"],
            ),
        },
        ToolDefinition {
            name: "find_references",
            description: "Find all references to a symbol at a given position",
            input_schema: schema_object(
                &[
                    ("file_path", "string", "Path to the file"),
                    ("line", "integer", "Line number"),
                    ("column", "integer", "Column number"),
                    (
                        "include_declaration",
                        "boolean",
                        "Include declaration in results",
                    ),
                ],
                &["file_path", "line", "column"],
            ),
        },
        ToolDefinition {
            name: "hover",
            description: "Get type info and documentation for a symbol at position",
            input_schema: schema_object(
                &[
                    ("file_path", "string", "Path to the file"),
                    ("line", "integer", "Line number"),
                    ("column", "integer", "Column number"),
                ],
                &["file_path", "line", "column"],
            ),
        },
        ToolDefinition {
            name: "completions",
            description: "Get code completions at a position",
            input_schema: schema_object(
                &[
                    ("file_path", "string", "Path to the file"),
                    ("line", "integer", "Line number"),
                    ("column", "integer", "Column number"),
                    ("trigger_character", "string", "Completion trigger"),
                ],
                &["file_path", "line", "column"],
            ),
        },
        ToolDefinition {
            name: "get_diagnostics",
            description: "Get all compiler errors/warnings for a file or entire workspace",
            input_schema: schema_object_optional(&[(
                "file_path",
                "string",
                "Specific file, or omit for entire workspace",
            )]),
        },
        ToolDefinition {
            name: "code_action",
            description: "Get available code actions (fixes, refactors) at a position",
            input_schema: schema_object(
                &[
                    ("file_path", "string", "Path to the file"),
                    ("line", "integer", "Line number"),
                    ("column", "integer", "Column number"),
                    ("kind", "string", "Filter by action kind"),
                ],
                &["file_path", "line", "column"],
            ),
        },
        ToolDefinition {
            name: "rename_symbol",
            description: "Rename a symbol across the entire workspace",
            input_schema: schema_object(
                &[
                    ("file_path", "string", "Path to the file"),
                    ("line", "integer", "Line number"),
                    ("column", "integer", "Column number"),
                    ("new_name", "string", "New name for the symbol"),
                ],
                &["file_path", "line", "column", "new_name"],
            ),
        },
        ToolDefinition {
            name: "workspace_symbol",
            description: "Search for symbols across the entire workspace by name",
            input_schema: schema_object(
                &[
                    ("query", "string", "Fuzzy search query"),
                    ("limit", "integer", "Maximum results (default: 50)"),
                ],
                &["query"],
            ),
        },
        ToolDefinition {
            name: "file_structure",
            description: "Get the symbol outline/structure of a file",
            input_schema: schema_object(
                &[("file_path", "string", "Path to the file")],
                &["file_path"],
            ),
        },
        ToolDefinition {
            name: "format_file",
            description: "Format a file using rustfmt via rust-analyzer",
            input_schema: schema_object(
                &[("file_path", "string", "Path to the file")],
                &["file_path"],
            ),
        },
        ToolDefinition {
            name: "inlay_hints",
            description: "Get inlay hints (type annotations, parameter names) for a file",
            input_schema: schema_object(
                &[("file_path", "string", "Path to the file")],
                &["file_path"],
            ),
        },
        ToolDefinition {
            name: "expand_macro",
            description: "Expand a macro at a given position to see generated code",
            input_schema: schema_object(
                &[
                    ("file_path", "string", "Path to the file"),
                    ("line", "integer", "Line number"),
                    ("column", "integer", "Column number"),
                ],
                &["file_path", "line", "column"],
            ),
        },
        ToolDefinition {
            name: "runnables",
            description: "List all runnable items (tests, benches, binaries) in project",
            input_schema: schema_object_optional(&[(
                "file_path",
                "string",
                "Optional: limit to a specific file",
            )]),
        },
        ToolDefinition {
            name: "view_hir",
            description: "View the HIR (High-level IR) of a function or item",
            input_schema: schema_object(
                &[
                    ("file_path", "string", "Path to the file"),
                    ("line", "integer", "Line number"),
                    ("column", "integer", "Column number"),
                ],
                &["file_path", "line", "column"],
            ),
        },
        ToolDefinition {
            name: "check_update",
            description: "Check if a newer version of rust-analyzer is available",
            input_schema: schema_empty(),
        },
        ToolDefinition {
            name: "update_rust_analyzer",
            description: "Update rust-analyzer to the latest nightly release",
            input_schema: schema_object_optional(&[(
                "force",
                "boolean",
                "Force update even if same version",
            )]),
        },
        ToolDefinition {
            name: "health_check",
            description: "Get comprehensive health status of the MCP server",
            input_schema: schema_object_optional(&[(
                "include_details",
                "boolean",
                "Include detailed error logs and metrics",
            )]),
        },
        ToolDefinition {
            name: "switch_workspace",
            description: "Switch to a different Rust project workspace",
            input_schema: schema_object(
                &[("workspace_root", "string", "Path to the workspace")],
                &["workspace_root"],
            ),
        },
        ToolDefinition {
            name: "list_workspaces",
            description: "List all open workspaces and their crates",
            input_schema: schema_empty(),
        },
        ToolDefinition {
            name: "goto_type_definition",
            description: "Jump to the type definition of a symbol",
            input_schema: schema_object(
                &[
                    ("file_path", "string", "Path to the file"),
                    ("line", "integer", "Line number"),
                    ("column", "integer", "Column number"),
                ],
                &["file_path", "line", "column"],
            ),
        },
        ToolDefinition {
            name: "goto_implementation",
            description: "Find all implementations of a trait or struct",
            input_schema: schema_object(
                &[
                    ("file_path", "string", "Path to the file"),
                    ("line", "integer", "Line number"),
                    ("column", "integer", "Column number"),
                ],
                &["file_path", "line", "column"],
            ),
        },
    ]
}
