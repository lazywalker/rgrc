//! Lightweight ANSI style implementation
//!
//! This module provides a minimal replacement for the `console` crate's styling functionality.
//! It supports all the color and text attribute features needed by rgrc while keeping the
//! implementation simple and dependency-free.

use std::fmt;

/// ANSI style builder for terminal colors and text attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    fg_color: Option<Color>,
    bg_color: Option<Color>,
    bold: bool,
    underlined: bool,
    italic: bool,
    blink: bool,
    reverse: bool,
    bright: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Style {
    /// Create a new empty style with no formatting
    #[inline]
    pub const fn new() -> Self {
        Style {
            fg_color: None,
            bg_color: None,
            bold: false,
            underlined: false,
            italic: false,
            blink: false,
            reverse: false,
            bright: false,
        }
    }

    // Foreground colors
    #[inline]
    pub const fn black(mut self) -> Self {
        self.fg_color = Some(Color::Black);
        self
    }

    #[inline]
    pub const fn red(mut self) -> Self {
        self.fg_color = Some(Color::Red);
        self
    }

    #[inline]
    pub const fn green(mut self) -> Self {
        self.fg_color = Some(Color::Green);
        self
    }

    #[inline]
    pub const fn yellow(mut self) -> Self {
        self.fg_color = Some(Color::Yellow);
        self
    }

    #[inline]
    pub const fn blue(mut self) -> Self {
        self.fg_color = Some(Color::Blue);
        self
    }

    #[inline]
    pub const fn magenta(mut self) -> Self {
        self.fg_color = Some(Color::Magenta);
        self
    }

    #[inline]
    pub const fn cyan(mut self) -> Self {
        self.fg_color = Some(Color::Cyan);
        self
    }

    #[inline]
    pub const fn white(mut self) -> Self {
        self.fg_color = Some(Color::White);
        self
    }

    // Background colors
    #[inline]
    pub const fn on_black(mut self) -> Self {
        self.bg_color = Some(Color::Black);
        self
    }

    #[inline]
    pub const fn on_red(mut self) -> Self {
        self.bg_color = Some(Color::Red);
        self
    }

    #[inline]
    pub const fn on_green(mut self) -> Self {
        self.bg_color = Some(Color::Green);
        self
    }

    #[inline]
    pub const fn on_yellow(mut self) -> Self {
        self.bg_color = Some(Color::Yellow);
        self
    }

    #[inline]
    pub const fn on_blue(mut self) -> Self {
        self.bg_color = Some(Color::Blue);
        self
    }

    #[inline]
    pub const fn on_magenta(mut self) -> Self {
        self.bg_color = Some(Color::Magenta);
        self
    }

    #[inline]
    pub const fn on_cyan(mut self) -> Self {
        self.bg_color = Some(Color::Cyan);
        self
    }

    #[inline]
    pub const fn on_white(mut self) -> Self {
        self.bg_color = Some(Color::White);
        self
    }

    // Text attributes
    #[inline]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    #[inline]
    pub const fn underlined(mut self) -> Self {
        self.underlined = true;
        self
    }

    #[inline]
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    #[inline]
    pub const fn blink(mut self) -> Self {
        self.blink = true;
        self
    }

    #[inline]
    pub const fn reverse(mut self) -> Self {
        self.reverse = true;
        self
    }

    #[inline]
    pub const fn bright(mut self) -> Self {
        self.bright = true;
        self
    }

    /// Apply this style to a string, returning a formatted wrapper
    pub fn apply_to<'a>(&self, text: &'a str) -> StyledText<'a> {
        StyledText { text, style: *self }
    }

    /// Generate ANSI escape codes for this style
    fn to_ansi_codes(self) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut codes = Vec::new();

        // Text attributes
        if self.bold {
            codes.push("1");
        }
        if self.italic {
            codes.push("3");
        }
        if self.underlined {
            codes.push("4");
        }
        if self.blink {
            codes.push("5");
        }
        if self.reverse {
            codes.push("7");
        }

        // Foreground color
        if let Some(fg) = self.fg_color {
            codes.push(match fg {
                Color::Black => "30",
                Color::Red if self.bright => "91",
                Color::Green if self.bright => "92",
                Color::Yellow if self.bright => "93",
                Color::Blue if self.bright => "94",
                Color::Magenta if self.bright => "95",
                Color::Cyan if self.bright => "96",
                Color::White if self.bright => "97",
                Color::Red => "31",
                Color::Green => "32",
                Color::Yellow => "33",
                Color::Blue => "34",
                Color::Magenta => "35",
                Color::Cyan => "36",
                Color::White => "37",
            });
        }

        // Background color
        if let Some(bg) = self.bg_color {
            codes.push(match bg {
                Color::Black => "40",
                Color::Red => "41",
                Color::Green => "42",
                Color::Yellow => "43",
                Color::Blue => "44",
                Color::Magenta => "45",
                Color::Cyan => "46",
                Color::White => "47",
            });
        }

        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    /// Check if this style has any formatting
    const fn is_empty(&self) -> bool {
        self.fg_color.is_none()
            && self.bg_color.is_none()
            && !self.bold
            && !self.underlined
            && !self.italic
            && !self.blink
            && !self.reverse
    }
}

