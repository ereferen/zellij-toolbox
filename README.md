# Zellij Toolbox

A Zellij plugin for displaying development tool versions at a glance.

## Features

- Display versions of 24+ development tools (Python, Node, Rust, Go, Docker, etc.)
- Support for asdf/mise/nvm directory-specific versions
- Configurable tool list with custom tool definitions and overrides
- Git repository information (branch, status, ahead/behind remote tracking)
- Optional system info (memory, CPU)
- Powerline-style colored output (single-line and multiline)
- Virtual environment detection (Python venv, Conda)
- CLI tool for standalone usage
- Zellij WASM plugin with auto-refresh

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
#  🌿 main (+3 ↑2)
# ───────────────
#  🐍 py 3.12.1
#  📦 node 20.11
#  🦀 rust 1.75
#  🔷 go 1.21
#
# Git status format:
#  +N  = N local changes (modified/staged/untracked)
#  ↑N  = N commits ahead of remote
#  ↓N  = N commits behind remote

# Powerline-style output (colored segments)
toolbox --powerline

# Single-line powerline (for status bars)
toolbox --powerline --single-line

# Color control
toolbox --color always    # Force colors
toolbox --color never     # No colors

# Compact mode (shorter output)
toolbox --compact

# JSON output
toolbox --format json

# Specify directory (for asdf/mise)
toolbox --dir /path/to/project

# Initialize config file
toolbox init

# Show current configuration
toolbox show-config

# List available tools
toolbox list-tools
```

### Zellij Plugin

Create a layout file to use the plugin:

```kdl
// ~/.config/zellij/layouts/toolbox.kdl
layout {
    pane size=1 borderless=true {
        plugin location="file:~/.config/zellij/plugins/toolbox_zellij.wasm" {
            refresh_interval "5"           // Refresh every 5 seconds
            working_dir "/path/to/project" // Optional: for asdf/mise support
            single_line "true"             // Optional: single-line display
            powerline "true"               // Optional: powerline-style output
        }
    }
    pane
}
```

Launch Zellij with the layout:

```bash
zellij --layout toolbox
```

The plugin will:
- Display tool versions at the top of your terminal
- Auto-refresh every N seconds (configurable)
- Show Git branch and status with ahead/behind tracking
- Support powerline-style colored output
- Handle Unicode character widths correctly (emojis, CJK characters)

### Configuration

Config file location: `~/.config/toolbox/config.toml`

```toml
[display]
refresh_interval = 5
show_icons = true
compact = true

# Override settings for default tools
[[tool_overrides]]
name = "Ruby"
enabled = true          # Enable a tool disabled by default

[[tool_overrides]]
name = "Python"
icon = "🐍"             # Change icon

# Add completely custom tools
[[custom_tools]]
name = "My Tool"
command = "my-tool --version"
parse_regex = "v(\\d+\\.\\d+\\.\\d+)"
icon = "🔧"
enabled = true
short_name = "mytool"

[extras]
git_branch = true
git_status = true
current_directory = true
virtual_env = true
shell = true
system_memory = false
system_cpu = false
```

> **Note:** Default tools (Python, Node, Rust, Go, Docker, etc.) are included automatically.
> Use `use_default_tools = false` in `[display]` to disable all defaults and define tools manually.

## Default Tools

The following 24 tools are pre-configured (enabled tools marked with *):

| Category | Tools |
|----------|-------|
| Languages | Python*, Node*, Rust*, Go*, Ruby, Java, PHP, Elixir, Zig |
| Runtimes | Deno, Bun |
| Package Managers | npm, pnpm, yarn |
| Containers | Docker* |
| DevOps | kubectl, Terraform, AWS CLI |
| Version Managers | mise, asdf |

> Tools marked with * are enabled by default. Others can be enabled via configuration overrides.

## Development

### Prerequisites

- Rust 1.70+
- For Zellij plugin: `wasm32-wasip1` target

### Build

```bash
# Build all (CLI only, native target)
cargo build

# Build release
cargo build --release

# Build WASM plugin
cargo build -p toolbox-zellij --target wasm32-wasip1 --release
```

### Test

```bash
# Run all tests (69 unit tests)
cargo test

# Run specific crate tests
cargo test -p toolbox-core
```

### Code Quality

```bash
# Format check
cargo fmt --check

# Lint
cargo clippy -- -D warnings

# Full check
cargo check
```

### With DevContainer

This project includes a devcontainer configuration for VS Code / GitHub Codespaces with all necessary tools pre-installed (Rust, Node.js, Python, Go, Docker, GitHub CLI).

## Architecture

```
┌──────────────┐     ┌──────────────────┐
│ toolbox-cli  │────>│   toolbox-core   │
│  (clap CLI)  │     │  (config, detect,│
└──────────────┘     │   info, color)   │
                     └──────────────────┘
┌──────────────┐              │
│toolbox-zellij│──run_command─┘
│ (WASM plugin)│─────> toolbox CLI binary
└──────────────┘
```

The Zellij plugin cannot execute commands directly from WASM. It calls the `toolbox` CLI binary via Zellij's `run_command()` API and parses the output.

## License

MIT
