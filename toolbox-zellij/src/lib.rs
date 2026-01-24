//! Zellij plugin for toolbox
//!
//! This plugin displays tool versions in a Zellij pane.
//! Note: WASM environment has limitations - we can't directly run commands.
//! The plugin will need to communicate with the CLI tool for updates.

use std::collections::BTreeMap;
use zellij_tile::prelude::*;

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
}

register_plugin!(ToolboxPlugin);

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

        // Initial content
        self.content = vec![
            "─".repeat(15),
            " Loading...".to_string(),
            "─".repeat(15),
        ];

        // Request tool versions
        self.request_tool_versions();

        // Start periodic refresh
        set_timeout(self.refresh_interval);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::RunCommandResult(exit_code, stdout, stderr, _context) => {
                if exit_code == Some(0) {
                    self.parse_output(&stdout);
                } else {
                    self.content = vec![
                        "─".repeat(15),
                        " Error".to_string(),
                        format!(" {}", String::from_utf8_lossy(&stderr)),
                        "─".repeat(15),
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

        for (i, line) in self.content.iter().enumerate() {
            if i >= rows {
                break;
            }
            // Truncate line if too long
            let display_line = if line.len() > cols {
                &line[..cols]
            } else {
                line
            };
            println!("{}", display_line);
        }
    }
}

impl ToolboxPlugin {
    fn request_tool_versions(&self) {
        // Run the toolbox CLI to get versions
        // The CLI should be installed and in PATH
        run_command(
            &["toolbox", "--format", "text", "--compact"],
            BTreeMap::new(),
        );
    }

    fn parse_output(&mut self, stdout: &[u8]) {
        let output = String::from_utf8_lossy(stdout);
        self.content = output.lines().map(String::from).collect();

        // Ensure at least some content
        if self.content.is_empty() {
            self.content = vec![" No tools detected".to_string()];
        }
    }
}
