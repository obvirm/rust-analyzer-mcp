# rust-analyzer-mcp Windows Installer

param(
    [string]$InstallPath = "$env:LOCALAPPDATA\rust-analyzer-mcp",
    [switch]$AddToPath
)

Write-Host "Installing rust-analyzer-mcp..." -ForegroundColor Cyan

# Check if Rust/Cargo is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Rust/Cargo is not installed" -ForegroundColor Red
    Write-Host "Install from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Build the project
Write-Host "Building..." -ForegroundColor Yellow
cargo build --release

# Create install directory
if (-not (Test-Path $InstallPath)) {
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
}

# Copy binary
$BinaryPath = "target\release\rust-analyzer-mcp.exe"
if (Test-Path $BinaryPath) {
    Copy-Item $BinaryPath "$InstallPath\rust-analyzer-mcp.exe" -Force
} else {
    $BinaryPath = "target\debug\rust-analyzer-mcp.exe"
    if (Test-Path $BinaryPath) {
        Copy-Item $BinaryPath "$InstallPath\rust-analyzer-mcp.exe" -Force
    } else {
        Write-Host "Error: Binary not found. Run cargo build first." -ForegroundColor Red
        exit 1
    }
}

Write-Host "Installed to: $InstallPath" -ForegroundColor Green

# Add to PATH if requested
if ($AddToPath) {
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $currentPath.Contains($InstallPath)) {
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallPath", "User")
        Write-Host "Added to PATH" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "Usage:" -ForegroundColor Cyan
Write-Host "  rust-analyzer-mcp.exe --help"
Write-Host "  rust-analyzer-mcp.exe --project-root C:\path\to\rust\project"