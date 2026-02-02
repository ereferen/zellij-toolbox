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

    // --- ColorMode ---

    #[test]
    fn test_color_mode_from_str() {
        assert_eq!("auto".parse::<ColorMode>().unwrap(), ColorMode::Auto);
        assert_eq!("always".parse::<ColorMode>().unwrap(), ColorMode::Always);
        assert_eq!("never".parse::<ColorMode>().unwrap(), ColorMode::Never);
        assert!("invalid".parse::<ColorMode>().is_err());
    }

    #[test]
    fn test_color_mode_from_str_case_insensitive() {
        assert_eq!("AUTO".parse::<ColorMode>().unwrap(), ColorMode::Auto);
        assert_eq!("Always".parse::<ColorMode>().unwrap(), ColorMode::Always);
        assert_eq!("NEVER".parse::<ColorMode>().unwrap(), ColorMode::Never);
    }

    #[test]
    fn test_color_mode_default() {
        assert_eq!(ColorMode::default(), ColorMode::Auto);
    }

    #[test]
    fn test_color_mode_invalid_error_message() {
        let err = "invalid".parse::<ColorMode>().unwrap_err();
        assert!(err.contains("Invalid color mode"));
        assert!(err.contains("invalid"));
    }

    // --- Segment constructors ---

    #[test]
    fn test_segment_creation() {
        let seg = Segment::blue("test");
        assert_eq!(seg.text, "test");
    }

    #[test]
    fn test_segment_blue() {
        let seg = Segment::blue("dir");
        assert_eq!(seg.text, "dir");
        assert_eq!(seg.fg, ansi::FG_WHITE);
        assert_eq!(seg.bg, ansi::BG_BLUE);
        assert_eq!(seg.bg_color_fg, ansi::FG_BLUE);
    }

    #[test]
    fn test_segment_green() {
        let seg = Segment::green("ok");
        assert_eq!(seg.text, "ok");
        assert_eq!(seg.fg, ansi::FG_BLACK);
        assert_eq!(seg.bg, ansi::BG_GREEN);
        assert_eq!(seg.bg_color_fg, ansi::FG_GREEN);
    }

    #[test]
    fn test_segment_yellow() {
        let seg = Segment::yellow("warn");
        assert_eq!(seg.text, "warn");
        assert_eq!(seg.fg, ansi::FG_BLACK);
        assert_eq!(seg.bg, ansi::BG_YELLOW);
        assert_eq!(seg.bg_color_fg, ansi::FG_YELLOW);
    }

    #[test]
    fn test_segment_cyan() {
        let seg = Segment::cyan("info");
        assert_eq!(seg.text, "info");
        assert_eq!(seg.fg, ansi::FG_BLACK);
        assert_eq!(seg.bg, ansi::BG_CYAN);
        assert_eq!(seg.bg_color_fg, ansi::FG_CYAN);
    }

    #[test]
    fn test_segment_magenta() {
        let seg = Segment::magenta("special");
        assert_eq!(seg.text, "special");
        assert_eq!(seg.fg, ansi::FG_WHITE);
        assert_eq!(seg.bg, ansi::BG_MAGENTA);
        assert_eq!(seg.bg_color_fg, ansi::FG_MAGENTA);
    }

    #[test]
    fn test_segment_gray() {
        let seg = Segment::gray("muted");
        assert_eq!(seg.text, "muted");
        assert_eq!(seg.fg, ansi::FG_WHITE);
        assert_eq!(seg.bg, ansi::BG_GRAY);
        assert_eq!(seg.bg_color_fg, ansi::FG_GRAY);
    }

    #[test]
    fn test_segment_dark_gray() {
        let seg = Segment::dark_gray("bg");
        assert_eq!(seg.text, "bg");
        assert_eq!(seg.fg, ansi::FG_WHITE);
        assert_eq!(seg.bg, ansi::BG_DARK_GRAY);
        assert_eq!(seg.bg_color_fg, ansi::FG_DARK_GRAY);
    }

    #[test]
    fn test_segment_new_custom() {
        let seg = Segment::new("custom", ansi::FG_BLACK, ansi::BG_RED, ansi::FG_RED);
        assert_eq!(seg.text, "custom");
        assert_eq!(seg.fg, ansi::FG_BLACK);
        assert_eq!(seg.bg, ansi::BG_RED);
        assert_eq!(seg.bg_color_fg, ansi::FG_RED);
    }

    #[test]
    fn test_segment_from_string_type() {
        let seg = Segment::blue(String::from("owned"));
        assert_eq!(seg.text, "owned");
    }

    // --- render_powerline ---

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

    #[test]
    fn test_render_powerline_empty() {
        let segments: Vec<Segment> = vec![];
        let result = render_powerline(&segments, true);
        assert!(result.is_empty());
    }

    #[test]
    fn test_render_powerline_empty_no_color() {
        let segments: Vec<Segment> = vec![];
        let result = render_powerline(&segments, false);
        assert!(result.is_empty());
    }

    #[test]
    fn test_render_powerline_single_segment() {
        let segments = vec![Segment::blue("only")];
        let result = render_powerline(&segments, true);
        assert!(result.contains("only"));
        assert!(result.contains(ansi::RESET));
    }

    #[test]
    fn test_render_powerline_single_segment_no_color() {
        let segments = vec![Segment::blue("only")];
        let result = render_powerline(&segments, false);
        assert_eq!(result, "only");
    }

    #[test]
    fn test_render_powerline_separator_present() {
        let segments = vec![Segment::blue("a"), Segment::green("b")];
        let result = render_powerline(&segments, true);
        assert!(result.contains(SEPARATOR_RIGHT));
    }

    #[test]
    fn test_render_powerline_three_segments() {
        let segments = vec![
            Segment::blue("one"),
            Segment::green("two"),
            Segment::cyan("three"),
        ];
        let result = render_powerline(&segments, true);
        assert!(result.contains("one"));
        assert!(result.contains("two"));
        assert!(result.contains("three"));
    }

    #[test]
    fn test_render_powerline_ends_with_reset() {
        let segments = vec![Segment::blue("test")];
        let result = render_powerline(&segments, true);
        assert!(result.ends_with(ansi::RESET));
    }

    // --- render_powerline_multiline ---

    #[test]
    fn test_render_powerline_multiline_no_color() {
        let segments = vec![Segment::blue("dir"), Segment::green("main")];
        let result = render_powerline_multiline(&segments, false);
        assert_eq!(result, " dir\n main");
    }

    #[test]
    fn test_render_powerline_multiline_with_color() {
        let segments = vec![Segment::blue("dir"), Segment::green("main")];
        let result = render_powerline_multiline(&segments, true);
        assert!(result.contains("dir"));
        assert!(result.contains("main"));
        assert!(result.contains('\n'));
        assert!(result.contains(ansi::RESET));
    }

    #[test]
    fn test_render_powerline_multiline_empty() {
        let segments: Vec<Segment> = vec![];
        let result = render_powerline_multiline(&segments, true);
        assert!(result.is_empty());
    }

    #[test]
    fn test_render_powerline_multiline_single() {
        let segments = vec![Segment::cyan("only")];
        let result = render_powerline_multiline(&segments, true);
        assert!(result.contains("only"));
        assert!(!result.contains('\n'));
    }

    #[test]
    fn test_render_powerline_multiline_each_line_has_separator() {
        let segments = vec![Segment::blue("a"), Segment::green("b")];
        let result = render_powerline_multiline(&segments, true);
        for line in result.lines() {
            assert!(
                line.contains(SEPARATOR_RIGHT),
                "Each line should have separator"
            );
        }
    }

    // --- should_use_color ---

    #[test]
    fn test_should_use_color_always() {
        assert!(should_use_color(ColorMode::Always));
    }

    #[test]
    fn test_should_use_color_never() {
        assert!(!should_use_color(ColorMode::Never));
    }
}
