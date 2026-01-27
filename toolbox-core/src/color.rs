//! ANSI color support for terminal output
//!
//! Provides Powerline-style colored output with segments.

/// ANSI color codes
pub mod ansi {
    // Reset
    pub const RESET: &str = "\x1b[0m";

    // Foreground colors
    pub const FG_BLACK: &str = "\x1b[30m";
    pub const FG_WHITE: &str = "\x1b[97m";
    pub const FG_BRIGHT_WHITE: &str = "\x1b[97m";

    // Background colors (using 256 color palette for better appearance)
    pub const BG_BLUE: &str = "\x1b[44m";
    pub const BG_GREEN: &str = "\x1b[42m";
    pub const BG_YELLOW: &str = "\x1b[43m";
    pub const BG_RED: &str = "\x1b[41m";
    pub const BG_MAGENTA: &str = "\x1b[45m";
    pub const BG_CYAN: &str = "\x1b[46m";
    pub const BG_GRAY: &str = "\x1b[100m";
    pub const BG_DARK_GRAY: &str = "\x1b[48;5;236m";

    // Foreground colors for separators (matching backgrounds)
    pub const FG_BLUE: &str = "\x1b[34m";
    pub const FG_GREEN: &str = "\x1b[32m";
    pub const FG_YELLOW: &str = "\x1b[33m";
    pub const FG_RED: &str = "\x1b[31m";
    pub const FG_MAGENTA: &str = "\x1b[35m";
    pub const FG_CYAN: &str = "\x1b[36m";
    pub const FG_GRAY: &str = "\x1b[90m";
    pub const FG_DARK_GRAY: &str = "\x1b[38;5;236m";
}

/// Powerline separator characters
pub const SEPARATOR_RIGHT: char = '\u{E0B0}'; //
pub const SEPARATOR_RIGHT_THIN: char = '\u{E0B1}'; //

/// Color mode for output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    /// Automatically detect if terminal supports colors
    #[default]
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

impl std::str::FromStr for ColorMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(ColorMode::Auto),
            "always" => Ok(ColorMode::Always),
            "never" => Ok(ColorMode::Never),
            _ => Err(format!("Invalid color mode: {}", s)),
        }
    }
}

/// A colored segment in the powerline
#[derive(Debug, Clone)]
pub struct Segment {
    pub text: String,
    pub fg: &'static str,
    pub bg: &'static str,
    pub bg_color_fg: &'static str, // foreground color matching the background (for separator)
}

impl Segment {
    pub fn new(
        text: impl Into<String>,
        fg: &'static str,
        bg: &'static str,
        bg_color_fg: &'static str,
    ) -> Self {
        Self {
            text: text.into(),
            fg,
            bg,
            bg_color_fg,
        }
    }

    /// Create a blue segment (for directory)
    pub fn blue(text: impl Into<String>) -> Self {
        Self::new(text, ansi::FG_WHITE, ansi::BG_BLUE, ansi::FG_BLUE)
    }

    /// Create a green segment (for clean git status)
    pub fn green(text: impl Into<String>) -> Self {
        Self::new(text, ansi::FG_BLACK, ansi::BG_GREEN, ansi::FG_GREEN)
    }

    /// Create a yellow segment (for dirty git status)
    pub fn yellow(text: impl Into<String>) -> Self {
        Self::new(text, ansi::FG_BLACK, ansi::BG_YELLOW, ansi::FG_YELLOW)
    }

    /// Create a cyan segment (for tools)
    pub fn cyan(text: impl Into<String>) -> Self {
        Self::new(text, ansi::FG_BLACK, ansi::BG_CYAN, ansi::FG_CYAN)
    }

    /// Create a magenta segment (for tools)
    pub fn magenta(text: impl Into<String>) -> Self {
        Self::new(text, ansi::FG_WHITE, ansi::BG_MAGENTA, ansi::FG_MAGENTA)
    }

    /// Create a gray segment
    pub fn gray(text: impl Into<String>) -> Self {
        Self::new(text, ansi::FG_WHITE, ansi::BG_GRAY, ansi::FG_GRAY)
    }

    /// Create a dark gray segment
    pub fn dark_gray(text: impl Into<String>) -> Self {
        Self::new(text, ansi::FG_WHITE, ansi::BG_DARK_GRAY, ansi::FG_DARK_GRAY)
    }
}

/// Render segments as a powerline string
pub fn render_powerline(segments: &[Segment], use_color: bool) -> String {
    if !use_color || segments.is_empty() {
        // Plain text fallback
        return segments
            .iter()
            .map(|s| s.text.clone())
            .collect::<Vec<_>>()
            .join(" | ");
    }

    let mut result = String::new();

    for (i, segment) in segments.iter().enumerate() {
        // Background and foreground for this segment
        result.push_str(segment.bg);
        result.push_str(segment.fg);
        result.push(' ');
        result.push_str(&segment.text);
        result.push(' ');

        // Separator
        if i < segments.len() - 1 {
            // Next segment's background
            let next_bg = &segments[i + 1].bg;
            result.push_str(ansi::RESET);
            result.push_str(next_bg);
            result.push_str(segment.bg_color_fg);
            result.push(SEPARATOR_RIGHT);
        } else {
            // Final separator
            result.push_str(ansi::RESET);
            result.push_str(segment.bg_color_fg);
            result.push(SEPARATOR_RIGHT);
            result.push_str(ansi::RESET);
        }
    }

    result
}

/// Render segments as multiline powerline (each segment on its own line)
pub fn render_powerline_multiline(segments: &[Segment], use_color: bool) -> String {
    if !use_color || segments.is_empty() {
        // Plain text fallback
        return segments
            .iter()
            .map(|s| format!(" {}", s.text))
            .collect::<Vec<_>>()
            .join("\n");
    }

    let mut lines = Vec::new();

    for segment in segments {
        let mut line = String::new();

        // Background and foreground for this segment
        line.push_str(segment.bg);
        line.push_str(segment.fg);
        line.push(' ');
        line.push_str(&segment.text);
        line.push(' ');

        // End of line separator
        line.push_str(ansi::RESET);
        line.push_str(segment.bg_color_fg);
        line.push(SEPARATOR_RIGHT);
        line.push_str(ansi::RESET);

        lines.push(line);
    }

    lines.join("\n")
}

/// Check if stdout is a terminal that supports colors
pub fn should_use_color(mode: ColorMode) -> bool {
    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            // Check if stdout is a TTY using std::io::IsTerminal (Rust 1.70+)
            use std::io::IsTerminal;
            std::io::stdout().is_terminal()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode_from_str() {
        assert_eq!("auto".parse::<ColorMode>().unwrap(), ColorMode::Auto);
        assert_eq!("always".parse::<ColorMode>().unwrap(), ColorMode::Always);
        assert_eq!("never".parse::<ColorMode>().unwrap(), ColorMode::Never);
        assert!("invalid".parse::<ColorMode>().is_err());
    }

    #[test]
    fn test_segment_creation() {
        let seg = Segment::blue("test");
        assert_eq!(seg.text, "test");
    }

    #[test]
    fn test_render_powerline_no_color() {
        let segments = vec![Segment::blue("dir"), Segment::green("main")];
        let result = render_powerline(&segments, false);
        assert_eq!(result, "dir | main");
    }

    #[test]
    fn test_render_powerline_with_color() {
        let segments = vec![Segment::blue("dir"), Segment::green("main")];
        let result = render_powerline(&segments, true);
        assert!(result.contains("\x1b[")); // Contains ANSI codes
        assert!(result.contains("dir"));
        assert!(result.contains("main"));
    }
}
