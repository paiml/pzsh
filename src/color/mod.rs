//! Color module for pzsh
//!
//! O(1) ANSI color rendering with pre-computed escape sequences.
//! Provides oh-my-zsh compatible color support while maintaining performance.

use std::fmt;

/// ANSI color codes (16-color palette)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    // Standard colors
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    // Bright colors
    BrightBlack = 8,
    BrightRed = 9,
    BrightGreen = 10,
    BrightYellow = 11,
    BrightBlue = 12,
    BrightMagenta = 13,
    BrightCyan = 14,
    BrightWhite = 15,
}

impl Color {
    /// Get foreground ANSI code
    #[must_use]
    pub const fn fg_code(self) -> u8 {
        match self as u8 {
            0..=7 => 30 + self as u8,
            _ => 90 + (self as u8 - 8),
        }
    }

    /// Get background ANSI code
    #[must_use]
    pub const fn bg_code(self) -> u8 {
        match self as u8 {
            0..=7 => 40 + self as u8,
            _ => 100 + (self as u8 - 8),
        }
    }
}

/// Text style attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Option<ColorSpec>,
    pub bg: Option<ColorSpec>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
}

/// Color specification (supports 16, 256, and true color)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpec {
    /// 16-color ANSI
    Ansi(Color),
    /// 256-color palette
    Palette(u8),
    /// True color RGB
    Rgb(u8, u8, u8),
}

impl Style {
    /// Create a new empty style
    #[must_use]
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
        }
    }

    /// Set foreground color
    #[must_use]
    pub const fn fg(mut self, color: ColorSpec) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set foreground to ANSI color
    #[must_use]
    pub const fn fg_ansi(mut self, color: Color) -> Self {
        self.fg = Some(ColorSpec::Ansi(color));
        self
    }

    /// Set background color
    #[must_use]
    pub const fn bg(mut self, color: ColorSpec) -> Self {
        self.bg = Some(color);
        self
    }

    /// Set background to ANSI color
    #[must_use]
    pub const fn bg_ansi(mut self, color: Color) -> Self {
        self.bg = Some(ColorSpec::Ansi(color));
        self
    }

    /// Set bold
    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Set dim
    #[must_use]
    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Set italic
    #[must_use]
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Set underline
    #[must_use]
    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Generate ANSI escape sequence for this style
    /// Returns empty string if no styling applied
    #[must_use]
    pub fn to_ansi(&self) -> String {
        if self.fg.is_none() && self.bg.is_none() && !self.bold && !self.dim && !self.italic && !self.underline {
            return String::new();
        }

        let mut codes = Vec::with_capacity(8);

        if self.bold {
            codes.push("1".to_string());
        }
        if self.dim {
            codes.push("2".to_string());
        }
        if self.italic {
            codes.push("3".to_string());
        }
        if self.underline {
            codes.push("4".to_string());
        }

        if let Some(fg) = &self.fg {
            match fg {
                ColorSpec::Ansi(c) => codes.push(c.fg_code().to_string()),
                ColorSpec::Palette(n) => codes.push(format!("38;5;{n}")),
                ColorSpec::Rgb(r, g, b) => codes.push(format!("38;2;{r};{g};{b}")),
            }
        }

        if let Some(bg) = &self.bg {
            match bg {
                ColorSpec::Ansi(c) => codes.push(c.bg_code().to_string()),
                ColorSpec::Palette(n) => codes.push(format!("48;5;{n}")),
                ColorSpec::Rgb(r, g, b) => codes.push(format!("48;2;{r};{g};{b}")),
            }
        }

        format!("\x1b[{}m", codes.join(";"))
    }
}

/// ANSI reset sequence
pub const RESET: &str = "\x1b[0m";

/// Styled text wrapper for O(1) rendering
#[derive(Debug, Clone)]
pub struct Styled {
    pub text: String,
    pub style: Style,
}

impl Styled {
    /// Create new styled text
    #[must_use]
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    /// Create unstyled text
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::new(),
        }
    }

    /// Render to string with ANSI codes
    #[must_use]
    pub fn render(&self) -> String {
        let prefix = self.style.to_ansi();
        if prefix.is_empty() {
            self.text.clone()
        } else {
            format!("{prefix}{}{RESET}", self.text)
        }
    }
}

impl fmt::Display for Styled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

/// Pre-defined color themes for oh-my-zsh compatibility
pub mod themes {
    use super::*;

    /// Default theme colors
    pub struct DefaultTheme;

    impl DefaultTheme {
        /// User segment style
        #[must_use]
        pub const fn user() -> Style {
            Style::new().fg_ansi(Color::Green).bold()
        }

        /// Host segment style
        #[must_use]
        pub const fn host() -> Style {
            Style::new().fg_ansi(Color::Blue).bold()
        }

