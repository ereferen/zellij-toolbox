//! Error types for toolbox

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolboxError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Version parse error: {0}")]
    VersionParse(String),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[cfg(feature = "git")]
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
}

pub type Result<T> = std::result::Result<T, ToolboxError>;
