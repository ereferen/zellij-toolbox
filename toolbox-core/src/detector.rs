//! Tool version detection

use crate::config::{Config, ToolConfig};
use crate::error::{Result, ToolboxError};
use crate::info::{GitInfo, SystemInfo, ToolInfo, ToolboxInfo};
use regex::Regex;
use std::path::Path;
use std::process::Command;

/// Main detector for tool versions and system info
pub struct ToolDetector {
    config: Config,
    /// Working directory for command execution
    working_dir: Option<String>,
}

impl ToolDetector {
    /// Create a new detector with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            working_dir: None,
        }
    }

    /// Create a detector with default configuration
    pub fn with_defaults() -> Self {
        Self::new(Config::default())
    }

    /// Set the working directory for command execution
    /// This is important for asdf/mise which have directory-specific versions
    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Detect all enabled tools and gather information
    pub fn detect_all(&self) -> ToolboxInfo {
        let mut info = ToolboxInfo::new();

        // Current directory
        if self.config.extras.current_directory {
            info.current_dir = self.get_current_dir();
        }

        // Git info
        if self.config.extras.git_branch || self.config.extras.git_status {
            info.git = self.get_git_info();
        }

        // Tool versions
        for tool_config in self.config.enabled_tools() {
            let tool_info = self.detect_tool(tool_config);
            info.tools.push(tool_info);
        }

        // Virtual environment
        if self.config.extras.virtual_env {
            info.virtual_env = self.get_virtual_env();
        }

        // Shell
        if self.config.extras.shell {
            info.shell = self.get_shell();
        }

        // System info
        #[cfg(feature = "sysinfo")]
        if self.config.extras.system_memory || self.config.extras.system_cpu {
            info.system = self.get_system_info();
        }

        info
    }

    /// Detect a single tool's version
    pub fn detect_tool(&self, tool_config: &ToolConfig) -> ToolInfo {
        match self.run_version_command(&tool_config.command) {
            Ok(output) => {
                let version = if let Some(ref regex_str) = tool_config.parse_regex {
                    self.parse_version(&output, regex_str)
                        .unwrap_or_else(|| output.trim().to_string())
                } else {
                    output.trim().to_string()
                };

                ToolInfo::available(tool_config.name.clone(), version)
                    .with_icon(tool_config.icon.clone())
                    .with_short_name(tool_config.short_name.clone())
            }
            Err(e) => ToolInfo::unavailable(tool_config.name.clone(), Some(e.to_string()))
                .with_icon(tool_config.icon.clone())
                .with_short_name(tool_config.short_name.clone()),
        }
    }

    /// Run a command and get its output
    fn run_version_command(&self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ToolboxError::CommandFailed("Empty command".to_string()));
        }

        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }

        // Set working directory if specified
        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        // Inherit PATH and other environment variables for asdf/mise support
        let output = cmd.output().map_err(|e| {
            ToolboxError::CommandFailed(format!("{}: {}", parts[0], e))
        })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            // Some tools output to stderr
            if stdout.trim().is_empty() {
                Ok(String::from_utf8_lossy(&output.stderr).to_string())
            } else {
                Ok(stdout)
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ToolboxError::CommandFailed(format!(
                "{}: {}",
                parts[0],
                stderr.trim()
            )))
        }
    }

    /// Parse version from output using regex
    fn parse_version(&self, output: &str, regex_str: &str) -> Option<String> {
        let re = Regex::new(regex_str).ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }

    /// Get current working directory
    fn get_current_dir(&self) -> Option<String> {
        if let Some(ref dir) = self.working_dir {
            Some(dir.clone())
        } else {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
        }
    }

    /// Get git repository information
    #[cfg(feature = "git")]
    fn get_git_info(&self) -> Option<GitInfo> {
        let dir = self.working_dir.as_deref().unwrap_or(".");
        let repo = git2::Repository::discover(dir).ok()?;

        // Get current branch
        let head = repo.head().ok()?;
        let branch = if head.is_branch() {
            head.shorthand().unwrap_or("HEAD").to_string()
        } else {
            // Detached HEAD - show short commit hash
            head.target()
                .map(|oid| oid.to_string()[..7].to_string())
                .unwrap_or_else(|| "HEAD".to_string())
        };

        // Get status
        let mut modified_count = 0;
        let mut staged_count = 0;
        let mut untracked_count = 0;

        if self.config.extras.git_status {
            if let Ok(statuses) = repo.statuses(None) {
                for entry in statuses.iter() {
                    let status = entry.status();
                    if status.is_wt_modified() || status.is_wt_deleted() || status.is_wt_renamed() {
                        modified_count += 1;
                    }
                    if status.is_index_new()
                        || status.is_index_modified()
                        || status.is_index_deleted()
                        || status.is_index_renamed()
                    {
                        staged_count += 1;
                    }
                    if status.is_wt_new() {
                        untracked_count += 1;
                    }
                }
            }
        }

        let is_dirty = modified_count > 0 || staged_count > 0 || untracked_count > 0;

        Some(GitInfo {
            branch,
            modified_count: if self.config.extras.git_status {
                Some(modified_count)
            } else {
                None
            },
            staged_count: if self.config.extras.git_status {
                Some(staged_count)
            } else {
                None
            },
            untracked_count: if self.config.extras.git_status {
                Some(untracked_count)
            } else {
                None
            },
            is_dirty,
            ahead: None,  // TODO: implement
            behind: None, // TODO: implement
        })
    }

    #[cfg(not(feature = "git"))]
    fn get_git_info(&self) -> Option<GitInfo> {
        None
    }

    /// Get virtual environment name
    fn get_virtual_env(&self) -> Option<String> {
        // Check VIRTUAL_ENV for Python venv
        if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
            return Path::new(&venv)
                .file_name()
                .and_then(|n| n.to_str())
                .map(String::from);
        }

        // Check CONDA_DEFAULT_ENV for Conda
        if let Ok(conda) = std::env::var("CONDA_DEFAULT_ENV") {
            return Some(conda);
        }

        None
    }

    /// Get current shell name
    fn get_shell(&self) -> Option<String> {
        std::env::var("SHELL")
            .ok()
            .and_then(|s| Path::new(&s).file_name().map(|n| n.to_str().map(String::from)))
            .flatten()
    }

    /// Get system resource information
    #[cfg(feature = "sysinfo")]
    fn get_system_info(&self) -> Option<SystemInfo> {
        use sysinfo::System;

        let mut sys = System::new();

        let mut info = SystemInfo {
            memory_percent: None,
            memory_total_gb: None,
            memory_used_gb: None,
            cpu_percent: None,
        };

        if self.config.extras.system_memory {
            sys.refresh_memory();
            let total = sys.total_memory() as f32 / 1_073_741_824.0; // bytes to GB
            let used = sys.used_memory() as f32 / 1_073_741_824.0;
            info.memory_total_gb = Some(total);
            info.memory_used_gb = Some(used);
            info.memory_percent = Some((used / total) * 100.0);
        }

        if self.config.extras.system_cpu {
            sys.refresh_cpu_usage();
            // Need to wait a bit for accurate CPU readings
            std::thread::sleep(std::time::Duration::from_millis(100));
            sys.refresh_cpu_usage();
            let cpu_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>()
                / sys.cpus().len() as f32;
            info.cpu_percent = Some(cpu_usage);
        }

        Some(info)
    }

    #[cfg(not(feature = "sysinfo"))]
    fn get_system_info(&self) -> Option<SystemInfo> {
        None
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}
