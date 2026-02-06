//! Zellij plugin for toolbox
//!
//! This plugin displays tool versions in a Zellij pane.
//! Note: WASM environment has limitations - we can't directly run commands.
//! The plugin will need to communicate with the CLI tool for updates.

// This crate is a Zellij WASM plugin. For native targets we build a tiny stub
// binary so `cargo build` for the workspace succeeds.

#[cfg(target_arch = "wasm32")]
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
use unicode_width::UnicodeWidthChar;

#[cfg(target_arch = "wasm32")]
use zellij_tile::prelude::*;

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct ToolboxPlugin {
    /// Display content
    content: Vec<String>,
    /// Plugin width
    cols: usize,
    /// Plugin height
    rows: usize,
    /// Refresh interval in seconds
    refresh_interval: f64,
    /// Working directory for tool detection
    working_dir: Option<String>,
    /// Single line display mode
    single_line: bool,
    /// Powerline style output
    powerline: bool,
    /// Theme preset name (default, dark, light, solarized)
    theme: Option<String>,
}

#[cfg(target_arch = "wasm32")]
register_plugin!(ToolboxPlugin);

#[cfg(target_arch = "wasm32")]
impl ZellijPlugin for ToolboxPlugin {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        // Request permissions
        request_permission(&[
            PermissionType::RunCommands,
            PermissionType::ReadApplicationState,
        ]);

        // Subscribe to events
        subscribe(&[
            EventType::ModeUpdate,
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::RunCommandResult,
            EventType::Timer,
        ]);

        // Read refresh interval from configuration (default: 5 seconds)
        self.refresh_interval = configuration
            .get("refresh_interval")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(5.0);

        // Read working directory from configuration
        self.working_dir = configuration.get("working_dir").cloned();

        // Read single line mode from configuration (default: false)
        self.single_line = configuration
            .get("single_line")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);

        // Read powerline mode from configuration (default: false)
        self.powerline = configuration
            .get("powerline")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);

        // Read theme preset from configuration
        self.theme = configuration.get("theme").cloned();

        // Initial content (use marker for dynamic separator)
        self.content = vec![
            "---".to_string(),
            " Loading...".to_string(),
            "---".to_string(),
        ];

        // Trigger first refresh immediately (short timer to avoid I/O error in load)
        set_timeout(0.1);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::RunCommandResult(exit_code, stdout, stderr, _context) => {
                if exit_code == Some(0) {
                    self.parse_output(&stdout);
                } else {
                    self.content = vec![
                        "---".to_string(),
                        " Error".to_string(),
                        format!(" {}", String::from_utf8_lossy(&stderr)),
                        "---".to_string(),
                    ];
                }
                true
            }
            Event::Timer(_elapsed) => {
                // Periodic refresh
                self.request_tool_versions();
                // Schedule next refresh
                set_timeout(self.refresh_interval);
                false
            }
            Event::PaneUpdate(pane_manifest) => {
                // Could track active pane's working directory here
                // and refresh tool versions when it changes
                let _ = pane_manifest;
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        self.rows = rows;
        self.cols = cols;

        if self.single_line {
            // Single line mode: join all non-separator lines (no trailing newline)
            let line = self.build_single_line();
            let display_line = truncate_to_width(&line, cols);
            print!("{}", display_line);
        } else {
            // Multi-line mode
            for (i, line) in self.content.iter().enumerate() {
                if i >= rows {
                    break;
                }
                // Check if this is a separator line (starts with ─ or is "---" marker)
                let display_line = if line.starts_with('─') || line == "---" {
                    "─".repeat(cols)
                } else {
                    truncate_to_width(line, cols)
                };
                println!("{}", display_line);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl ToolboxPlugin {
    fn request_tool_versions(&self) {
        // Run the toolbox CLI to get versions
        // The CLI should be installed and in PATH
        let mut args = vec!["toolbox", "--format", "text", "--compact"];

        // Add powerline flag if enabled
        if self.powerline {
            args.push("--powerline");
            args.push("--color");
            args.push("always");

            // Add single line flag if enabled
            if self.single_line {
                args.push("--single-line");
            }
        }

        // Add theme if configured
        let theme_arg;
        if let Some(ref theme) = self.theme {
            args.push("--theme");
            theme_arg = theme.clone();
            args.push(&theme_arg);
        }

        // Add working directory if configured
        let dir_arg;
        if let Some(ref dir) = self.working_dir {
            args.push("--dir");
            dir_arg = dir.clone();
            args.push(&dir_arg);
        }

        run_command(&args, BTreeMap::new());
    }

    fn parse_output(&mut self, stdout: &[u8]) {
        let output = String::from_utf8_lossy(stdout);
        self.content = output.lines().map(String::from).collect();

        // Ensure at least some content
        if self.content.is_empty() {
            self.content = vec![" No tools detected".to_string()];
        }
    }

    fn build_single_line(&self) -> String {
        // Filter out separators and join with " | "
        let parts: Vec<&str> = self
            .content
            .iter()
            .filter(|line| {
                !line.starts_with('─') && line.as_str() != "---" && !line.trim().is_empty()
            })
            .map(|s| s.trim())
            .collect();
        parts.join(" | ")
    }
}

/// Truncate a string to fit within a given display width
/// Accounts for Unicode character widths (e.g., emojis are width 2)
/// Properly skips ANSI escape sequences (they have zero display width)
#[cfg(target_arch = "wasm32")]
fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut current_width = 0;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        // Check for ANSI escape sequence start
        if c == '\x1b' {
            // Add the escape character
            result.push(c);

            // Check for CSI sequence (ESC [)
            if chars.peek() == Some(&'[') {
                result.push(chars.next().unwrap()); // consume '['

                // Read until we hit a letter (the final byte of the sequence)
                while let Some(&next_c) = chars.peek() {
                    result.push(chars.next().unwrap());
                    // CSI sequences end with a letter in range 0x40-0x7E
                    if next_c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            // ANSI sequences have zero display width, so don't add to current_width
            continue;
        }

        let char_width = UnicodeWidthChar::width(c).unwrap_or(0);
        if current_width + char_width > max_width {
            break;
        }
        result.push(c);
        current_width += char_width;
    }

    result
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!(
        "toolbox-zellij is a Zellij WASM plugin. Build it with: cargo build -p toolbox-zellij --release --target wasm32-wasip1"
    );
}
