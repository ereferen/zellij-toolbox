//! Information structures for toolbox output

use serde::{Deserialize, Serialize};

/// Complete toolbox information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolboxInfo {
    /// Current directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_dir: Option<String>,
    /// Git information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
    /// Tool versions
    pub tools: Vec<ToolInfo>,
    /// System information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemInfo>,
    /// Virtual environment name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_env: Option<String>,
    /// Shell name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
}

/// Information about a single tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Short name for compact display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    /// Detected version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Icon/emoji
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Whether the tool is available
    pub available: bool,
    /// Error message if detection failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolInfo {
    /// Create a new ToolInfo for an available tool
    pub fn available(name: String, version: String) -> Self {
        Self {
            name,
            short_name: None,
            version: Some(version),
            icon: None,
            available: true,
            error: None,
        }
    }

    /// Create a new ToolInfo for an unavailable tool
    pub fn unavailable(name: String, error: Option<String>) -> Self {
        Self {
            name,
            short_name: None,
            version: None,
            icon: None,
            available: false,
            error,
        }
    }

    /// Set the icon
    pub fn with_icon(mut self, icon: Option<String>) -> Self {
        self.icon = icon;
        self
    }

    /// Set the short name
    pub fn with_short_name(mut self, short_name: Option<String>) -> Self {
        self.short_name = short_name;
        self
    }
}

/// Git repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    /// Current branch name
    pub branch: String,
    /// Number of modified files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_count: Option<usize>,
    /// Number of staged files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staged_count: Option<usize>,
    /// Number of untracked files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub untracked_count: Option<usize>,
    /// Whether there are uncommitted changes
    pub is_dirty: bool,
    /// Ahead/behind remote
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ahead: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behind: Option<usize>,
}

impl GitInfo {
    /// Get a summary string like "+3 -1" for changes
    pub fn changes_summary(&self) -> Option<String> {
        let mut parts = Vec::new();

        let total_changes = self.modified_count.unwrap_or(0)
            + self.staged_count.unwrap_or(0)
            + self.untracked_count.unwrap_or(0);

        if total_changes > 0 {
            parts.push(format!("+{}", total_changes));
        }

        if !parts.is_empty() {
            Some(parts.join(" "))
        } else {
            None
        }
    }

    /// Get ahead/behind summary like "↑2 ↓1"
    pub fn ahead_behind_summary(&self) -> Option<String> {
        let mut parts = Vec::new();

        if let Some(ahead) = self.ahead {
            parts.push(format!("↑{}", ahead));
        }

        if let Some(behind) = self.behind {
            parts.push(format!("↓{}", behind));
        }

        if !parts.is_empty() {
            Some(parts.join(" "))
        } else {
            None
        }
    }
}

/// Status of a tool diagnostic check
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticStatus {
    /// Tool found and version parsed successfully
    Ok,
    /// Tool found but version parse had issues
    Warning,
    /// Tool not found or command execution failed
    Error,
}

/// Diagnostic result for a single tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDiagnostic {
    /// Tool name
    pub name: String,
    /// Icon/emoji
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Diagnostic status
    pub status: DiagnosticStatus,
    /// The command that was checked
    pub command: String,
    /// Resolved path of the command binary (from which)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_path: Option<String>,
    /// Detected version (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Detailed error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<String>,
    /// Suggestion for fixing the issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    /// Whether the tool is enabled in config
    pub enabled: bool,
}

impl ToolDiagnostic {
    /// Format a single diagnostic line for display
    pub fn format_display(&self) -> String {
        let status_icon = match self.status {
            DiagnosticStatus::Ok => "OK",
            DiagnosticStatus::Warning => "WARN",
            DiagnosticStatus::Error => "ERR",
        };

        let icon = self.icon.as_deref().unwrap_or(" ");
        let enabled_tag = if self.enabled { "" } else { " (disabled)" };

        let mut line = match &self.status {
            DiagnosticStatus::Ok => {
                let version = self.version.as_deref().unwrap_or("?");
                let path = self
                    .command_path
                    .as_deref()
                    .map(|p| format!(" ({})", p))
                    .unwrap_or_default();
                format!(
                    " {} {} {}{}{} {}",
                    status_icon, icon, self.name, enabled_tag, path, version
                )
            }
            DiagnosticStatus::Warning => {
                let version = self.version.as_deref().unwrap_or("?");
                let path = self
                    .command_path
                    .as_deref()
                    .map(|p| format!(" ({})", p))
                    .unwrap_or_default();
                format!(
                    " {} {} {}{}{} {}",
                    status_icon, icon, self.name, enabled_tag, path, version
                )
            }
            DiagnosticStatus::Error => {
                let detail = self.error_detail.as_deref().unwrap_or("unknown error");
                format!(
                    " {} {} {}{} - {}",
                    status_icon, icon, self.name, enabled_tag, detail
                )
            }
        };

        if let Some(ref suggestion) = self.suggestion {
            line.push_str(&format!("\n      -> {}", suggestion));
        }

        line
    }
}