        /// Directory segment style
        #[must_use]
        pub const fn cwd() -> Style {
            Style::new().fg_ansi(Color::Cyan)
        }

        /// Git clean branch style
        #[must_use]
        pub const fn git_clean() -> Style {
            Style::new().fg_ansi(Color::Green)
        }

        /// Git dirty branch style
        #[must_use]
        pub const fn git_dirty() -> Style {
            Style::new().fg_ansi(Color::Yellow)
        }

        /// Error/failure style
        #[must_use]
        pub const fn error() -> Style {
            Style::new().fg_ansi(Color::Red).bold()
        }

        /// Success style
        #[must_use]
        pub const fn success() -> Style {
            Style::new().fg_ansi(Color::Green)
        }

        /// Warning style
        #[must_use]
        pub const fn warning() -> Style {
            Style::new().fg_ansi(Color::Yellow)
        }

        /// Prompt char (normal user)
        #[must_use]
        pub const fn prompt_char() -> Style {
            Style::new().fg_ansi(Color::White).bold()
        }

        /// Prompt char (root)
        #[must_use]
        pub const fn prompt_root() -> Style {
            Style::new().fg_ansi(Color::Red).bold()
        }
    }
}

/// Check if terminal supports colors
#[must_use]
pub fn supports_color() -> bool {
    // Check TERM environment variable
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Check NO_COLOR environment variable (https://no-color.org/)
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check CLICOLOR_FORCE
    if std::env::var("CLICOLOR_FORCE").is_ok() {
        return true;
    }

    // Default: assume color support in modern terminals
    true
}

/// Check if terminal supports true color (24-bit)
#[must_use]
pub fn supports_true_color() -> bool {
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        return colorterm == "truecolor" || colorterm == "24bit";
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_fg_codes() {
        assert_eq!(Color::Black.fg_code(), 30);
        assert_eq!(Color::Red.fg_code(), 31);
        assert_eq!(Color::White.fg_code(), 37);
        assert_eq!(Color::BrightBlack.fg_code(), 90);
        assert_eq!(Color::BrightWhite.fg_code(), 97);
    }

    #[test]
    fn test_color_bg_codes() {
        assert_eq!(Color::Black.bg_code(), 40);
        assert_eq!(Color::Red.bg_code(), 41);
        assert_eq!(Color::BrightRed.bg_code(), 101);
    }

    #[test]
    fn test_style_empty() {
        let style = Style::new();
        assert_eq!(style.to_ansi(), "");
    }

    #[test]
    fn test_style_fg_only() {
        let style = Style::new().fg_ansi(Color::Red);
        assert_eq!(style.to_ansi(), "\x1b[31m");
    }

    #[test]
    fn test_style_bold() {
        let style = Style::new().bold();
        assert_eq!(style.to_ansi(), "\x1b[1m");
    }

    #[test]
    fn test_style_combined() {
        let style = Style::new().fg_ansi(Color::Green).bold();
        assert_eq!(style.to_ansi(), "\x1b[1;32m");
    }

    #[test]
    fn test_style_256_color() {
        let style = Style::new().fg(ColorSpec::Palette(196));
        assert_eq!(style.to_ansi(), "\x1b[38;5;196m");
    }

    #[test]
    fn test_style_true_color() {
        let style = Style::new().fg(ColorSpec::Rgb(255, 128, 64));
        assert_eq!(style.to_ansi(), "\x1b[38;2;255;128;64m");
    }

    #[test]
    fn test_styled_render() {
        let styled = Styled::new("hello", Style::new().fg_ansi(Color::Red));
        assert_eq!(styled.render(), "\x1b[31mhello\x1b[0m");
    }

    #[test]
    fn test_styled_plain() {
        let styled = Styled::plain("hello");
        assert_eq!(styled.render(), "hello");
    }

    #[test]
    fn test_theme_styles() {
        // Just verify they compile and produce output
        let user_style = themes::DefaultTheme::user();
        assert!(user_style.bold);
        assert!(user_style.fg.is_some());

        let git_dirty = themes::DefaultTheme::git_dirty();
        assert!(git_dirty.fg.is_some());
    }

    #[test]
    fn test_render_performance() {
        // Ensure rendering is fast (O(1))
        let style = Style::new()
            .fg_ansi(Color::Green)
            .bg_ansi(Color::Black)
            .bold()
            .italic()
            .underline();

        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = style.to_ansi();
        }
        let elapsed = start.elapsed();

        // Should complete 10k iterations reasonably fast
        // Allow 100ms for coverage/debug builds
        assert!(
            elapsed < std::time::Duration::from_millis(100),
            "Style rendering too slow: {:?}",
            elapsed
        );
    }
}
