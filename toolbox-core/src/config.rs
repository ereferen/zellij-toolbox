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
            name: "pnpm".to_string(),
            command: "pnpm --version".to_string(),
            parse_regex: Some(r"(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("📦".to_string()),
            enabled: false,
            short_name: Some("pnpm".to_string()),
        },
        ToolConfig {
            name: "yarn".to_string(),
            command: "yarn --version".to_string(),
            parse_regex: Some(r"(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🧶".to_string()),
            enabled: false,
            short_name: Some("yarn".to_string()),
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
            name: "PHP".to_string(),
            command: "php --version".to_string(),
            parse_regex: Some(r"PHP\s+(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🐘".to_string()),
            enabled: false,
            short_name: Some("php".to_string()),
        },
        ToolConfig {
            name: "Elixir".to_string(),
            command: "elixir --version".to_string(),
            parse_regex: Some(r"Elixir\s+(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("💧".to_string()),
            enabled: false,
            short_name: Some("elixir".to_string()),
        },
        ToolConfig {
            name: "Zig".to_string(),
            command: "zig version".to_string(),
            parse_regex: Some(r"(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("⚡".to_string()),
            enabled: false,
            short_name: Some("zig".to_string()),
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
        // DevOps tools
        ToolConfig {
            name: "kubectl".to_string(),
            command: "kubectl version --client --short 2>/dev/null || kubectl version --client".to_string(),
            parse_regex: Some(r"v?(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("☸️".to_string()),
            enabled: false,
            short_name: Some("k8s".to_string()),
        },
        ToolConfig {
            name: "terraform".to_string(),
            command: "terraform --version".to_string(),
            parse_regex: Some(r"Terraform\s+v?(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🏗️".to_string()),
            enabled: false,
            short_name: Some("tf".to_string()),
        },
        ToolConfig {
            name: "aws-cli".to_string(),
            command: "aws --version".to_string(),
            parse_regex: Some(r"aws-cli/(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("☁️".to_string()),
            enabled: false,
            short_name: Some("aws".to_string()),
        },
        // Version managers
        ToolConfig {
            name: "mise".to_string(),
            command: "mise --version".to_string(),
            parse_regex: Some(r"mise\s+(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🔧".to_string()),
            enabled: false,
            short_name: Some("mise".to_string()),
        },
        ToolConfig {
            name: "asdf".to_string(),
            command: "asdf --version".to_string(),
            parse_regex: Some(r"v?(\d+\.\d+(?:\.\d+)?)".to_string()),
            icon: Some("🔧".to_string()),
            enabled: false,
            short_name: Some("asdf".to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.tools.is_empty());
        assert!(config.display.show_icons);
        assert!(config.display.compact);
        assert_eq!(config.display.refresh_interval, 5);
    }

    #[test]
    fn test_default_display_config() {
        let display = DisplayConfig::default();
        assert_eq!(display.refresh_interval, 5);
        assert!(display.show_icons);
        assert!(display.compact);
    }

    #[test]
    fn test_default_extras_config() {
        let extras = ExtrasConfig::default();
        assert!(extras.git_branch);
        assert!(extras.git_status);
        assert!(!extras.system_memory);
        assert!(!extras.system_cpu);
        assert!(extras.current_directory);
        assert!(extras.virtual_env);
        assert!(!extras.shell);
    }

    #[test]
    fn test_enabled_tools() {
        let config = Config::default();
        let enabled = config.enabled_tools();
        // Check that all returned tools are enabled
        for tool in &enabled {
            assert!(tool.enabled);
        }
        // Default config has some enabled tools
        assert!(!enabled.is_empty());
    }

    #[test]
    fn test_config_save_and_load() {
        let config = Config::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Save
        config.save_to_path(&path).unwrap();

        // Load
        let loaded = Config::load_from_path(&path).unwrap();

        assert_eq!(loaded.tools.len(), config.tools.len());
        assert_eq!(loaded.display.refresh_interval, config.display.refresh_interval);
        assert_eq!(loaded.display.show_icons, config.display.show_icons);
    }

    #[test]
    fn test_config_toml_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.tools.len(), config.tools.len());
        assert_eq!(parsed.display.compact, config.display.compact);
    }

    #[test]
    fn test_tool_config_serde() {
        let tool = ToolConfig {
            name: "Test".to_string(),
            command: "test --version".to_string(),
            parse_regex: Some(r"(\d+\.\d+)".to_string()),
            icon: Some("🔧".to_string()),
            enabled: true,
            short_name: Some("t".to_string()),
        };

        let toml_str = toml::to_string(&tool).unwrap();
        let parsed: ToolConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.name, tool.name);
        assert_eq!(parsed.command, tool.command);
        assert_eq!(parsed.parse_regex, tool.parse_regex);
        assert_eq!(parsed.icon, tool.icon);
        assert_eq!(parsed.enabled, tool.enabled);
        assert_eq!(parsed.short_name, tool.short_name);
    }

    #[test]
    fn test_config_load_nonexistent() {
        let path = PathBuf::from("/nonexistent/path/config.toml");
        let result = Config::load_from_path(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_invalid_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid toml {{{{").unwrap();
        let path = temp_file.path().to_path_buf();

        let result = Config::load_from_path(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_tools_have_required_fields() {
        let tools = default_tools();
        for tool in tools {
            assert!(!tool.name.is_empty());
            assert!(!tool.command.is_empty());
            // parse_regex should be valid if present
            if let Some(ref regex) = tool.parse_regex {
                assert!(regex::Regex::new(regex).is_ok(), "Invalid regex for {}", tool.name);
            }
        }
    }
}