/// Summary of diagnostic results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    /// Config file path (if found)
    pub config_path: Option<String>,
    /// Whether config file exists
    pub config_exists: bool,
    /// Total tools checked
    pub total: usize,
    /// Tools with Ok status
    pub ok_count: usize,
    /// Tools with Warning status
    pub warning_count: usize,
    /// Tools with Error status
    pub error_count: usize,
    /// Individual tool diagnostics
    pub tools: Vec<ToolDiagnostic>,
}

impl DiagnosticSummary {
    /// Format the full diagnostic report
    pub fn format_display(&self) -> String {
        let mut lines = Vec::new();

        lines.push("Toolbox Doctor".to_string());
        lines.push("=".repeat(40));

        // Config info
        if let Some(ref path) = self.config_path {
            if self.config_exists {
                lines.push(format!(" Config: {}", path));
            } else {
                lines.push(format!(" Config: {} (not found, using defaults)", path));
            }
        } else {
            lines.push(" Config: (no config path available)".to_string());
        }

        lines.push(String::new());
        lines.push("Tool Status:".to_string());
        lines.push("-".repeat(40));

        for diag in &self.tools {
            lines.push(diag.format_display());
        }

        lines.push(String::new());
        lines.push("-".repeat(40));
        lines.push(format!(
            " {} tools checked: {} ok, {} warning, {} error",
            self.total, self.ok_count, self.warning_count, self.error_count
        ));

        lines.join("\n")
    }
}

/// System resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Memory usage percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_percent: Option<f32>,
    /// Total memory in GB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_total_gb: Option<f32>,
    /// Used memory in GB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_used_gb: Option<f32>,
    /// CPU usage percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_percent: Option<f32>,
}

impl ToolboxInfo {
    /// Create a new empty ToolboxInfo
    pub fn new() -> Self {
        Self {
            current_dir: None,
            git: None,
            tools: Vec::new(),
            system: None,
            virtual_env: None,
            shell: None,
        }
    }

    /// Format for display (simple text format)
    pub fn format_display(&self, compact: bool, show_icons: bool) -> String {
        let mut lines = Vec::new();
        let separator = "─".repeat(15);

        // Current directory
        if let Some(ref dir) = self.current_dir {
            let display_dir = if compact {
                shorten_path(dir)
            } else {
                dir.clone()
            };
            if show_icons {
                lines.push(format!(" 📂 {}", display_dir));
            } else {
                lines.push(format!(" {}", display_dir));
            }
        }

        // Git info
        if let Some(ref git) = self.git {
            let branch_display = if show_icons {
                format!(" 🌿 {}", git.branch)
            } else {
                format!(" {}", git.branch)
            };

            let mut suffixes = Vec::new();
            if let Some(summary) = git.changes_summary() {
                suffixes.push(summary);
            }
            if let Some(ab_summary) = git.ahead_behind_summary() {
                suffixes.push(ab_summary);
            }

            if !suffixes.is_empty() {
                lines.push(format!("{} ({})", branch_display, suffixes.join(" ")));
            } else {
                lines.push(branch_display);
            }
        }

        if !lines.is_empty() {
            lines.push(separator.clone());
        }

        // Tools
        for tool in &self.tools {
            if !tool.available {
                continue;
            }

            let name = if compact {
                tool.short_name.as_ref().unwrap_or(&tool.name)
            } else {
                &tool.name
            };

            let version = tool.version.as_deref().unwrap_or("?");

            if show_icons {
                let icon = tool.icon.as_deref().unwrap_or(" ");
                lines.push(format!(" {} {} {}", icon, name, version));
            } else {
                lines.push(format!(" {} {}", name, version));
            }
        }

        // Virtual env
        if let Some(ref venv) = self.virtual_env {
            if !lines.is_empty() && !self.tools.is_empty() {
                lines.push(separator.clone());
            }
            if show_icons {
                lines.push(format!(" 🐍 {}", venv));
            } else {
                lines.push(format!(" venv: {}", venv));
            }
        }

        // System info
        if let Some(ref sys) = self.system {
            if !lines.is_empty() {
                lines.push(separator);
            }
            if let Some(mem) = sys.memory_percent {
                if show_icons {
                    lines.push(format!(" 💾 {:.0}%", mem));
                } else {
                    lines.push(format!(" mem: {:.0}%", mem));
                }
            }
            if let Some(cpu) = sys.cpu_percent {
                if show_icons {
                    lines.push(format!(" 🔥 {:.0}%", cpu));
                } else {
                    lines.push(format!(" cpu: {:.0}%", cpu));
                }
            }
        }

        lines.join("\n")
    }