/// Wrapper for styled text that implements Display
pub struct StyledText<'a> {
    text: &'a str,
    style: Style,
}

impl<'a> fmt::Display for StyledText<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.style.is_empty() {
            // No styling - just write the text
            write!(f, "{}", self.text)
        } else {
            // Write: ANSI codes + text + reset
            write!(f, "{}{}\x1b[0m", self.style.to_ansi_codes(), self.text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_colors() {
        let style = Style::new().red();
        assert_eq!(style.to_ansi_codes(), "\x1b[31m");

        let style = Style::new().green();
        assert_eq!(style.to_ansi_codes(), "\x1b[32m");

        let style = Style::new().blue();
        assert_eq!(style.to_ansi_codes(), "\x1b[34m");
    }

    #[test]
    fn test_bright_colors() {
        let style = Style::new().bright().red();
        assert_eq!(style.to_ansi_codes(), "\x1b[91m");

        let style = Style::new().bright().green();
        assert_eq!(style.to_ansi_codes(), "\x1b[92m");
    }

    #[test]
    fn test_background_colors() {
        let style = Style::new().on_red();
        assert_eq!(style.to_ansi_codes(), "\x1b[41m");

        let style = Style::new().on_green();
        assert_eq!(style.to_ansi_codes(), "\x1b[42m");
    }

    #[test]
    fn test_text_attributes() {
        let style = Style::new().bold();
        assert_eq!(style.to_ansi_codes(), "\x1b[1m");

        let style = Style::new().underlined();
        assert_eq!(style.to_ansi_codes(), "\x1b[4m");

        let style = Style::new().italic();
        assert_eq!(style.to_ansi_codes(), "\x1b[3m");
    }

    #[test]
    fn test_combined_styles() {
        let style = Style::new().bold().red();
        assert_eq!(style.to_ansi_codes(), "\x1b[1;31m");

        let style = Style::new().bold().underlined().green();
        assert_eq!(style.to_ansi_codes(), "\x1b[1;4;32m");

        let style = Style::new().red().on_blue();
        assert_eq!(style.to_ansi_codes(), "\x1b[31;44m");
    }

    #[test]
    fn test_apply_to() {
        let style = Style::new().red();
        let styled = style.apply_to("hello");
        assert_eq!(format!("{}", styled), "\x1b[31mhello\x1b[0m");
    }

    #[test]
    fn test_empty_style() {
        let style = Style::new();
        assert_eq!(style.to_ansi_codes(), "");

        let styled = style.apply_to("hello");
        assert_eq!(format!("{}", styled), "hello");
    }
}
