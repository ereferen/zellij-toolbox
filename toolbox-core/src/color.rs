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
    pub fg: String,
    pub bg: String,
    pub bg_color_fg: String, // foreground color matching the background (for separator)
}

impl Segment {
    pub fn new(text: impl Into<String>, fg: &str, bg: &str, bg_color_fg: &str) -> Self {
        Self {
            text: text.into(),
            fg: fg.to_string(),
            bg: bg.to_string(),
            bg_color_fg: bg_color_fg.to_string(),
        }
    }

    /// Create a segment from ThemeColor values
    pub fn from_theme_colors(
        text: impl Into<String>,
        fg_color: &ThemeColor,
        bg_color: &ThemeColor,
    ) -> Self {
        Self {
            text: text.into(),
            fg: fg_color.to_ansi_fg(),
            bg: bg_color.to_ansi_bg(),
            bg_color_fg: bg_color.to_ansi_fg(),
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
        result.push_str(&segment.bg);
        result.push_str(&segment.fg);
        result.push(' ');
        result.push_str(&segment.text);
        result.push(' ');

        // Separator
        if i < segments.len() - 1 {
            // Next segment's background
            let next_bg = &segments[i + 1].bg;
            result.push_str(ansi::RESET);
            result.push_str(next_bg);
            result.push_str(&segment.bg_color_fg);
            result.push(SEPARATOR_RIGHT);
        } else {
            // Final separator
            result.push_str(ansi::RESET);
            result.push_str(&segment.bg_color_fg);
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
        line.push_str(&segment.bg);
        line.push_str(&segment.fg);
        line.push(' ');
        line.push_str(&segment.text);
        line.push(' ');

        // End of line separator
        line.push_str(ansi::RESET);
        line.push_str(&segment.bg_color_fg);
        line.push(SEPARATOR_RIGHT);
        line.push_str(ansi::RESET);

        lines.push(line);
    }

    lines.join("\n")
}

use crate::config::{CustomThemeConfig, ThemeColor, ThemeConfig};

/// A fully resolved theme with all colors determined
#[derive(Debug, Clone)]
pub struct ResolvedTheme {
    pub directory_bg: ThemeColor,
    pub directory_fg: ThemeColor,
    pub git_clean_bg: ThemeColor,
    pub git_clean_fg: ThemeColor,
    pub git_dirty_bg: ThemeColor,
    pub git_dirty_fg: ThemeColor,
    pub tool_colors: Vec<(ThemeColor, ThemeColor)>, // (bg, fg) pairs
    pub venv_bg: ThemeColor,
    pub venv_fg: ThemeColor,
}

impl ResolvedTheme {
    /// Default theme (matches the original hardcoded colors)
    pub fn default_theme() -> Self {
        Self {
            directory_bg: ThemeColor::Blue,
            directory_fg: ThemeColor::White,
            git_clean_bg: ThemeColor::Green,
            git_clean_fg: ThemeColor::Black,
            git_dirty_bg: ThemeColor::Yellow,
            git_dirty_fg: ThemeColor::Black,
            tool_colors: vec![
                (ThemeColor::Cyan, ThemeColor::Black),
                (ThemeColor::Magenta, ThemeColor::White),
                (ThemeColor::Gray, ThemeColor::White),
            ],
            venv_bg: ThemeColor::Green,
            venv_fg: ThemeColor::Black,
        }
    }

    /// Dark theme
    pub fn dark_theme() -> Self {
        Self {
            directory_bg: ThemeColor::Rgb(0x34, 0x65, 0xA4),
            directory_fg: ThemeColor::White,
            git_clean_bg: ThemeColor::Rgb(0x4E, 0x9A, 0x06),
            git_clean_fg: ThemeColor::White,
            git_dirty_bg: ThemeColor::Rgb(0xC4, 0xA0, 0x00),
            git_dirty_fg: ThemeColor::Black,
            tool_colors: vec![
                (ThemeColor::Rgb(0x06, 0x98, 0x9A), ThemeColor::White),
                (ThemeColor::Rgb(0x75, 0x50, 0x7B), ThemeColor::White),
                (ThemeColor::Rgb(0x55, 0x57, 0x53), ThemeColor::White),
            ],
            venv_bg: ThemeColor::Rgb(0x4E, 0x9A, 0x06),
            venv_fg: ThemeColor::White,
        }
    }

    /// Light theme
    pub fn light_theme() -> Self {
        Self {
            directory_bg: ThemeColor::Rgb(0x72, 0x9F, 0xCF),
            directory_fg: ThemeColor::Black,
            git_clean_bg: ThemeColor::Rgb(0x8A, 0xE2, 0x34),
            git_clean_fg: ThemeColor::Black,
            git_dirty_bg: ThemeColor::Rgb(0xFC, 0xE9, 0x4F),
            git_dirty_fg: ThemeColor::Black,
            tool_colors: vec![
                (ThemeColor::Rgb(0x34, 0xE2, 0xE2), ThemeColor::Black),
                (ThemeColor::Rgb(0xAD, 0x7F, 0xA8), ThemeColor::Black),
                (ThemeColor::Rgb(0xBA, 0xBD, 0xB6), ThemeColor::Black),
            ],
            venv_bg: ThemeColor::Rgb(0x8A, 0xE2, 0x34),
            venv_fg: ThemeColor::Black,
        }
    }

    /// Solarized theme
    pub fn solarized_theme() -> Self {
        Self {
            directory_bg: ThemeColor::Rgb(0x26, 0x8B, 0xD2), // blue
            directory_fg: ThemeColor::Rgb(0xFD, 0xF6, 0xE3), // base3
            git_clean_bg: ThemeColor::Rgb(0x85, 0x99, 0x00), // green
            git_clean_fg: ThemeColor::Rgb(0xFD, 0xF6, 0xE3),
            git_dirty_bg: ThemeColor::Rgb(0xB5, 0x89, 0x00), // yellow
            git_dirty_fg: ThemeColor::Rgb(0xFD, 0xF6, 0xE3),
            tool_colors: vec![
                (
                    ThemeColor::Rgb(0x2A, 0xA1, 0x98), // cyan
                    ThemeColor::Rgb(0xFD, 0xF6, 0xE3),
                ),
                (
                    ThemeColor::Rgb(0xD3, 0x36, 0x82), // magenta
                    ThemeColor::Rgb(0xFD, 0xF6, 0xE3),
                ),
                (
                    ThemeColor::Rgb(0x58, 0x6E, 0x75), // base01
                    ThemeColor::Rgb(0xFD, 0xF6, 0xE3),
                ),
            ],
            venv_bg: ThemeColor::Rgb(0x85, 0x99, 0x00),
            venv_fg: ThemeColor::Rgb(0xFD, 0xF6, 0xE3),
        }
    }

    /// Get a preset theme by name
    pub fn from_preset(name: &str) -> Self {
        match name {
            "dark" => Self::dark_theme(),
            "light" => Self::light_theme(),
            "solarized" => Self::solarized_theme(),
            _ => Self::default_theme(),
        }
    }

    /// Resolve a theme from config: start with preset, apply custom overrides
    pub fn from_config(config: &ThemeConfig) -> Self {
        let mut theme = Self::from_preset(&config.preset);

        if let Some(ref custom) = config.custom {
            Self::apply_custom(&mut theme, custom);
        }

        theme
    }

    fn apply_custom(theme: &mut Self, custom: &CustomThemeConfig) {
        if let Some(ref c) = custom.directory_bg {
            theme.directory_bg = c.clone();
        }
        if let Some(ref c) = custom.directory_fg {
            theme.directory_fg = c.clone();
        }
        if let Some(ref c) = custom.git_clean_bg {
            theme.git_clean_bg = c.clone();
        }
        if let Some(ref c) = custom.git_clean_fg {
            theme.git_clean_fg = c.clone();
        }
        if let Some(ref c) = custom.git_dirty_bg {
            theme.git_dirty_bg = c.clone();
        }
        if let Some(ref c) = custom.git_dirty_fg {
            theme.git_dirty_fg = c.clone();
        }
        if let Some(ref c) = custom.venv_bg {
            theme.venv_bg = c.clone();
        }
        if let Some(ref c) = custom.venv_fg {
            theme.venv_fg = c.clone();
        }
        // For tool_bg/tool_fg, rebuild the tool_colors pairs
        if let Some(ref bgs) = custom.tool_bg {
            let fgs = custom.tool_fg.as_deref();
            let mut new_colors = Vec::new();
            for (i, bg) in bgs.iter().enumerate() {
                let fg = fgs.and_then(|f| f.get(i)).cloned().unwrap_or_else(|| {
                    theme
                        .tool_colors
                        .get(i)
                        .map(|c| c.1.clone())
                        .unwrap_or(ThemeColor::White)
                });
                new_colors.push((bg.clone(), fg));
            }
            theme.tool_colors = new_colors;
        } else if let Some(ref fgs) = custom.tool_fg {
            // Only fg overrides, keep existing bg
            for (i, fg) in fgs.iter().enumerate() {
                if let Some(pair) = theme.tool_colors.get_mut(i) {
                    pair.1 = fg.clone();
                }
            }
        }
    }
}

impl ThemeColor {
    /// Convert to ANSI background escape sequence
    pub fn to_ansi_bg(&self) -> String {
        match self {
            Self::Blue => ansi::BG_BLUE.to_string(),
            Self::Green => ansi::BG_GREEN.to_string(),
            Self::Yellow => ansi::BG_YELLOW.to_string(),
            Self::Cyan => ansi::BG_CYAN.to_string(),
            Self::Magenta => ansi::BG_MAGENTA.to_string(),
            Self::Gray => ansi::BG_GRAY.to_string(),
            Self::DarkGray => ansi::BG_DARK_GRAY.to_string(),
            Self::Red => ansi::BG_RED.to_string(),
            Self::White => "\x1b[107m".to_string(),
            Self::Black => "\x1b[40m".to_string(),
            Self::Rgb(r, g, b) => format!("\x1b[48;2;{};{};{}m", r, g, b),
        }
    }

    /// Convert to ANSI foreground escape sequence
    pub fn to_ansi_fg(&self) -> String {
        match self {
            Self::Blue => ansi::FG_BLUE.to_string(),
            Self::Green => ansi::FG_GREEN.to_string(),
            Self::Yellow => ansi::FG_YELLOW.to_string(),
            Self::Cyan => ansi::FG_CYAN.to_string(),
            Self::Magenta => ansi::FG_MAGENTA.to_string(),
            Self::Gray => ansi::FG_GRAY.to_string(),
            Self::DarkGray => ansi::FG_DARK_GRAY.to_string(),
            Self::Red => ansi::FG_RED.to_string(),
            Self::White => ansi::FG_WHITE.to_string(),
            Self::Black => ansi::FG_BLACK.to_string(),
            Self::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
        }
    }
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
        assert_eq!(seg.fg.as_str(), ansi::FG_WHITE);
        assert_eq!(seg.bg.as_str(), ansi::BG_BLUE);
        assert_eq!(seg.bg_color_fg.as_str(), ansi::FG_BLUE);
    }

    #[test]
    fn test_segment_green() {
        let seg = Segment::green("ok");
        assert_eq!(seg.text, "ok");
        assert_eq!(seg.fg.as_str(), ansi::FG_BLACK);
        assert_eq!(seg.bg.as_str(), ansi::BG_GREEN);
        assert_eq!(seg.bg_color_fg.as_str(), ansi::FG_GREEN);
    }

    #[test]
    fn test_segment_yellow() {
        let seg = Segment::yellow("warn");
        assert_eq!(seg.text, "warn");
        assert_eq!(seg.fg.as_str(), ansi::FG_BLACK);
        assert_eq!(seg.bg.as_str(), ansi::BG_YELLOW);
        assert_eq!(seg.bg_color_fg.as_str(), ansi::FG_YELLOW);
    }

    #[test]
    fn test_segment_cyan() {
        let seg = Segment::cyan("info");
        assert_eq!(seg.text, "info");
        assert_eq!(seg.fg.as_str(), ansi::FG_BLACK);
        assert_eq!(seg.bg.as_str(), ansi::BG_CYAN);
        assert_eq!(seg.bg_color_fg.as_str(), ansi::FG_CYAN);
    }

    #[test]
    fn test_segment_magenta() {
        let seg = Segment::magenta("special");
        assert_eq!(seg.text, "special");
        assert_eq!(seg.fg.as_str(), ansi::FG_WHITE);
        assert_eq!(seg.bg.as_str(), ansi::BG_MAGENTA);
        assert_eq!(seg.bg_color_fg.as_str(), ansi::FG_MAGENTA);
    }

    #[test]
    fn test_segment_gray() {
        let seg = Segment::gray("muted");
        assert_eq!(seg.text, "muted");
        assert_eq!(seg.fg.as_str(), ansi::FG_WHITE);
        assert_eq!(seg.bg.as_str(), ansi::BG_GRAY);
        assert_eq!(seg.bg_color_fg.as_str(), ansi::FG_GRAY);
    }

    #[test]
    fn test_segment_dark_gray() {
        let seg = Segment::dark_gray("bg");
        assert_eq!(seg.text, "bg");
        assert_eq!(seg.fg.as_str(), ansi::FG_WHITE);
        assert_eq!(seg.bg.as_str(), ansi::BG_DARK_GRAY);
        assert_eq!(seg.bg_color_fg.as_str(), ansi::FG_DARK_GRAY);
    }

    #[test]
    fn test_segment_new_custom() {
        let seg = Segment::new("custom", ansi::FG_BLACK, ansi::BG_RED, ansi::FG_RED);
        assert_eq!(seg.text, "custom");
        assert_eq!(seg.fg.as_str(), ansi::FG_BLACK);
        assert_eq!(seg.bg.as_str(), ansi::BG_RED);
        assert_eq!(seg.bg_color_fg.as_str(), ansi::FG_RED);
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

    // --- ThemeColor ANSI conversion ---

    #[test]
    fn test_theme_color_to_ansi_bg_named() {
        use crate::config::ThemeColor;
        assert_eq!(ThemeColor::Blue.to_ansi_bg(), ansi::BG_BLUE);
        assert_eq!(ThemeColor::Green.to_ansi_bg(), ansi::BG_GREEN);
        assert_eq!(ThemeColor::Yellow.to_ansi_bg(), ansi::BG_YELLOW);
        assert_eq!(ThemeColor::Red.to_ansi_bg(), ansi::BG_RED);
        assert_eq!(ThemeColor::Cyan.to_ansi_bg(), ansi::BG_CYAN);
        assert_eq!(ThemeColor::Magenta.to_ansi_bg(), ansi::BG_MAGENTA);
        assert_eq!(ThemeColor::Gray.to_ansi_bg(), ansi::BG_GRAY);
        assert_eq!(ThemeColor::DarkGray.to_ansi_bg(), ansi::BG_DARK_GRAY);
    }

    #[test]
    fn test_theme_color_to_ansi_fg_named() {
        use crate::config::ThemeColor;
        assert_eq!(ThemeColor::Blue.to_ansi_fg(), ansi::FG_BLUE);
        assert_eq!(ThemeColor::White.to_ansi_fg(), ansi::FG_WHITE);
        assert_eq!(ThemeColor::Black.to_ansi_fg(), ansi::FG_BLACK);
    }

    #[test]
    fn test_theme_color_to_ansi_rgb() {
        use crate::config::ThemeColor;
        assert_eq!(
            ThemeColor::Rgb(0x34, 0x65, 0xA4).to_ansi_bg(),
            "\x1b[48;2;52;101;164m"
        );
        assert_eq!(
            ThemeColor::Rgb(0x34, 0x65, 0xA4).to_ansi_fg(),
            "\x1b[38;2;52;101;164m"
        );
    }

    // --- Segment::from_theme_colors ---

    #[test]
    fn test_segment_from_theme_colors() {
        use crate::config::ThemeColor;
        let seg = Segment::from_theme_colors("test", &ThemeColor::White, &ThemeColor::Blue);
        assert_eq!(seg.text, "test");
        assert_eq!(seg.fg, ThemeColor::White.to_ansi_fg());
        assert_eq!(seg.bg, ThemeColor::Blue.to_ansi_bg());
        assert_eq!(seg.bg_color_fg, ThemeColor::Blue.to_ansi_fg());
    }

    #[test]
    fn test_segment_from_theme_colors_rgb() {
        use crate::config::ThemeColor;
        let bg = ThemeColor::Rgb(0x34, 0x65, 0xA4);
        let fg = ThemeColor::White;
        let seg = Segment::from_theme_colors("dir", &fg, &bg);
        assert!(seg.bg.contains("48;2;"));
        assert!(seg.bg_color_fg.contains("38;2;"));
    }

    // --- ResolvedTheme ---

    #[test]
    fn test_resolved_theme_default() {
        use crate::config::ThemeColor;
        let theme = ResolvedTheme::default_theme();
        assert_eq!(theme.directory_bg, ThemeColor::Blue);
        assert_eq!(theme.directory_fg, ThemeColor::White);
        assert_eq!(theme.git_clean_bg, ThemeColor::Green);
        assert_eq!(theme.git_dirty_bg, ThemeColor::Yellow);
        assert_eq!(theme.tool_colors.len(), 3);
    }

    #[test]
    fn test_resolved_theme_from_preset() {
        let default = ResolvedTheme::from_preset("default");
        assert_eq!(default.directory_bg, crate::config::ThemeColor::Blue);

        let dark = ResolvedTheme::from_preset("dark");
        assert_eq!(
            dark.directory_bg,
            crate::config::ThemeColor::Rgb(0x34, 0x65, 0xA4)
        );

        let light = ResolvedTheme::from_preset("light");
        assert_eq!(
            light.directory_bg,
            crate::config::ThemeColor::Rgb(0x72, 0x9F, 0xCF)
        );

        let solarized = ResolvedTheme::from_preset("solarized");
        assert_eq!(
            solarized.directory_bg,
            crate::config::ThemeColor::Rgb(0x26, 0x8B, 0xD2)
        );

        // Unknown preset falls back to default
        let unknown = ResolvedTheme::from_preset("unknown");
        assert_eq!(unknown.directory_bg, crate::config::ThemeColor::Blue);
    }

    #[test]
    fn test_resolved_theme_from_config_preset_only() {
        use crate::config::ThemeConfig;
        let config = ThemeConfig {
            preset: "dark".to_string(),
            custom: None,
        };
        let theme = ResolvedTheme::from_config(&config);
        assert_eq!(
            theme.directory_bg,
            crate::config::ThemeColor::Rgb(0x34, 0x65, 0xA4)
        );
    }

    #[test]
    fn test_resolved_theme_from_config_with_custom_overrides() {
        use crate::config::{CustomThemeConfig, ThemeColor, ThemeConfig};
        let config = ThemeConfig {
            preset: "default".to_string(),
            custom: Some(CustomThemeConfig {
                directory_bg: Some(ThemeColor::Red),
                directory_fg: Some(ThemeColor::Black),
                ..Default::default()
            }),
        };
        let theme = ResolvedTheme::from_config(&config);
        assert_eq!(theme.directory_bg, ThemeColor::Red);
        assert_eq!(theme.directory_fg, ThemeColor::Black);
        // Non-overridden values should remain from preset
        assert_eq!(theme.git_clean_bg, ThemeColor::Green);
    }

    #[test]
    fn test_resolved_theme_custom_tool_colors() {
        use crate::config::{CustomThemeConfig, ThemeColor, ThemeConfig};
        let config = ThemeConfig {
            preset: "default".to_string(),
            custom: Some(CustomThemeConfig {
                tool_bg: Some(vec![ThemeColor::Red, ThemeColor::Blue]),
                tool_fg: Some(vec![ThemeColor::White, ThemeColor::Black]),
                ..Default::default()
            }),
        };
        let theme = ResolvedTheme::from_config(&config);
        assert_eq!(theme.tool_colors.len(), 2);
        assert_eq!(theme.tool_colors[0], (ThemeColor::Red, ThemeColor::White));
        assert_eq!(theme.tool_colors[1], (ThemeColor::Blue, ThemeColor::Black));
    }

    #[test]
    fn test_resolved_theme_custom_tool_bg_only() {
        use crate::config::{CustomThemeConfig, ThemeColor, ThemeConfig};
        let config = ThemeConfig {
            preset: "default".to_string(),
            custom: Some(CustomThemeConfig {
                tool_bg: Some(vec![ThemeColor::Red, ThemeColor::Blue]),
                // No tool_fg: should keep default fg from preset
                ..Default::default()
            }),
        };
        let theme = ResolvedTheme::from_config(&config);
        assert_eq!(theme.tool_colors.len(), 2);
        assert_eq!(theme.tool_colors[0].0, ThemeColor::Red);
        // Default preset tool_colors[0].fg is Black (cyan segment)
        assert_eq!(theme.tool_colors[0].1, ThemeColor::Black);
    }
}
