---
name: add-tool
description: Add a new tool definition to toolbox-core's default tools
argument-hint: "<name> <command>"
---

# /add-tool - Add a new tool to the default configuration

Add a new tool definition to toolbox-core's default tools.

## Usage

- `/add-tool <name> <command>` - Add a new tool with basic settings
- Example: `/add-tool pnpm "pnpm --version"`

## Instructions

When the user runs this skill:

1. Parse the tool name and command from arguments
2. Open `toolbox-core/src/config.rs`
3. Find the `default_tools()` function
4. Add a new `ToolConfig` entry with:
   - name: The provided name
   - command: The provided command
   - parse_regex: Infer a reasonable regex pattern (usually capturing version numbers)
   - icon: Suggest an appropriate emoji
   - enabled: false (let users enable it)
   - short_name: A shorter version of the name
5. Run `cargo check` to verify the change compiles
6. Show the user the added configuration
