//! toolbox-core: Core library for tool version detection and system info
//!
//! This library provides:
//! - Configuration loading and management
//! - Tool version detection (Python, Node, Docker, etc.)
//! - Directory-aware version detection (asdf, mise, nvm support)
//! - Git repository information
//! - System resource information

pub mod color;
pub mod config;
pub mod detector;
pub mod error;
pub mod info;

pub use config::Config;
pub use detector::ToolDetector;
pub use error::ToolboxError;
pub use info::{
    DiagnosticStatus, DiagnosticSummary, GitInfo, SystemInfo, ToolDiagnostic, ToolInfo, ToolboxInfo,
};
