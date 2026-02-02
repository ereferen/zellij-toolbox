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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_config() {
        let err = ToolboxError::Config("bad config".to_string());
        assert_eq!(err.to_string(), "Configuration error: bad config");
    }

    #[test]
    fn test_error_display_command_failed() {
        let err = ToolboxError::CommandFailed("python: not found".to_string());
        assert_eq!(
            err.to_string(),
            "Command execution failed: python: not found"
        );
    }

    #[test]
    fn test_error_display_version_parse() {
        let err = ToolboxError::VersionParse("no match".to_string());
        assert_eq!(err.to_string(), "Version parse error: no match");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = ToolboxError::from(io_err);
        assert!(err.to_string().contains("file missing"));
    }

    #[test]
    fn test_error_from_toml() {
        let toml_result: std::result::Result<toml::Value, _> = toml::from_str("invalid {{");
        let toml_err = toml_result.unwrap_err();
        let err = ToolboxError::from(toml_err);
        assert!(err.to_string().contains("TOML parse error"));
    }

    #[test]
    fn test_error_from_regex() {
        let bad_regex = "[invalid(";
        let regex_err = regex::Regex::new(bad_regex).unwrap_err();
        let err = ToolboxError::from(regex_err);
        assert!(err.to_string().contains("Regex error"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        match result {
            Ok(v) => assert_eq!(v, 42),
            Err(_) => panic!("expected Ok"),
        }
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(ToolboxError::Config("test".to_string()));
        assert!(result.is_err());
    }
}
