//! Tool version detection

use crate::cache::VersionCache;
use crate::config::{Config, ToolConfig};
use crate::error::{Result, ToolboxError};
use crate::info::{
    DiagnosticStatus, DiagnosticSummary, GitInfo, SystemInfo, ToolDiagnostic, ToolInfo, ToolboxInfo,
};
use regex::Regex;
use std::path::Path;
use std::process::Command;

/// Main detector for tool versions and system info
pub struct ToolDetector {
    config: Config,
    /// Working directory for command execution
    working_dir: Option<String>,
    /// Version cache for avoiding redundant detections
    cache: Option<VersionCache>,
}

impl ToolDetector {
    /// Create a new detector with the given configuration
    pub fn new(config: Config) -> Self {
        let cache = if config.cache.enabled {
            Some(VersionCache::new(config.cache.default_ttl))
        } else {
            None
        };
        Self {
            config,
            working_dir: None,
            cache,
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

    /// Disable the cache (equivalent to --no-cache)
    pub fn with_cache_disabled(mut self) -> Self {
        self.cache = None;
        self
    }

    /// Force refresh: clear existing cache entries but keep cache enabled
    pub fn with_cache_refresh(mut self) -> Self {
        if let Some(ref mut cache) = self.cache {
            cache.clear();
        }
        self
    }

    /// Get a reference to the cache (if enabled)
    pub fn cache(&self) -> Option<&VersionCache> {
        self.cache.as_ref()
    }

    /// Detect all enabled tools and gather information
    pub fn detect_all(&mut self) -> ToolboxInfo {
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
        let enabled_tools = self.config.enabled_tools();
        for tool_config in &enabled_tools {
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

    /// Detect a single tool's version, using cache if available
    pub fn detect_tool(&mut self, tool_config: &ToolConfig) -> ToolInfo {
        // Try cache first
        if let Some(ref mut cache) = self.cache {
            if let Some(cached) = cache.get(&tool_config.name, &self.working_dir) {
                return cached.clone();
            }
        }

        // Cache miss or disabled — run detection
        let tool_info = self.detect_tool_uncached(tool_config);

        // Store in cache
        if let Some(ref mut cache) = self.cache {
            cache.put(
                tool_config.name.clone(),
                tool_info.clone(),
                self.working_dir.clone(),
            );
        }

        tool_info
    }

    /// Detect a single tool's version without cache
    fn detect_tool_uncached(&self, tool_config: &ToolConfig) -> ToolInfo {
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
    #[allow(dead_code)]
    fn get_system_info(&self) -> Option<SystemInfo> {
        None
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Run diagnostics on a single tool, returning detailed results
    pub fn diagnose_tool(&self, tool_config: &ToolConfig) -> ToolDiagnostic {
        let cmd_name = tool_config.command.split_whitespace().next().unwrap_or("");

        // Try to find the command in PATH
        let command_path = Self::which_command(cmd_name);

        // Try to run the version command
        match self.run_version_command(&tool_config.command) {
            Ok(output) => {
                if let Some(ref regex_str) = tool_config.parse_regex {
                    match self.parse_version(&output, regex_str) {
                        Some(version) => ToolDiagnostic {
                            name: tool_config.name.clone(),
                            icon: tool_config.icon.clone(),
                            status: DiagnosticStatus::Ok,
                            command: tool_config.command.clone(),
                            command_path,
                            version: Some(version),
                            error_detail: None,
                            suggestion: None,
                            enabled: tool_config.enabled,
                        },
                        None => {
                            // Command ran but regex didn't match
                            let raw_output = output.trim().to_string();
                            ToolDiagnostic {
                                name: tool_config.name.clone(),
                                icon: tool_config.icon.clone(),
                                status: DiagnosticStatus::Warning,
                                command: tool_config.command.clone(),
                                command_path,
                                version: Some(raw_output.clone()),
                                error_detail: Some(format!(
                                    "version parse: regex '{}' did not match output '{}'",
                                    regex_str,
                                    truncate_string(&raw_output, 80)
                                )),
                                suggestion: Some(
                                    "Check parse_regex in config matches the command output"
                                        .to_string(),
                                ),
                                enabled: tool_config.enabled,
                            }
                        }
                    }
                } else {
                    // No regex, use raw output
                    ToolDiagnostic {
                        name: tool_config.name.clone(),
                        icon: tool_config.icon.clone(),
                        status: DiagnosticStatus::Ok,
                        command: tool_config.command.clone(),
                        command_path,
                        version: Some(output.trim().to_string()),
                        error_detail: None,
                        suggestion: None,
                        enabled: tool_config.enabled,
                    }
                }
            }
            Err(e) => {
                let error_str = e.to_string();
                let (error_detail, suggestion) = if error_str.contains("No such file or directory")
                    || error_str.contains("not found")
                {
                    (
                        format!("command not found: '{}'", cmd_name),
                        Some(format!(
                            "Install {} or add it to your PATH",
                            tool_config.name
                        )),
                    )
                } else {
                    (error_str, None)
                };

                ToolDiagnostic {
                    name: tool_config.name.clone(),
                    icon: tool_config.icon.clone(),
                    status: DiagnosticStatus::Error,
                    command: tool_config.command.clone(),
                    command_path: None,
                    version: None,
                    error_detail: Some(error_detail),
                    suggestion,
                    enabled: tool_config.enabled,
                }
            }
        }
    }

    /// Run diagnostics on all configured tools (both enabled and disabled)
    pub fn diagnose_all(&self) -> DiagnosticSummary {
        let all_tools = self.config.effective_tools();

        let config_path = Config::config_path().map(|p| p.display().to_string());
        let config_exists = config_path
            .as_ref()
            .map(|p| std::path::Path::new(p).exists())
            .unwrap_or(false);

        let diagnostics: Vec<ToolDiagnostic> =
            all_tools.iter().map(|t| self.diagnose_tool(t)).collect();

        let ok_count = diagnostics
            .iter()
            .filter(|d| d.status == DiagnosticStatus::Ok)
            .count();
        let warning_count = diagnostics
            .iter()
            .filter(|d| d.status == DiagnosticStatus::Warning)
            .count();
        let error_count = diagnostics
            .iter()
            .filter(|d| d.status == DiagnosticStatus::Error)
            .count();

        DiagnosticSummary {
            config_path,
            config_exists,
            total: diagnostics.len(),
            ok_count,
            warning_count,
            error_count,
            tools: diagnostics,
        }
    }

    /// Look up the full path of a command using `which`
    fn which_command(cmd: &str) -> Option<String> {
        if cmd.is_empty() {
            return None;
        }
        Command::new("which")
            .arg(cmd)
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }
}

/// Truncate a string to a maximum length, appending "..." if truncated
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
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
        let mut detector = test_detector();
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
        let mut detector = test_detector();
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
        let mut detector = test_detector();
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

        let mut detector = ToolDetector::new(config);
        let info = detector.detect_all();

        // Should return ToolboxInfo even with no tools
        assert!(info.tools.is_empty());
    }

    // --- diagnose_tool tests ---

    #[test]
    fn test_diagnose_tool_available_with_regex() {
        let detector = test_detector();
        let tool_config = ToolConfig {
            name: "Echo".to_string(),
            command: "echo v1.2.3".to_string(),
            parse_regex: Some(r"v?(\d+\.\d+\.\d+)".to_string()),
            icon: Some("T".to_string()),
            enabled: true,
            short_name: None,
        };

        let diag = detector.diagnose_tool(&tool_config);
        assert_eq!(diag.status, DiagnosticStatus::Ok);
        assert_eq!(diag.version, Some("1.2.3".to_string()));
        assert!(diag.command_path.is_some()); // echo should be found in PATH
        assert!(diag.error_detail.is_none());
        assert!(diag.suggestion.is_none());
        assert!(diag.enabled);
    }

    #[test]
    fn test_diagnose_tool_available_no_regex() {
        let detector = test_detector();
        let tool_config = ToolConfig {
            name: "Raw".to_string(),
            command: "echo hello world".to_string(),
            parse_regex: None,
            icon: None,
            enabled: true,
            short_name: None,
        };

        let diag = detector.diagnose_tool(&tool_config);
        assert_eq!(diag.status, DiagnosticStatus::Ok);
        assert_eq!(diag.version, Some("hello world".to_string()));
    }

    #[test]
    fn test_diagnose_tool_unavailable() {
        let detector = test_detector();
        let tool_config = ToolConfig {
            name: "NonExistent".to_string(),
            command: "nonexistent_cmd_xyz --version".to_string(),
            parse_regex: None,
            icon: Some("?".to_string()),
            enabled: false,
            short_name: None,
        };

        let diag = detector.diagnose_tool(&tool_config);
        assert_eq!(diag.status, DiagnosticStatus::Error);
        assert!(diag.version.is_none());
        assert!(diag.command_path.is_none());
        assert!(diag.error_detail.is_some());
        assert!(diag.suggestion.is_some());
        assert!(!diag.enabled);
    }

    #[test]
    fn test_diagnose_tool_regex_mismatch() {
        let detector = test_detector();
        let tool_config = ToolConfig {
            name: "Mismatch".to_string(),
            command: "echo some random output".to_string(),
            parse_regex: Some(r"Python\s+(\d+\.\d+)".to_string()),
            icon: None,
            enabled: true,
            short_name: None,
        };

        let diag = detector.diagnose_tool(&tool_config);
        assert_eq!(diag.status, DiagnosticStatus::Warning);
        assert!(diag.version.is_some()); // raw output is used
        assert!(diag.error_detail.is_some());
        assert!(diag
            .error_detail
            .as_ref()
            .unwrap()
            .contains("did not match"));
        assert!(diag.suggestion.is_some());
    }

    // --- diagnose_all tests ---

    #[test]
    fn test_diagnose_all_empty_config() {
        let config = Config {
            use_default_tools: false,
            ..Config::default()
        };
        let detector = ToolDetector::new(config);
        let summary = detector.diagnose_all();

        assert_eq!(summary.total, 0);
        assert_eq!(summary.ok_count, 0);
        assert_eq!(summary.warning_count, 0);
        assert_eq!(summary.error_count, 0);
        assert!(summary.tools.is_empty());
    }

    #[test]
    fn test_diagnose_all_with_mixed_tools() {
        let mut config = Config {
            use_default_tools: false,
            ..Config::default()
        };
        config.custom_tools.push(ToolConfig {
            name: "GoodTool".to_string(),
            command: "echo v2.0.0".to_string(),
            parse_regex: Some(r"v(\d+\.\d+\.\d+)".to_string()),
            icon: None,
            enabled: true,
            short_name: None,
        });
        config.custom_tools.push(ToolConfig {
            name: "BadTool".to_string(),
            command: "nonexistent_cmd_12345 --version".to_string(),
            parse_regex: None,
            icon: None,
            enabled: true,
            short_name: None,
        });

        let detector = ToolDetector::new(config);
        let summary = detector.diagnose_all();

        assert_eq!(summary.total, 2);
        assert_eq!(summary.ok_count, 1);
        assert_eq!(summary.error_count, 1);
    }

    #[test]
    fn test_diagnose_all_has_config_info() {
        let config = Config {
            use_default_tools: false,
            ..Config::default()
        };
        let detector = ToolDetector::new(config);
        let summary = detector.diagnose_all();

        // config_path should be available (may or may not exist)
        assert!(summary.config_path.is_some());
    }

    // --- truncate_string tests ---

    #[test]
    fn test_truncate_string_short() {
        assert_eq!(truncate_string("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_string_exact() {
        assert_eq!(truncate_string("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_string_long() {
        assert_eq!(truncate_string("hello world", 5), "hello...");
    }

    // --- which_command tests ---

    #[test]
    fn test_which_command_found() {
        let path = ToolDetector::which_command("echo");
        assert!(path.is_some());
    }

    #[test]
    fn test_which_command_not_found() {
        let path = ToolDetector::which_command("nonexistent_cmd_xyz_12345");
        assert!(path.is_none());
    }

    #[test]
    fn test_which_command_empty() {
        let path = ToolDetector::which_command("");
        assert!(path.is_none());
    }

    // --- Cache integration tests ---

    #[test]
    fn test_detect_tool_uses_cache() {
        let mut detector = test_detector();
        let tool_config = ToolConfig {
            name: "Echo".to_string(),
            command: "echo v1.0.0".to_string(),
            parse_regex: Some(r"v?(\d+\.\d+\.\d+)".to_string()),
            icon: None,
            enabled: true,
            short_name: None,
        };

        // First call should be a miss
        let info1 = detector.detect_tool(&tool_config);
        assert!(info1.available);
        assert_eq!(info1.version, Some("1.0.0".to_string()));

        let cache = detector.cache().unwrap();
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 1);

        // Second call should be a hit
        let info2 = detector.detect_tool(&tool_config);
        assert!(info2.available);
        assert_eq!(info2.version, Some("1.0.0".to_string()));

        let cache = detector.cache().unwrap();
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);
    }

    #[test]
    fn test_detect_tool_cache_disabled() {
        let mut detector = test_detector().with_cache_disabled();
        let tool_config = ToolConfig {
            name: "Echo".to_string(),
            command: "echo v1.0.0".to_string(),
            parse_regex: Some(r"v?(\d+\.\d+\.\d+)".to_string()),
            icon: None,
            enabled: true,
            short_name: None,
        };

        let info = detector.detect_tool(&tool_config);
        assert!(info.available);
        assert!(detector.cache().is_none());
    }

    #[test]
    fn test_detect_tool_cache_refresh() {
        let mut detector = test_detector();
        let tool_config = ToolConfig {
            name: "Echo".to_string(),
            command: "echo v1.0.0".to_string(),
            parse_regex: Some(r"v?(\d+\.\d+\.\d+)".to_string()),
            icon: None,
            enabled: true,
            short_name: None,
        };

        // Populate cache
        detector.detect_tool(&tool_config);
        assert_eq!(detector.cache().unwrap().len(), 1);

        // Refresh clears cache
        detector = detector.with_cache_refresh();
        assert_eq!(detector.cache().unwrap().len(), 0);
    }

    #[test]
    fn test_cache_default_enabled() {
        let detector = ToolDetector::with_defaults();
        assert!(detector.cache().is_some());
    }

    #[test]
    fn test_cache_config_disabled() {
        let mut config = Config::default();
        config.cache.enabled = false;
        let detector = ToolDetector::new(config);
        assert!(detector.cache().is_none());
    }

    #[test]
    fn test_cache_config_custom_ttl() {
        let mut config = Config::default();
        config.cache.default_ttl = 60;
        let detector = ToolDetector::new(config);
        assert_eq!(detector.cache().unwrap().default_ttl(), 60);
    }
}
