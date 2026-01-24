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
        format!("…/{}", parts[parts.len()-2..].join("/"))
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
        info.tools.push(ToolInfo::available("Rust".to_string(), "1.75.0".to_string())
            .with_icon(Some("🦀".to_string()))
            .with_short_name(Some("rust".to_string())));

        let output = info.format_display(true, true);
        assert!(output.contains("🦀"));
        assert!(output.contains("rust"));
        assert!(output.contains("1.75.0"));
    }

    #[test]
    fn test_toolbox_info_format_display_no_icons() {
        let mut info = ToolboxInfo::new();
        info.tools.push(ToolInfo::available("Rust".to_string(), "1.75.0".to_string())
            .with_icon(Some("🦀".to_string())));

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
        info.tools.push(ToolInfo::unavailable("Ruby".to_string(), Some("not found".to_string())));

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
}
