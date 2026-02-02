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
        for tool_config in &self.config.enabled_tools() {
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
        let output = cmd
            .output()
            .map_err(|e| ToolboxError::CommandFailed(format!("{}: {}", parts[0], e)))?;

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

        // Get ahead/behind counts
        let (ahead, behind) = if head.is_branch() {
            Self::get_ahead_behind(&repo, &head).unwrap_or((None, None))
        } else {
            (None, None)
        };

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
            ahead,
            behind,
        })
    }

    /// Get ahead/behind counts relative to upstream
    #[cfg(feature = "git")]
    fn get_ahead_behind(
        repo: &git2::Repository,
        head: &git2::Reference,
    ) -> Option<(Option<usize>, Option<usize>)> {
        // Get the local OID
        let local_oid = head.target()?;

        // Get the branch name and create a Branch object
        let branch_name = head.shorthand()?;
        let branch = repo
            .find_branch(branch_name, git2::BranchType::Local)
            .ok()?;

        // Get the upstream branch
        let upstream = branch.upstream().ok()?;
        let upstream_oid = upstream.get().target()?;

        // Calculate ahead/behind
        let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid).ok()?;

        Some((
            if ahead > 0 { Some(ahead) } else { None },
            if behind > 0 { Some(behind) } else { None },
        ))
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
            .and_then(|s| {
                Path::new(&s)
                    .file_name()
                    .map(|n| n.to_str().map(String::from))
            })
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
            let cpu_usage: f32 =
                sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ToolConfig;

    // Helper to create a simple ToolDetector for testing
    fn test_detector() -> ToolDetector {
        ToolDetector::with_defaults()
    }

    // parse_version tests
    #[test]
    fn test_parse_version_python() {
        let detector = test_detector();
        let output = "Python 3.11.4";
        let regex = r"Python\s+(\d+\.\d+(?:\.\d+)?)";
        let version = detector.parse_version(output, regex);
        assert_eq!(version, Some("3.11.4".to_string()));
    }

    #[test]
    fn test_parse_version_node() {
        let detector = test_detector();
        let output = "v20.10.0";
        let regex = r"v?(\d+\.\d+(?:\.\d+)?)";
        let version = detector.parse_version(output, regex);
        assert_eq!(version, Some("20.10.0".to_string()));
    }

    #[test]
    fn test_parse_version_rustc() {
        let detector = test_detector();
        let output = "rustc 1.75.0 (82e1608df 2023-12-21)";
        let regex = r"rustc\s+(\d+\.\d+(?:\.\d+)?)";
        let version = detector.parse_version(output, regex);
        assert_eq!(version, Some("1.75.0".to_string()));
    }

    #[test]
    fn test_parse_version_go() {
        let detector = test_detector();
        let output = "go version go1.21.5 linux/amd64";
        let regex = r"go(\d+\.\d+(?:\.\d+)?)";
        let version = detector.parse_version(output, regex);
        assert_eq!(version, Some("1.21.5".to_string()));
    }

    #[test]
    fn test_parse_version_docker() {
        let detector = test_detector();
        let output = "Docker version 24.0.7, build afdd53b";
        let regex = r"Docker version\s+(\d+\.\d+(?:\.\d+)?)";
        let version = detector.parse_version(output, regex);
        assert_eq!(version, Some("24.0.7".to_string()));
    }

    #[test]
    fn test_parse_version_ruby() {
        let detector = test_detector();
        let output = "ruby 3.2.2 (2023-03-30 revision e51014f9c0) [x86_64-linux]";
        let regex = r"ruby\s+(\d+\.\d+(?:\.\d+)?)";
        let version = detector.parse_version(output, regex);
        assert_eq!(version, Some("3.2.2".to_string()));
    }

    #[test]
    fn test_parse_version_java() {
        let detector = test_detector();
        let output = "openjdk 17.0.9 2023-10-17";
        let regex = r"(\d+\.\d+(?:\.\d+)?)";
        let version = detector.parse_version(output, regex);
        assert_eq!(version, Some("17.0.9".to_string()));
    }

    #[test]
    fn test_parse_version_no_match() {
        let detector = test_detector();
        let output = "some random output";
        let regex = r"Python\s+(\d+\.\d+)";
        let version = detector.parse_version(output, regex);
        assert!(version.is_none());
    }

    #[test]
    fn test_parse_version_invalid_regex() {
        let detector = test_detector();
        let output = "Python 3.11.4";
        let regex = r"[invalid(regex";
        let version = detector.parse_version(output, regex);
        assert!(version.is_none());
    }

    #[test]
    fn test_parse_version_two_digit() {
        let detector = test_detector();
        let output = "v21.5";
        let regex = r"v?(\d+\.\d+(?:\.\d+)?)";
        let version = detector.parse_version(output, regex);
        assert_eq!(version, Some("21.5".to_string()));
    }

    #[test]
    fn test_parse_version_multiline() {
        let detector = test_detector();
        let output =
            "deno 1.38.5 (release, x86_64-unknown-linux-gnu)\nv8 12.0.267.1\ntypescript 5.2.2";
        let regex = r"deno\s+(\d+\.\d+(?:\.\d+)?)";
        let version = detector.parse_version(output, regex);
        assert_eq!(version, Some("1.38.5".to_string()));
    }

    // ToolDetector construction tests
    #[test]
    fn test_detector_with_defaults() {
        let detector = ToolDetector::with_defaults();
        assert!(!detector.config().effective_tools().is_empty());
        assert!(detector.working_dir.is_none());
    }

    #[test]
    fn test_detector_with_working_dir() {
        let detector = ToolDetector::with_defaults().with_working_dir("/tmp/test".to_string());
        assert_eq!(detector.working_dir, Some("/tmp/test".to_string()));
    }

    #[test]
    fn test_detector_new_with_config() {
        let config = Config::default();
        let detector = ToolDetector::new(config);
        assert!(!detector.config().effective_tools().is_empty());
    }

    // detect_tool tests
    #[test]
    fn test_detect_tool_unavailable() {
        let detector = test_detector();
        let tool_config = ToolConfig {
            name: "NonExistent".to_string(),
            command: "nonexistent_command_12345 --version".to_string(),
            parse_regex: None,
            icon: Some("❓".to_string()),
            enabled: true,
            short_name: None,
        };

        let info = detector.detect_tool(&tool_config);
        assert!(!info.available);
        assert!(info.error.is_some());
        assert_eq!(info.name, "NonExistent");
    }

    #[test]
    fn test_detect_tool_with_echo() {
        let detector = test_detector();
        let tool_config = ToolConfig {
            name: "Echo".to_string(),
            command: "echo v1.2.3".to_string(),
            parse_regex: Some(r"v?(\d+\.\d+\.\d+)".to_string()),
            icon: None,
            enabled: true,
            short_name: Some("echo".to_string()),
        };

        let info = detector.detect_tool(&tool_config);
        assert!(info.available);
        assert_eq!(info.version, Some("1.2.3".to_string()));
        assert_eq!(info.short_name, Some("echo".to_string()));
    }

    #[test]
    fn test_detect_tool_no_regex() {
        let detector = test_detector();
        let tool_config = ToolConfig {
            name: "Raw".to_string(),
            command: "echo hello world".to_string(),
            parse_regex: None,
            icon: None,
            enabled: true,
            short_name: None,
        };

        let info = detector.detect_tool(&tool_config);
        assert!(info.available);
        assert_eq!(info.version, Some("hello world".to_string()));
    }

    // run_version_command tests
    #[test]
    fn test_run_version_command_empty() {
        let detector = test_detector();
        let result = detector.run_version_command("");
        assert!(result.is_err());
    }

    #[test]
    fn test_run_version_command_success() {
        let detector = test_detector();
        let result = detector.run_version_command("echo test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "test");
    }

    #[test]
    fn test_run_version_command_not_found() {
        let detector = test_detector();
        let result = detector.run_version_command("nonexistent_cmd_xyz --version");
        assert!(result.is_err());
    }

    // Environment variable tests
    #[test]
    fn test_get_virtual_env_none() {
        // This test assumes VIRTUAL_ENV and CONDA_DEFAULT_ENV are not set
        let detector = test_detector();
        // We can't reliably test this without modifying env vars
        // Just verify the method doesn't panic
        let _ = detector.get_virtual_env();
    }

    #[test]
    fn test_get_shell() {
        let detector = test_detector();
        // On most systems, SHELL should be set
        let shell = detector.get_shell();
        // Just verify it doesn't panic and returns a reasonable value if set
        if let Some(s) = shell {
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn test_get_current_dir() {
        let detector = test_detector();
        let dir = detector.get_current_dir();
        assert!(dir.is_some());
    }

    #[test]
    fn test_get_current_dir_with_working_dir() {
        let detector = ToolDetector::with_defaults().with_working_dir("/tmp".to_string());
        let dir = detector.get_current_dir();
        assert_eq!(dir, Some("/tmp".to_string()));
    }

    // detect_all basic test
    #[test]
    fn test_detect_all_returns_toolbox_info() {
        // Create a minimal config to speed up the test
        let config = Config {
            use_default_tools: false,
            extras: crate::config::ExtrasConfig {
                git_branch: false,
                git_status: false,
                system_memory: false,
                system_cpu: false,
                ..Default::default()
            },
            ..Config::default()
        };

        let detector = ToolDetector::new(config);
        let info = detector.detect_all();

        // Should return ToolboxInfo even with no tools
        assert!(info.tools.is_empty());
    }
}
