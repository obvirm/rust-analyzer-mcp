# rust-analyzer-mcp

MCP (Model Context Protocol) Server yang membungkus rust-analyzer LSP untuk AI assistants seperti Claude, Cursor, dan client MCP lainnya.

## Fitur

- **Full LSP Integration**: Komunikasi dengan rust-analyzer via Language Server Protocol
- **MCP Tools**: 15+ tools untuk analisis kode (goto definition, find references, hover, completions, dll)
- **Auto-Update**: Download rust-analyzer nightly terbaru otomatis
- **Workspace Management**: Multi-project workspace support
- **Caching**: Built-in result caching untuk performa

## Cara Pasang

### 1. Clone & Build

```bash
git clone https://github.com/username/rust-analyzer-mcp.git
cd rust-analyzer-mcp
cargo build --release
```

Binary ada di `target/release/rust-analyzer-mcp.exe` (Windows) atau `target/release/rust-analyzer-mcp` (Linux/Mac).

### 2. Konfigurasi MCP Client

#### VS Code / Cursor / Windsurf

Tambahkan ke `.vscode/mcp.json`:

```json
{
  "mcpServers": {
    "rust-analyzer": {
      "command": "path/to/rust-analyzer-mcp.exe",
      "args": ["--project-root", "${workspaceFolder}"]
    }
  }
}
```

#### Claude Desktop

Tambahkan ke `~/Library/Application Support/Claude/claude_desktop_config.json` (Mac) atau `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "rust-analyzer": {
      "command": "path/to/rust-analyzer-mcp.exe",
      "args": ["--project-root", "${workspaceFolder}"]
    }
  }
}
```

#### OpenCode

Tambahkan ke `~/.config/opencode/opencode.json`:

```json
{
  "mcp": {
    "rust-analyzer": {
      "command": ["path/to/rust-analyzer-mcp.exe"],
      "args": ["--project-root", "${workspaceFolder}"],
      "enabled": true,
      "type": "local"
    }
  }
}
```

## Cara Pakai

### Command Line Options

```bash
rust-analyzer-mcp [OPTIONS]

Options:
      --ra-path <PATH>         Path ke rust-analyzer binary
      --project-root <PATH>    Project root yang dianalisa
  -c, --config <FILE>          Config file (default: config/default.toml)
      --no-auto-update         Disable auto-update
      --log-level <LEVEL>      Log level (default: info)
      --health-check           Health check mode (exit setelah check)
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

Lihat `config/default.toml` untuk semua opsi:

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
| `open_project` | Buka Rust project workspace |
| `status` | Get status dan version server |
| `goto_definition` | Lompat ke simbol definition |
| `find_references` | Cari semua referensi ke simbol |
| `hover` | Get informasi hover |
| `completions` | Get auto-completions |
| `diagnostics` | Get error/warning kompilasi |
| `code_action` | Get code actions yang tersedia |
| `document_symbol` | List simbol di file |
| `workspace_symbol` | Cari simbol di seluruh workspace |
| `inlay_hints` | Get inlay hints |
| `expand_macro` | Expand macro di cursor |
| `format` | Format file dengan rustfmt |
| `rename` | Rename simbol |

## MCP Resources

| Resource | Description |
|----------|-------------|
| `rust://version` | Current rust-analyzer version |
| `rust://config` | Server configuration |

## MCP Prompts

| Prompt | Description |
|--------|-------------|
| `analyze_code` | Analyze code di lokasi |
| `explain_error` | Explain error compiler |

## Development

### Run Tests

```bash
cargo test
```

### Run dengan Debug Logging

```bash
RUST_LOG=debug cargo run
```

### Health Check

```bash
cargo run -- --health-check
```

## Troubleshooting

### rust-analyzer tidak ditemukan

```bash
# Install rust-analyzer
rustup component add rust-analyzer

# atau via system package manager
brew install rust-analyzer  # macOS
sudo apt install rust-analyzer  # Linux
```

### MCP tidak connect

1. Pastikan binary sudah di-build dengan benar
2. Cek log dengan `--log-level debug`
3. Verify rust-analyzer accessible: `cargo run -- --health-check`

### Workspace tidak terdeteksi

Pastikan ada `Cargo.toml` di project root yang ingin di-open.

## Architecture

```
             JSON-RPC
┌─────────────┐     ┌─────────────┐
│   MCP       │ ◄──► │   rust-     │
│   Client    │     │   analyzer  │
└─────────────┘     └─────────────┘
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