    /// Format for display as a powerline-style colored output
    /// If single_line is true, all segments are joined in one line
    /// If false, each segment is on its own line with colored background
    pub fn format_powerline(
        &self,
        compact: bool,
        show_icons: bool,
        use_color: bool,
        single_line: bool,
        theme: &crate::color::ResolvedTheme,
    ) -> String {
        use crate::color::{render_powerline, render_powerline_multiline, Segment};

        let mut segments = Vec::new();

        // Current directory
        if let Some(ref dir) = self.current_dir {
            let display_dir = if compact {
                shorten_path(dir)
            } else {
                dir.clone()
            };
            let text = if show_icons {
                format!("📂 {}", display_dir)
            } else {
                display_dir
            };
            segments.push(Segment::from_theme_colors(
                text,
                &theme.directory_fg,
                &theme.directory_bg,
            ));
        }

        // Git info
        if let Some(ref git) = self.git {
            let mut text = if show_icons {
                format!(" {}", git.branch)
            } else {
                git.branch.clone()
            };

            let mut suffixes = Vec::new();
            if let Some(summary) = git.changes_summary() {
                suffixes.push(summary);
            }
            if let Some(ab_summary) = git.ahead_behind_summary() {
                suffixes.push(ab_summary);
            }

            if !suffixes.is_empty() {
                text = format!("{} {}", text, suffixes.join(" "));
            }

            // Use clean/dirty colors from theme
            if git.is_dirty {
                segments.push(Segment::from_theme_colors(
                    text,
                    &theme.git_dirty_fg,
                    &theme.git_dirty_bg,
                ));
            } else {
                segments.push(Segment::from_theme_colors(
                    text,
                    &theme.git_clean_fg,
                    &theme.git_clean_bg,
                ));
            }
        }

        // Tools - group them or show individually
        let available_tools: Vec<_> = self.tools.iter().filter(|t| t.available).collect();

        for (i, tool) in available_tools.iter().enumerate() {
            let name = if compact {
                tool.short_name.as_ref().unwrap_or(&tool.name)
            } else {
                &tool.name
            };
            let version = tool.version.as_deref().unwrap_or("?");

            let text = if show_icons {
                let icon = tool.icon.as_deref().unwrap_or("");
                format!("{} {} {}", icon, name, version)
            } else {
                format!("{} {}", name, version)
            };

            let (ref bg, ref fg) = theme.tool_colors[i % theme.tool_colors.len()];
            segments.push(Segment::from_theme_colors(text, fg, bg));
        }

        // Virtual env
        if let Some(ref venv) = self.virtual_env {
            let text = if show_icons {
                format!("🐍 {}", venv)
            } else {
                format!("venv: {}", venv)
            };
            segments.push(Segment::from_theme_colors(
                text,
                &theme.venv_fg,
                &theme.venv_bg,
            ));
        }

        if single_line {
            render_powerline(&segments, use_color)
        } else {
            render_powerline_multiline(&segments, use_color)
        }
    }
}

