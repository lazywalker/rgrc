//! Zero-dependency ANSI styling: 8 colors, bright variants, text attributes.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    fg_color: Option<Color>,
    bg_color: Option<Color>,
    bold: bool,
    dim: bool,
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
    Ansi256(u8),
    Rgb(u8, u8, u8),
}

impl Style {
    #[inline]
    pub const fn new() -> Self {
        Style {
            fg_color: None,
            bg_color: None,
            bold: false,
            dim: false,
            underlined: false,
            italic: false,
            blink: false,
            reverse: false,
            bright: false,
        }
    }

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

    #[inline]
    pub const fn ansi256(mut self, n: u8) -> Self {
        self.fg_color = Some(Color::Ansi256(n));
        self
    }

    #[inline]
    pub const fn rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.fg_color = Some(Color::Rgb(r, g, b));
        self
    }

    #[inline]
    pub const fn on_ansi256(mut self, n: u8) -> Self {
        self.bg_color = Some(Color::Ansi256(n));
        self
    }

    #[inline]
    pub const fn on_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.bg_color = Some(Color::Rgb(r, g, b));
        self
    }

    #[inline]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    #[allow(dead_code)]
    #[inline]
    pub const fn dim(mut self) -> Self {
        self.dim = true;
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

    pub fn apply_to<'a>(&self, text: &'a str) -> StyledText<'a> {
        StyledText { text, style: *self }
    }

    fn to_ansi_codes(self) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut codes: Vec<String> = Vec::new();

        if self.bold {
            codes.push("1".to_string());
        }
        if self.dim {
            codes.push("2".to_string());
        }
        if self.italic {
            codes.push("3".to_string());
        }
        if self.underlined {
            codes.push("4".to_string());
        }
        if self.blink {
            codes.push("5".to_string());
        }
        if self.reverse {
            codes.push("7".to_string());
        }

        if let Some(fg) = self.fg_color {
            match fg {
                Color::Black if self.bright => codes.push("90".to_string()),
                Color::Black => codes.push("30".to_string()),
                Color::Red if self.bright => codes.push("91".to_string()),
                Color::Green if self.bright => codes.push("92".to_string()),
                Color::Yellow if self.bright => codes.push("93".to_string()),
                Color::Blue if self.bright => codes.push("94".to_string()),
                Color::Magenta if self.bright => codes.push("95".to_string()),
                Color::Cyan if self.bright => codes.push("96".to_string()),
                Color::White if self.bright => codes.push("97".to_string()),
                Color::Red => codes.push("31".to_string()),
                Color::Green => codes.push("32".to_string()),
                Color::Yellow => codes.push("33".to_string()),
                Color::Blue => codes.push("34".to_string()),
                Color::Magenta => codes.push("35".to_string()),
                Color::Cyan => codes.push("36".to_string()),
                Color::White => codes.push("37".to_string()),
                Color::Ansi256(n) => codes.push(format!("38;5;{}", n)),
                Color::Rgb(r, g, b) => codes.push(format!("38;2;{};{};{}", r, g, b)),
            }
        }

        if let Some(bg) = self.bg_color {
            match bg {
                Color::Black => codes.push("40".to_string()),
                Color::Red => codes.push("41".to_string()),
                Color::Green => codes.push("42".to_string()),
                Color::Yellow => codes.push("43".to_string()),
                Color::Blue => codes.push("44".to_string()),
                Color::Magenta => codes.push("45".to_string()),
                Color::Cyan => codes.push("46".to_string()),
                Color::White => codes.push("47".to_string()),
                Color::Ansi256(n) => codes.push(format!("48;5;{}", n)),
                Color::Rgb(r, g, b) => codes.push(format!("48;2;{};{};{}", r, g, b)),
            }
        }

        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    const fn is_empty(&self) -> bool {
        self.fg_color.is_none()
            && self.bg_color.is_none()
            && !self.bold
            && !self.dim
            && !self.underlined
            && !self.italic
            && !self.blink
            && !self.reverse
    }
}

pub struct StyledText<'a> {
    text: &'a str,
    style: Style,
}

impl<'a> fmt::Display for StyledText<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.style.is_empty() {
            // no styling, passthrough
            write!(f, "{}", self.text)
        } else {
            // SGR codes + text + reset
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
        let style = Style::new().bright().black();
        assert_eq!(style.to_ansi_codes(), "\x1b[90m");

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

    #[test]
    fn ansi256_fg() {
        let style = Style::new().ansi256(140);
        assert_eq!(style.to_ansi_codes(), "\x1b[38;5;140m");
    }

    #[test]
    fn rgb_fg() {
        let style = Style::new().rgb(255, 136, 0);
        assert_eq!(style.to_ansi_codes(), "\x1b[38;2;255;136;0m");
    }

    #[test]
    fn ansi256_bg() {
        let style = Style::new().on_ansi256(140);
        assert_eq!(style.to_ansi_codes(), "\x1b[48;5;140m");
    }

    #[test]
    fn rgb_bg() {
        let style = Style::new().on_rgb(255, 136, 0);
        assert_eq!(style.to_ansi_codes(), "\x1b[48;2;255;136;0m");
    }

    #[test]
    fn ansi256_with_bold() {
        let style = Style::new().bold().ansi256(140);
        assert_eq!(style.to_ansi_codes(), "\x1b[1;38;5;140m");
    }
}
