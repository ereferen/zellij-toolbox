//! Configuration management for toolbox

use crate::error::{Result, ToolboxError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Display settings
    pub display: DisplayConfig,
    /// Tool definitions
    pub tools: Vec<ToolConfig>,
    /// Extra information settings
    pub extras: ExtrasConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            tools: default_tools(),
            extras: ExtrasConfig::default(),
        }
    }
}

/// Display-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    /// Refresh interval in seconds
    pub refresh_interval: u64,
    /// Show icons (emoji)
    pub show_icons: bool,
    /// Compact mode (shorter version strings)
    pub compact: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            refresh_interval: 5,
            show_icons: true,
            compact: true,
        }
    }
}

/// Configuration for a single tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Tool name for display
    pub name: String,
    /// Command to run to get version
    pub command: String,
    /// Optional regex to extract version from output
    #[serde(default)]
    pub parse_regex: Option<String>,
    /// Icon/emoji for display
    #[serde(default)]
    pub icon: Option<String>,
    /// Whether this tool is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Short name for compact display
    #[serde(default)]
    pub short_name: Option<String>,
}

/// Extra information settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtrasConfig {
    /// Show current git branch
    pub git_branch: bool,
    /// Show git status (changed files count)
    pub git_status: bool,
    /// Show memory usage
    pub system_memory: bool,
    /// Show CPU usage
    pub system_cpu: bool,
    /// Show current directory
    pub current_directory: bool,
    /// Show virtual environment name
    pub virtual_env: bool,
    /// Show shell name
    pub shell: bool,
}

impl Default for ExtrasConfig {
    fn default() -> Self {
        Self {
            git_branch: true,
            git_status: true,
            system_memory: false,
            system_cpu: false,
            current_directory: true,
            virtual_env: true,
            shell: false,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Returns the default set of tools
fn default_tools() -> Vec<ToolConfig> {
    vec![
        ToolConfig {
            name: "Python".to_string(),
            command: "python3 --version".to_string(),
            parse_regex: Some(r"Python\s+(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🐍".to_string()),
            enabled: true,
            short_name: Some("py".to_string()),
        },
        ToolConfig {
            name: "Node".to_string(),
            command: "node --version".to_string(),
            parse_regex: Some(r"v?(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("📦".to_string()),
            enabled: true,
            short_name: Some("node".to_string()),
        },
        ToolConfig {
            name: "npm".to_string(),
            command: "npm --version".to_string(),
            parse_regex: Some(r"(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("📦".to_string()),
            enabled: false, // disabled by default, often redundant with node
            short_name: Some("npm".to_string()),
        },
        ToolConfig {
            name: "Docker".to_string(),
            command: "docker --version".to_string(),
            parse_regex: Some(r"Docker version\s+(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🐳".to_string()),
            enabled: true,
            short_name: Some("docker".to_string()),
        },
        ToolConfig {
            name: "Rust".to_string(),
            command: "rustc --version".to_string(),
            parse_regex: Some(r"rustc\s+(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🦀".to_string()),
            enabled: true,
            short_name: Some("rust".to_string()),
        },
        ToolConfig {
            name: "Go".to_string(),
            command: "go version".to_string(),
            parse_regex: Some(r"go(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🔷".to_string()),
            enabled: true,
            short_name: Some("go".to_string()),
        },
        ToolConfig {
            name: "Ruby".to_string(),
            command: "ruby --version".to_string(),
            parse_regex: Some(r"ruby\s+(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("💎".to_string()),
            enabled: false,
            short_name: Some("ruby".to_string()),
        },
        ToolConfig {
            name: "Java".to_string(),
            command: "java --version".to_string(),
            parse_regex: Some(r"(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("☕".to_string()),
            enabled: false,
            short_name: Some("java".to_string()),
        },
        ToolConfig {
            name: "Deno".to_string(),
            command: "deno --version".to_string(),
            parse_regex: Some(r"deno\s+(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🦕".to_string()),
            enabled: false,
            short_name: Some("deno".to_string()),
        },
        ToolConfig {
            name: "Bun".to_string(),
            command: "bun --version".to_string(),
            parse_regex: Some(r"(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🥟".to_string()),
            enabled: false,
            short_name: Some("bun".to_string()),
        },
    ]
}

impl Config {
    /// Load configuration from the default path
    pub fn load() -> Result<Self> {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                return Self::load_from_path(&path);
            }
        }
        Ok(Self::default())
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to the default path
    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            self.save_to_path(&path)?;
        }
        Ok(())
    }

    /// Save configuration to a specific path
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| ToolboxError::Config(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the default configuration file path
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("toolbox").join("config.toml"))
    }

    /// Get only enabled tools
    pub fn enabled_tools(&self) -> Vec<&ToolConfig> {
        self.tools.iter().filter(|t| t.enabled).collect()
    }
}
