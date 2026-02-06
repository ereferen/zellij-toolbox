---
name: check
description: Run code quality checks (fmt, clippy, cargo check)
argument-hint: "[fix]"
allowed-tools: "Bash(cargo:*)"
---

# /check - Check code quality

Run code quality checks (clippy, fmt, check).

## Usage

- `/check` - Run all checks (fmt, clippy, cargo check)
- `/check fix` - Auto-fix formatting and clippy warnings

## Instructions

When the user runs this skill:

1. Run checks in order:
   ```bash
   cargo fmt --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo check
   ```

2. If "fix" argument is provided:
   ```bash
   cargo fmt
   cargo clippy --fix --allow-dirty
   ```

3. Report any issues found
4. Suggest fixes for any remaining problems
