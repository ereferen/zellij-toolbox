# Zellij Toolbox

A Zellij plugin for displaying development tool versions at a glance.

## Features

- Display versions of common development tools (Python, Node, Rust, Go, Docker, etc.)
- Support for asdf/mise/nvm directory-specific versions
- Configurable tool list with custom tool support
- Git repository information (branch, status)
- Optional system info (memory, CPU)
- CLI tool for standalone usage

## Installation

### CLI Tool

```bash
cargo install --path toolbox-cli
```

### Zellij Plugin

```bash
# Build WASM plugin
rustup target add wasm32-wasip1
cargo build -p toolbox-zellij --target wasm32-wasip1 --release

# Copy to Zellij plugins directory
cp target/wasm32-wasip1/release/toolbox_zellij.wasm ~/.config/zellij/plugins/
```

## Usage

### CLI

```bash
# Basic usage
toolbox

# Output example:
#  📂 ~/project
#  🌿 main (+3)
# ───────────────
#  🐍 py 3.12.1
#  📦 node 20.11
#  🦀 rust 1.75
#  🔷 go 1.21

# JSON output
toolbox --format json

# Specify directory (for asdf/mise)
toolbox --dir /path/to/project

# Initialize config file
toolbox init

# List available tools
toolbox list-tools
```

### Configuration

Config file location: `~/.config/toolbox/config.toml`

```toml
[display]
refresh_interval = 5
show_icons = true
compact = true

[[tools]]
name = "Python"
command = "python3 --version"
parse_regex = "Python\\s+(\\d+\\.\\d+(?:\\.\\d+)?)"
icon = "🐍"
enabled = true
short_name = "py"

# Add custom tools
[[tools]]
name = "My Tool"
command = "my-tool --version"
enabled = true

[extras]
git_branch = true
git_status = true
current_directory = true
virtual_env = true
system_memory = false
system_cpu = false
```

## Development

### Prerequisites

- Rust 1.70+
- For Zellij plugin: `wasm32-wasip1` target

### Build

```bash
# Build all
cargo build

# Build release
cargo build --release

# Build WASM plugin
cargo build -p toolbox-zellij --target wasm32-wasip1 --release
```

### Test

```bash
cargo test
```

### With DevContainer

This project includes a devcontainer configuration for VS Code / GitHub Codespaces with all necessary tools pre-installed.

## License

MIT
