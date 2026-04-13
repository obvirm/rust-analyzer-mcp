# rust-analyzer-mcp

MCP (Model Context Protocol) Server that wraps rust-analyzer LSP for AI assistants like Claude, Cursor, and other MCP-compatible clients.

## Features

- **Full LSP Integration**: Communicate with rust-analyzer via Language Server Protocol
- **MCP Tools**: 15+ tools for code analysis (goto definition, find references, hover, completions, etc.)
- **Auto-Update**: Automatically download latest rust-analyzer nightly
- **Workspace Management**: Multi-project workspace support
- **Caching**: Built-in result caching for performance

## Installation

### Prerequisites

- Rust 1.70+
- rust-analyzer (auto-installed if not found in PATH)

### Build from Source

```bash
git clone https://github.com/yourusername/rust-analyzer-mcp.git
cd rust-analyzer-mcp
cargo build --release
```

The binary will be at `target/release/rust-analyzer-mcp` (or `.exe` on Windows).

## Usage

### Command Line Options

```bash
rust-analyzer-mcp [OPTIONS]

Options:
      --ra-path <PATH>         Path to rust-analyzer binary
      --project-root <PATH>    Rust project root to analyze
  -c, --config <FILE>          Configuration file (default: config/default.toml)
      --no-auto-update         Disable auto-update checking
      --log-level <LEVEL>      Log level (default: info)
      --metrics                Enable metrics endpoint
      --health-check           Health check mode (exit after check)
  -h, --help                   Print help
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_ANALYZER_PATH` | Custom rust-analyzer path | auto-detect |
| `RUST_PROJECT_ROOT` | Default project root | - |
| `RA_MCP_AUTO_UPDATE` | Enable auto-update | true |
| `RA_MCP_LOG_LEVEL` | Log level | info |

### Configuration File

See `config/default.toml` for all options:

```toml
[rust_analyzer]
auto_update = true
update_channel = "nightly"

[lsp]
timeout_seconds = 30
max_retries = 3
crash_recovery = true

[cache]
enabled = true
ttl_seconds = 300
max_entries = 1000

[security]
max_file_size_mb = 10
prevent_path_traversal = true
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `open_project` | Open a Rust project workspace |
| `status` | Get server status and version |
| `goto_definition` | Jump to symbol definition |
| `find_references` | Find all references to a symbol |
| `hover` | Get hover information |
| `completions` | Get auto-completions |
| `diagnostics` | Get compile errors/warnings |
| `code_action` | Get available code actions |
| `document_symbol` | List symbols in a file |
| `workspace_symbol` | Search symbols across workspace |
| `inlay_hints` | Get inlay hints |
| `expand_macro` | Expand macro at cursor |
| `format` | Format file with rustfmt |
| `rename` | Rename a symbol |

## MCP Resources

| Resource | Description |
|----------|-------------|
| `rust://version` | Current rust-analyzer version |
| `rust://config` | Server configuration |

## MCP Prompts

| Prompt | Description |
|--------|-------------|
| `analyze_code` | Analyze code at location |
| `explain_error` | Explain compiler error |

## Development

### Run Tests

```bash
cargo test
```

### Run with Debug Logging

```bash
RUST_LOG=debug cargo run
```

### Health Check

```bash
cargo run -- --health-check
```

## Architecture

```
┌─────────────┐     JSON-RPC     ┌─────────────┐
│   MCP       │ ◄──────────────► │   rust-     │
│   Client    │                  │   analyzer  │
└─────────────┘                  └─────────────┘
      │
      ▼
┌─────────────────────────────────────────────┐
│              McpServer                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │
│  │  Tools  │  │Resources │  │   Prompts   │  │
│  └─────────┘  └─────────┘  └─────────────┘  │
│  ┌─────────────────────────────────────────┐│
│  │         LspClient (LSP protocol)        ││
│  └─────────────────────────────────────────┘│
└─────────────────────────────────────────────┘
```

## License

MIT OR Apache-2.0