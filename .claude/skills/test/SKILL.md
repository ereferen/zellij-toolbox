---
name: test
description: Run project tests with cargo
argument-hint: "[all|core|cli]"
allowed-tools: "Bash(cargo:*)"
---

# /test - Run tests

Run project tests with cargo.

## Usage

- `/test` - Run all tests
- `/test core` - Run only toolbox-core tests
- `/test cli` - Run only toolbox-cli tests

## Instructions

When the user runs this skill:

1. Run the appropriate test command:
   - No args: `cargo test --workspace`
   - "core": `cargo test -p toolbox-core`
   - "cli": `cargo test -p toolbox-cli`
2. Report test results clearly
3. If tests fail, analyze the error and suggest fixes
