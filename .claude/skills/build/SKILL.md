---
name: build
description: Build the toolbox project (CLI or WASM plugin)
argument-hint: "[all|release|cli|wasm]"
allowed-tools: "Bash(cargo:*)"
---

# /build - Build the project

Build the toolbox project.

## Usage

- `/build` - Build all crates in debug mode
- `/build release` - Build all crates in release mode
- `/build cli` - Build only the CLI
- `/build wasm` - Build the Zellij plugin (WASM)

## Instructions

When the user runs this skill:

1. Parse the argument to determine what to build
2. Run the appropriate cargo command:
   - No args or "all": `cargo build`
   - "release": `cargo build --release`
   - "cli": `cargo build -p toolbox-cli`
   - "wasm": `cargo build -p toolbox-zellij --target wasm32-wasip1 --release`
3. Report any errors clearly
4. On success, show the built artifacts location