impl Default for ToolboxInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Shorten a path for compact display
fn shorten_path(path: &str) -> String {
    // Replace home directory with ~
    if let Some(home) = dirs::home_dir() {
        if let Some(home_str) = home.to_str() {
            if path.starts_with(home_str) {
                return path.replacen(home_str, "~", 1);
            }
        }
    }

    // If path is too long, show only last 2 components
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() > 2 {
        format!("…/{}", parts[parts.len() - 2..].join("/"))
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ToolInfo tests
    #[test]
    fn test_tool_info_available() {
        let info = ToolInfo::available("Rust".to_string(), "1.75.0".to_string());
        assert_eq!(info.name, "Rust");
        assert_eq!(info.version, Some("1.75.0".to_string()));
        assert!(info.available);
        assert!(info.error.is_none());
        assert!(info.icon.is_none());
        assert!(info.short_name.is_none());
    }

    #[test]
    fn test_tool_info_unavailable() {
        let info = ToolInfo::unavailable("Ruby".to_string(), Some("not found".to_string()));
        assert_eq!(info.name, "Ruby");
        assert!(info.version.is_none());
        assert!(!info.available);
        assert_eq!(info.error, Some("not found".to_string()));
    }

    #[test]
    fn test_tool_info_with_icon() {
        let info = ToolInfo::available("Rust".to_string(), "1.75.0".to_string())
            .with_icon(Some("🦀".to_string()));
        assert_eq!(info.icon, Some("🦀".to_string()));
    }

    #[test]
    fn test_tool_info_with_short_name() {
        let info = ToolInfo::available("Python".to_string(), "3.11.0".to_string())
            .with_short_name(Some("py".to_string()));
        assert_eq!(info.short_name, Some("py".to_string()));
    }

    #[test]
    fn test_tool_info_chained_builders() {
        let info = ToolInfo::available("Node".to_string(), "20.0.0".to_string())
            .with_icon(Some("📦".to_string()))
            .with_short_name(Some("node".to_string()));
        assert_eq!(info.icon, Some("📦".to_string()));
        assert_eq!(info.short_name, Some("node".to_string()));
    }

    // GitInfo tests
    #[test]
    fn test_git_info_changes_summary_with_changes() {
        let git = GitInfo {
            branch: "main".to_string(),
            modified_count: Some(2),
            staged_count: Some(1),
            untracked_count: Some(3),
            is_dirty: true,
            ahead: None,
            behind: None,
        };
        assert_eq!(git.changes_summary(), Some("+6".to_string()));
    }

    #[test]
    fn test_git_info_changes_summary_no_changes() {
        let git = GitInfo {
            branch: "main".to_string(),
            modified_count: Some(0),
            staged_count: Some(0),
            untracked_count: Some(0),
            is_dirty: false,
            ahead: None,
            behind: None,
        };
        assert!(git.changes_summary().is_none());
    }

    #[test]
    fn test_git_info_changes_summary_none_counts() {
        let git = GitInfo {
            branch: "main".to_string(),
            modified_count: None,
            staged_count: None,
            untracked_count: None,
            is_dirty: false,
            ahead: None,
            behind: None,
        };
        assert!(git.changes_summary().is_none());
    }

    #[test]
    fn test_git_info_ahead_behind_summary_ahead_only() {
        let git = GitInfo {
            branch: "feature".to_string(),
            modified_count: None,
            staged_count: None,
            untracked_count: None,
            is_dirty: false,
            ahead: Some(3),
            behind: None,
        };
        assert_eq!(git.ahead_behind_summary(), Some("↑3".to_string()));
    }

    #[test]
    fn test_git_info_ahead_behind_summary_behind_only() {
        let git = GitInfo {
            branch: "feature".to_string(),
            modified_count: None,
            staged_count: None,
            untracked_count: None,
            is_dirty: false,
            ahead: None,
            behind: Some(2),
        };
        assert_eq!(git.ahead_behind_summary(), Some("↓2".to_string()));
    }

    #[test]
    fn test_git_info_ahead_behind_summary_both() {
        let git = GitInfo {
            branch: "feature".to_string(),
            modified_count: None,
            staged_count: None,
            untracked_count: None,
            is_dirty: false,
            ahead: Some(5),
            behind: Some(3),
        };
        assert_eq!(git.ahead_behind_summary(), Some("↑5 ↓3".to_string()));
    }

    #[test]
    fn test_git_info_ahead_behind_summary_none() {
        let git = GitInfo {
            branch: "main".to_string(),
            modified_count: None,
            staged_count: None,
            untracked_count: None,
            is_dirty: false,
            ahead: None,
            behind: None,
        };
        assert!(git.ahead_behind_summary().is_none());
    }

    // ToolboxInfo tests
    #[test]
    fn test_toolbox_info_new() {
        let info = ToolboxInfo::new();
        assert!(info.current_dir.is_none());
        assert!(info.git.is_none());
        assert!(info.tools.is_empty());
        assert!(info.system.is_none());
        assert!(info.virtual_env.is_none());
        assert!(info.shell.is_none());
    }

    #[test]
    fn test_toolbox_info_default() {
        let info = ToolboxInfo::default();
        assert!(info.tools.is_empty());
    }

    #[test]
    fn test_toolbox_info_format_display_empty() {
        let info = ToolboxInfo::new();
        let output = info.format_display(true, true);
        assert!(output.is_empty());
    }

    #[test]
    fn test_toolbox_info_format_display_with_tools() {
        let mut info = ToolboxInfo::new();
        info.tools.push(
            ToolInfo::available("Rust".to_string(), "1.75.0".to_string())
                .with_icon(Some("🦀".to_string()))
                .with_short_name(Some("rust".to_string())),
        );

        let output = info.format_display(true, true);
        assert!(output.contains("🦀"));
        assert!(output.contains("rust"));
        assert!(output.contains("1.75.0"));
    }

    #[test]
    fn test_toolbox_info_format_display_no_icons() {
        let mut info = ToolboxInfo::new();
        info.tools.push(
            ToolInfo::available("Rust".to_string(), "1.75.0".to_string())
                .with_icon(Some("🦀".to_string())),
        );

        let output = info.format_display(false, false);
        assert!(!output.contains("🦀"));
        assert!(output.contains("Rust"));
        assert!(output.contains("1.75.0"));
    }

    #[test]
    fn test_toolbox_info_format_display_with_git() {
        let mut info = ToolboxInfo::new();
        info.git = Some(GitInfo {
            branch: "main".to_string(),
            modified_count: Some(2),
            staged_count: None,
            untracked_count: None,
            is_dirty: true,
            ahead: None,
            behind: None,
        });

        let output = info.format_display(true, true);
        assert!(output.contains("main"));
        assert!(output.contains("+2"));
    }

    #[test]
    fn test_toolbox_info_format_display_unavailable_tools_hidden() {
        let mut info = ToolboxInfo::new();
        info.tools.push(ToolInfo::unavailable(
            "Ruby".to_string(),
            Some("not found".to_string()),
        ));

        let output = info.format_display(true, true);
        assert!(!output.contains("Ruby"));
    }

    // shorten_path tests
    #[test]
    fn test_shorten_path_long_path() {
        let path = "/very/long/path/to/project";
        let shortened = shorten_path(path);
        assert_eq!(shortened, "…/to/project");
    }

    #[test]
    fn test_shorten_path_short_path() {
        let path = "/short/path";
        let shortened = shorten_path(path);
        assert_eq!(shortened, "/short/path");
    }

    #[test]
    fn test_shorten_path_root() {
        let path = "/";
        let shortened = shorten_path(path);
        assert_eq!(shortened, "/");
    }

    // SystemInfo tests
    #[test]
    fn test_system_info_default() {
        let sys = SystemInfo {
            memory_percent: None,
            memory_total_gb: None,
            memory_used_gb: None,
            cpu_percent: None,
        };
        assert!(sys.memory_percent.is_none());
        assert!(sys.cpu_percent.is_none());
    }

    // Serialization tests
    #[test]
    fn test_tool_info_json_roundtrip() {
        let info = ToolInfo::available("Rust".to_string(), "1.75.0".to_string())
            .with_icon(Some("🦀".to_string()));
        let json = serde_json::to_string(&info).unwrap();
        let parsed: ToolInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, info.name);
        assert_eq!(parsed.version, info.version);
    }

    #[test]
    fn test_git_info_json_roundtrip() {
        let git = GitInfo {
            branch: "main".to_string(),
            modified_count: Some(1),
            staged_count: None,
            untracked_count: Some(2),
            is_dirty: true,
            ahead: Some(1),
            behind: None,
        };
        let json = serde_json::to_string(&git).unwrap();
        let parsed: GitInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.branch, git.branch);
        assert_eq!(parsed.modified_count, git.modified_count);
    }

    // --- format_display additional tests ---

    #[test]
    fn test_toolbox_info_format_display_with_virtual_env() {
        let mut info = ToolboxInfo::new();
        info.virtual_env = Some("myenv".to_string());
        info.tools.push(ToolInfo::available(
            "Python".to_string(),
            "3.12.0".to_string(),
        ));

        let output = info.format_display(false, true);
        assert!(output.contains("myenv"));
    }

    #[test]
    fn test_toolbox_info_format_display_virtual_env_no_icons() {
        let mut info = ToolboxInfo::new();
        info.virtual_env = Some("myenv".to_string());
        info.tools.push(ToolInfo::available(
            "Python".to_string(),
            "3.12.0".to_string(),
        ));

        let output = info.format_display(false, false);
        assert!(output.contains("venv: myenv"));
    }

    #[test]
    fn test_toolbox_info_format_display_with_system_info() {
        let mut info = ToolboxInfo::new();
        info.system = Some(SystemInfo {
            memory_percent: Some(50.0),
            memory_total_gb: Some(16.0),
            memory_used_gb: Some(8.0),
            cpu_percent: Some(25.0),
        });

        let output = info.format_display(false, true);
        assert!(output.contains("50%"));
        assert!(output.contains("25%"));
    }

    #[test]
    fn test_toolbox_info_format_display_system_no_icons() {
        let mut info = ToolboxInfo::new();
        info.system = Some(SystemInfo {
            memory_percent: Some(75.0),
            memory_total_gb: None,
            memory_used_gb: None,
            cpu_percent: Some(50.0),
        });

        let output = info.format_display(false, false);
        assert!(output.contains("mem: 75%"));
        assert!(output.contains("cpu: 50%"));
    }

    #[test]
    fn test_toolbox_info_format_display_with_current_dir() {
        let mut info = ToolboxInfo::new();
        info.current_dir = Some("/home/user/project".to_string());

        let output = info.format_display(false, true);
        assert!(output.contains("/home/user/project"));
    }

    #[test]
    fn test_toolbox_info_format_display_compact_dir() {
        let mut info = ToolboxInfo::new();
        info.current_dir = Some("/very/long/path/to/project".to_string());

        let output = info.format_display(true, false);
        assert!(output.contains("to/project"));
    }

    #[test]
    fn test_toolbox_info_format_display_git_clean() {
        let mut info = ToolboxInfo::new();
        info.git = Some(GitInfo {
            branch: "main".to_string(),
            modified_count: Some(0),
            staged_count: Some(0),
            untracked_count: Some(0),
            is_dirty: false,
            ahead: None,
            behind: None,
        });

        let output = info.format_display(false, true);
        assert!(output.contains("main"));
        // Clean repo should not show change count
        assert!(!output.contains("+"));
    }

    #[test]
    fn test_toolbox_info_format_display_git_ahead_behind() {
        let mut info = ToolboxInfo::new();
        info.git = Some(GitInfo {
            branch: "feature".to_string(),
            modified_count: Some(0),
            staged_count: Some(0),
            untracked_count: Some(0),
            is_dirty: false,
            ahead: Some(3),
            behind: Some(1),
        });

        let output = info.format_display(false, true);
        assert!(output.contains("feature"));
        assert!(output.contains("\u{2191}3")); // ↑3
        assert!(output.contains("\u{2193}1")); // ↓1
    }

    #[test]
    fn test_toolbox_info_format_display_separator_between_sections() {
        let mut info = ToolboxInfo::new();
        info.current_dir = Some("/tmp".to_string());
        info.tools.push(ToolInfo::available(
            "Rust".to_string(),
            "1.75.0".to_string(),
        ));

        let output = info.format_display(false, true);
        assert!(output.contains("\u{2500}")); // ─ separator
    }

    // --- format_powerline tests ---

    #[test]
    fn test_toolbox_info_format_powerline_empty() {
        let info = ToolboxInfo::new();
        let output = info.format_powerline(
            false,
            true,
            false,
            true,
            &crate::color::ResolvedTheme::default_theme(),
        );
        assert!(output.is_empty());
    }

    #[test]
    fn test_toolbox_info_format_powerline_with_tools_no_color() {
        let mut info = ToolboxInfo::new();
        info.tools.push(
            ToolInfo::available("Rust".to_string(), "1.75.0".to_string())
                .with_icon(Some("\u{1f980}".to_string())),
        );

        let output = info.format_powerline(
            false,
            true,
            false,
            true,
            &crate::color::ResolvedTheme::default_theme(),
        );
        assert!(output.contains("Rust"));
        assert!(output.contains("1.75.0"));
    }

    #[test]
    fn test_toolbox_info_format_powerline_with_color() {
        let mut info = ToolboxInfo::new();
        info.tools.push(ToolInfo::available(
            "Rust".to_string(),
            "1.75.0".to_string(),
        ));

        let output = info.format_powerline(
            false,
            false,
            true,
            true,
            &crate::color::ResolvedTheme::default_theme(),
        );
        assert!(output.contains("\x1b[")); // ANSI codes
        assert!(output.contains("Rust"));
    }

    #[test]
    fn test_toolbox_info_format_powerline_multiline() {
        let mut info = ToolboxInfo::new();
        info.tools
            .push(ToolInfo::available("A".to_string(), "1.0".to_string()));
        info.tools
            .push(ToolInfo::available("B".to_string(), "2.0".to_string()));

        let output = info.format_powerline(
            false,
            false,
            true,
            false,
            &crate::color::ResolvedTheme::default_theme(),
        );
        assert!(output.contains('\n'));
    }

    #[test]
    fn test_toolbox_info_format_powerline_git_clean() {
        let mut info = ToolboxInfo::new();
        info.git = Some(GitInfo {
            branch: "main".to_string(),
            modified_count: Some(0),
            staged_count: Some(0),
            untracked_count: Some(0),
            is_dirty: false,
            ahead: None,
            behind: None,
        });

        // Green segment for clean repo (no color for easy assertion)
        let output = info.format_powerline(
            false,
            false,
            false,
            true,
            &crate::color::ResolvedTheme::default_theme(),
        );
        assert!(output.contains("main"));
    }

    #[test]
    fn test_toolbox_info_format_powerline_git_dirty() {
        let mut info = ToolboxInfo::new();
        info.git = Some(GitInfo {
            branch: "dev".to_string(),
            modified_count: Some(3),
            staged_count: Some(0),
            untracked_count: Some(0),
            is_dirty: true,
            ahead: None,
            behind: None,
        });

        let output = info.format_powerline(
            false,
            false,
            false,
            true,
            &crate::color::ResolvedTheme::default_theme(),
        );
        assert!(output.contains("dev"));
        assert!(output.contains("+3"));
    }

    #[test]
    fn test_toolbox_info_format_powerline_compact() {
        let mut info = ToolboxInfo::new();
        info.tools.push(
            ToolInfo::available("Python".to_string(), "3.12.0".to_string())
                .with_short_name(Some("py".to_string())),
        );

        let output = info.format_powerline(
            true,
            false,
            false,
            true,
            &crate::color::ResolvedTheme::default_theme(),
        );
        assert!(output.contains("py"));
        assert!(!output.contains("Python"));
    }

    #[test]
    fn test_toolbox_info_format_powerline_virtual_env() {
        let mut info = ToolboxInfo::new();
        info.virtual_env = Some("myenv".to_string());

        let output = info.format_powerline(
            false,
            false,
            false,
            true,
            &crate::color::ResolvedTheme::default_theme(),
        );
        assert!(output.contains("venv: myenv"));
    }

    // --- ToolboxInfo JSON roundtrip ---

    #[test]
    fn test_toolbox_info_json_roundtrip() {
        let mut info = ToolboxInfo::new();
        info.current_dir = Some("/tmp".to_string());
        info.tools.push(ToolInfo::available(
            "Rust".to_string(),
            "1.75.0".to_string(),
        ));
        info.git = Some(GitInfo {
            branch: "main".to_string(),
            modified_count: Some(0),
            staged_count: Some(0),
            untracked_count: Some(0),
            is_dirty: false,
            ahead: None,
            behind: None,
        });

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ToolboxInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.current_dir, info.current_dir);
        assert_eq!(parsed.tools.len(), 1);
        assert!(parsed.git.is_some());
    }

    // --- SystemInfo JSON roundtrip ---

    #[test]
    fn test_system_info_json_roundtrip() {
        let sys = SystemInfo {
            memory_percent: Some(65.5),
            memory_total_gb: Some(16.0),
            memory_used_gb: Some(10.48),
            cpu_percent: Some(42.0),
        };
        let json = serde_json::to_string(&sys).unwrap();
        let parsed: SystemInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.memory_percent, sys.memory_percent);
        assert_eq!(parsed.cpu_percent, sys.cpu_percent);
    }

    // --- Multiple available and unavailable tools ---

    #[test]
    fn test_format_display_mixed_tools() {
        let mut info = ToolboxInfo::new();
        info.tools.push(ToolInfo::available(
            "Rust".to_string(),
            "1.75.0".to_string(),
        ));
        info.tools.push(ToolInfo::unavailable(
            "Ruby".to_string(),
            Some("not found".to_string()),
        ));
        info.tools
            .push(ToolInfo::available("Go".to_string(), "1.21.0".to_string()));

        let output = info.format_display(false, false);
        assert!(output.contains("Rust"));
        assert!(output.contains("Go"));
        assert!(!output.contains("Ruby")); // Unavailable hidden
    }

    // --- DiagnosticStatus tests ---

    #[test]
    fn test_diagnostic_status_equality() {
        assert_eq!(DiagnosticStatus::Ok, DiagnosticStatus::Ok);
        assert_eq!(DiagnosticStatus::Warning, DiagnosticStatus::Warning);
        assert_eq!(DiagnosticStatus::Error, DiagnosticStatus::Error);
        assert_ne!(DiagnosticStatus::Ok, DiagnosticStatus::Error);
    }

    // --- ToolDiagnostic format_display tests ---

    #[test]
    fn test_diagnostic_format_ok() {
        let diag = ToolDiagnostic {
            name: "Rust".to_string(),
            icon: Some("R".to_string()),
            status: DiagnosticStatus::Ok,
            command: "rustc --version".to_string(),
            command_path: Some("/usr/bin/rustc".to_string()),
            version: Some("1.75.0".to_string()),
            error_detail: None,
            suggestion: None,
            enabled: true,
        };

        let output = diag.format_display();
        assert!(output.contains("OK"));
        assert!(output.contains("Rust"));
        assert!(output.contains("1.75.0"));
        assert!(output.contains("/usr/bin/rustc"));
        assert!(!output.contains("disabled"));
    }

    #[test]
    fn test_diagnostic_format_ok_disabled() {
        let diag = ToolDiagnostic {
            name: "Ruby".to_string(),
            icon: None,
            status: DiagnosticStatus::Ok,
            command: "ruby --version".to_string(),
            command_path: Some("/usr/bin/ruby".to_string()),
            version: Some("3.2.0".to_string()),
            error_detail: None,
            suggestion: None,
            enabled: false,
        };

        let output = diag.format_display();
        assert!(output.contains("(disabled)"));
    }

    #[test]
    fn test_diagnostic_format_warning() {
        let diag = ToolDiagnostic {
            name: "Java".to_string(),
            icon: Some("J".to_string()),
            status: DiagnosticStatus::Warning,
            command: "java --version".to_string(),
            command_path: Some("/usr/bin/java".to_string()),
            version: Some("java 21.0.1 2023-10-17".to_string()),
            error_detail: Some("regex did not match".to_string()),
            suggestion: Some("Check parse_regex".to_string()),
            enabled: true,
        };

        let output = diag.format_display();
        assert!(output.contains("WARN"));
        assert!(output.contains("Java"));
        assert!(output.contains("-> Check parse_regex"));
    }

    #[test]
    fn test_diagnostic_format_error() {
        let diag = ToolDiagnostic {
            name: "Docker".to_string(),
            icon: Some("D".to_string()),
            status: DiagnosticStatus::Error,
            command: "docker --version".to_string(),
            command_path: None,
            version: None,
            error_detail: Some("command not found: 'docker'".to_string()),
            suggestion: Some("Install Docker or add it to your PATH".to_string()),
            enabled: true,
        };

        let output = diag.format_display();
        assert!(output.contains("ERR"));
        assert!(output.contains("Docker"));
        assert!(output.contains("command not found"));
        assert!(output.contains("-> Install Docker"));
    }

    // --- DiagnosticSummary format_display tests ---

    #[test]
    fn test_diagnostic_summary_format_empty() {
        let summary = DiagnosticSummary {
            config_path: Some("/home/user/.config/toolbox/config.toml".to_string()),
            config_exists: false,
            total: 0,
            ok_count: 0,
            warning_count: 0,
            error_count: 0,
            tools: vec![],
        };

        let output = summary.format_display();
        assert!(output.contains("Toolbox Doctor"));
        assert!(output.contains("not found, using defaults"));
        assert!(output.contains("0 tools checked"));
    }

    #[test]
    fn test_diagnostic_summary_format_with_tools() {
        let summary = DiagnosticSummary {
            config_path: Some("/home/user/.config/toolbox/config.toml".to_string()),
            config_exists: true,
            total: 3,
            ok_count: 2,
            warning_count: 0,
            error_count: 1,
            tools: vec![
                ToolDiagnostic {
                    name: "Rust".to_string(),
                    icon: None,
                    status: DiagnosticStatus::Ok,
                    command: "rustc --version".to_string(),
                    command_path: Some("/usr/bin/rustc".to_string()),
                    version: Some("1.75.0".to_string()),
                    error_detail: None,
                    suggestion: None,
                    enabled: true,
                },
                ToolDiagnostic {
                    name: "Python".to_string(),
                    icon: None,
                    status: DiagnosticStatus::Ok,
                    command: "python3 --version".to_string(),
                    command_path: Some("/usr/bin/python3".to_string()),
                    version: Some("3.12.0".to_string()),
                    error_detail: None,
                    suggestion: None,
                    enabled: true,
                },
                ToolDiagnostic {
                    name: "Docker".to_string(),
                    icon: None,
                    status: DiagnosticStatus::Error,
                    command: "docker --version".to_string(),
                    command_path: None,
                    version: None,
                    error_detail: Some("not found".to_string()),
                    suggestion: None,
                    enabled: true,
                },
            ],
        };

        let output = summary.format_display();
        assert!(output.contains("Toolbox Doctor"));
        assert!(output.contains("Config:"));
        assert!(!output.contains("not found, using defaults"));
        assert!(output.contains("3 tools checked: 2 ok, 0 warning, 1 error"));
    }

    // --- DiagnosticSummary JSON roundtrip ---

    #[test]
    fn test_diagnostic_summary_json_roundtrip() {
        let summary = DiagnosticSummary {
            config_path: Some("/tmp/config.toml".to_string()),
            config_exists: true,
            total: 1,
            ok_count: 1,
            warning_count: 0,
            error_count: 0,
            tools: vec![ToolDiagnostic {
                name: "Echo".to_string(),
                icon: None,
                status: DiagnosticStatus::Ok,
                command: "echo test".to_string(),
                command_path: Some("/bin/echo".to_string()),
                version: Some("test".to_string()),
                error_detail: None,
                suggestion: None,
                enabled: true,
            }],
        };

        let json = serde_json::to_string(&summary).unwrap();
        let parsed: DiagnosticSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total, 1);
        assert_eq!(parsed.ok_count, 1);
        assert_eq!(parsed.tools.len(), 1);
        assert_eq!(parsed.tools[0].status, DiagnosticStatus::Ok);
    }
}
