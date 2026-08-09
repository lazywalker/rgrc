//! Parse grc.conf (command to config mapping) and grcat config files
//! (regex + style rules). Uses a hybrid regex engine: standard `regex_lite`
//! for simple patterns, fancy-regex or EnhancedRegex for lookarounds.

use std::io::{BufRead, Lines};

#[cfg(not(feature = "fancy-regex"))]
use crate::enhanced_regex::EnhancedRegex;
use crate::style::Style;
#[cfg(feature = "fancy-regex")]
use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use regex_lite as regex;

/// Custom error type for regex compilation
#[derive(Debug)]
pub enum RegexError {
    Syntax(String),
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexError::Syntax(msg) => write!(f, "Regex syntax error: {}", msg),
        }
    }
}

impl std::error::Error for RegexError {}

impl From<regex::Error> for RegexError {
    fn from(err: regex::Error) -> Self {
        RegexError::Syntax(err.to_string())
    }
}

/// Hybrid regex: `Fast` for simple patterns, `Enhanced` for lookarounds.
#[derive(Debug, Clone)]
pub enum CompiledRegex {
    Fast(Regex),
    #[cfg(feature = "fancy-regex")]
    Enhanced(FancyRegex),
    #[cfg(not(feature = "fancy-regex"))]
    Enhanced(EnhancedRegex),
}

impl CompiledRegex {
    /// Compile a regex pattern, automatically selecting the fastest engine.
    /// Tries standard regex first, then falls back to EnhancedRegex for lookaround patterns.
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        // Try standard regex first (fastest, but no lookaround)
        if let Ok(re) = Regex::new(pattern) {
            return Ok(CompiledRegex::Fast(re));
        }

        // Fall back to Enhanced regex implementation
        #[cfg(feature = "fancy-regex")]
        {
            // Use battle-tested fancy-regex when enabled
            FancyRegex::new(pattern)
                .map(CompiledRegex::Enhanced)
                .map_err(|e| RegexError::Syntax(e.to_string()))
        }
        #[cfg(not(feature = "fancy-regex"))]
        {
            // Use our own EnhancedRegex implementation (default)
            EnhancedRegex::new(pattern)
                .map(CompiledRegex::Enhanced)
                .map_err(RegexError::from)
        }
    }

    /// Check if the regex matches anywhere in the text.
    #[allow(dead_code)]
    pub fn is_match(&self, text: &str) -> bool {
        match self {
            CompiledRegex::Fast(re) => re.is_match(text),
            #[cfg(feature = "fancy-regex")]
            CompiledRegex::Enhanced(re) => re.is_match(text).unwrap_or(false),
            #[cfg(not(feature = "fancy-regex"))]
            CompiledRegex::Enhanced(re) => re.is_match(text),
        }
    }

    /// Find all capture groups starting from the given position.
    #[allow(dead_code)]
    pub fn captures_from_pos<'t>(&self, text: &'t str, pos: usize) -> Option<Captures<'t>> {
        match self {
            CompiledRegex::Fast(re) => {
                // Standard regex: convert to our Captures format
                re.captures(&text[pos..])
                    .map(|caps| Captures::Fast(caps, pos))
            }
            #[cfg(feature = "fancy-regex")]
            CompiledRegex::Enhanced(re) => {
                // fancy-regex: convert to our Captures format
                re.captures(&text[pos..])
                    .ok()
                    .flatten()
                    .map(|caps| Captures::Fancy(caps, pos))
            }
            #[cfg(not(feature = "fancy-regex"))]
            CompiledRegex::Enhanced(re) => {
                // EnhancedRegex: convert to our Captures format
                re.captures_from_pos(text, pos)
                    .map(|caps| Captures::Fast(caps, 0))
            }
        }
    }

    /// Get the pattern string for debugging.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            CompiledRegex::Fast(re) => re.as_str(),
            #[cfg(feature = "fancy-regex")]
            CompiledRegex::Enhanced(re) => re.as_str(),
            #[cfg(not(feature = "fancy-regex"))]
            CompiledRegex::Enhanced(re) => re.as_str(),
        }
    }
}

/// Unified captures interface wrapping regex::Captures.
#[derive(Debug)]
#[allow(dead_code)]
pub enum Captures<'t> {
    Fast(regex::Captures<'t>, usize), // offset for position adjustment
    #[cfg(feature = "fancy-regex")]
    Fancy(fancy_regex::Captures<'t>, usize), // fancy-regex captures with offset
}

impl<'t> Captures<'t> {
    /// Get a capture group by index (0 = full match, 1+ = groups).
    #[allow(dead_code)]
    pub fn get(&self, index: usize) -> Option<Match<'t>> {
        match self {
            Captures::Fast(caps, offset) => caps.get(index).map(|m| Match::Fast(m, *offset)),
            #[cfg(feature = "fancy-regex")]
            Captures::Fancy(caps, offset) => caps.get(index).map(|m| Match::Fancy(m, *offset)),
        }
    }

    /// Get the number of capture groups.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        match self {
            Captures::Fast(caps, _) => caps.len(),
            #[cfg(feature = "fancy-regex")]
            Captures::Fancy(caps, _) => caps.len(),
        }
    }

    /// Check if there are no capture groups.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over all capture groups by index.
    /// Returns a vector to avoid lifetime issues with closures.
    #[allow(dead_code)]
    pub fn iter(&'t self) -> Vec<Option<Match<'t>>> {
        let len = self.len();
        (0..len).map(|i| self.get(i)).collect()
    }
}

/// Unified match interface wrapping regex::Match.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Match<'t> {
    Fast(regex::Match<'t>, usize), // offset for position adjustment
    #[cfg(feature = "fancy-regex")]
    Fancy(fancy_regex::Match<'t>, usize), // fancy-regex match with offset
}

impl<'t> Match<'t> {
    /// Get the start byte position of the match.
    #[allow(dead_code)]
    pub fn start(&self) -> usize {
        match self {
            Match::Fast(m, offset) => m.start() + offset,
            #[cfg(feature = "fancy-regex")]
            Match::Fancy(m, offset) => m.start() + offset,
        }
    }

    /// Get the end byte position of the match.
    #[allow(dead_code)]
    pub fn end(&self) -> usize {
        match self {
            Match::Fast(m, offset) => m.end() + offset,
            #[cfg(feature = "fancy-regex")]
            Match::Fancy(m, offset) => m.end() + offset,
        }
    }

    /// Get the matched text as a string slice.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'t str {
        match self {
            Match::Fast(m, _) => m.as_str(),
            #[cfg(feature = "fancy-regex")]
            Match::Fancy(m, _) => m.as_str(),
        }
    }
}

/// Parse space-separated style keywords into a Style.
pub fn style_from_str(text: &str) -> Result<Style, String> {
    text.split(' ').try_fold(Style::new(), |style, word| {
        if word.starts_with('"') && word.contains("\\033[") {
            return Ok(parse_raw_ansi(word).unwrap_or(style));
        }
        match word {
            "" => Ok(style),
            "unchanged" => Ok(style),
            "default" => Ok(style),
            "dark" => Ok(style.dim()),
            "none" => Ok(style),

            "black" => Ok(style.black()),
            "red" => Ok(style.red()),
            "green" => Ok(style.green()),
            "yellow" => Ok(style.yellow()),
            "blue" => Ok(style.blue()),
            "magenta" => Ok(style.magenta()),
            "cyan" => Ok(style.cyan()),
            "white" => Ok(style.white()),

            "on_black" => Ok(style.on_black()),
            "on_red" => Ok(style.on_red()),
            "on_green" => Ok(style.on_green()),
            "on_yellow" => Ok(style.on_yellow()),
            "on_blue" => Ok(style.on_blue()),
            "on_magenta" => Ok(style.on_magenta()),
            "on_cyan" => Ok(style.on_cyan()),
            "on_white" => Ok(style.on_white()),

            "bold" => Ok(style.bold()),
            "underline" => Ok(style.underlined()),
            "italic" => Ok(style.italic()),
            "blink" => Ok(style.blink()),
            "reverse" => Ok(style.reverse()),
            "dim" => Ok(style.dim()),

            "bright_black" => Ok(style.bright().black()),
            "bright_red" => Ok(style.bright().red()),
            "bright_green" => Ok(style.bright().green()),
            "bright_yellow" => Ok(style.bright().yellow()),
            "bright_blue" => Ok(style.bright().blue()),
            "bright_magenta" => Ok(style.bright().magenta()),
            "bright_cyan" => Ok(style.bright().cyan()),
            "bright_white" => Ok(style.bright().white()),

            w if w.starts_with("colour_") || w.starts_with("color_") => {
                let n = w
                    .split('_')
                    .nth(1)
                    .and_then(|s| s.parse::<u8>().ok())
                    .ok_or_else(|| format!("bad colour index: {}", w))?;
                Ok(style.ansi256(n))
            }
            w if w.starts_with("on_colour_") || w.starts_with("on_color_") => {
                let n = w
                    .split('_')
                    .nth(2)
                    .and_then(|s| s.parse::<u8>().ok())
                    .ok_or_else(|| format!("bad colour index: {}", w))?;
                Ok(style.on_ansi256(n))
            }
            w if w.starts_with("rgb:") => {
                parse_hex_rgb(&w[4..]).map(|(r, g, b)| style.rgb(r, g, b))
            }
            w if w.starts_with("on_rgb:") => {
                parse_hex_rgb(&w[7..]).map(|(r, g, b)| style.on_rgb(r, g, b))
            }

            _ => {
                let msg = format!("unhandled style: {}", word);
                eprintln!("{}", msg);
                Err(msg)
            }
        }
    })
}

fn parse_hex_rgb(hex: &str) -> Result<(u8, u8, u8), String> {
    if hex.len() != 6 {
        return Err(format!("bad rgb hex: {}", hex));
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| format!("bad rgb hex: {}", hex))?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| format!("bad rgb hex: {}", hex))?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| format!("bad rgb hex: {}", hex))?;
    Ok((r, g, b))
}

// parse a literal "\033[...m" token from a quoted colours value into a Style.
// supports 256-color (38;5;N / 48;5;N) and truecolor (38;2;R;G;B / 48;2;R;G;B).
fn parse_raw_ansi(token: &str) -> Option<Style> {
    let inner = token.trim_matches('"');
    // strip the leading escape: either literal "\033[" or a real ESC
    let params = inner
        .strip_prefix("\\033[")
        .or_else(|| inner.strip_prefix('\x1b').and_then(|s| s.strip_prefix('[')))?;
    let params = params.trim_end_matches('m');
    let nums: Vec<u32> = params.split(';').filter_map(|s| s.parse().ok()).collect();

    match nums.as_slice() {
        [38, 5, n] => Some(Style::new().ansi256(*n as u8)),
        [48, 5, n] => Some(Style::new().on_ansi256(*n as u8)),
        [38, 2, r, g, b] => Some(Style::new().rgb(*r as u8, *g as u8, *b as u8)),
        [48, 2, r, g, b] => Some(Style::new().on_rgb(*r as u8, *g as u8, *b as u8)),
        _ => None,
    }
}

/// Parse comma-separated styles (one per capture group), e.g. "bold red,yellow".
#[allow(dead_code)]
pub fn styles_from_str(text: &str) -> Result<Vec<Style>, String> {
    text.split(',').map(style_from_str).collect()
}

/// Iterator over grc.conf rules: yields (regex, config_file) pairs.
#[allow(dead_code)]
pub struct GrcConfigReader<A> {
    inner: Lines<A>,
}

#[allow(dead_code)]
impl<A: BufRead> GrcConfigReader<A> {
    /// Create a new GRC configuration reader from a line iterator.
    ///
    /// # Arguments
    ///
    /// * `inner` - A `Lines<A>` iterator yielding lines from a buffered reader
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// let file = File::open("~/.config/rgrc/grc.conf")?;
    /// let reader = BufReader::new(file);
    /// let config_reader = GrcConfigReader::new(reader.lines());
    /// ```
    pub fn new(inner: Lines<A>) -> Self {
        GrcConfigReader { inner }
    }

    /// Skip to the next non-empty, non-comment line.
    ///
    /// This helper method iterates through the input lines and returns the first line that is:
    /// - Not a comment (does not start with '#')
    /// - Not empty or whitespace-only
    /// - Not a line where whitespace precedes a comment character
    ///
    /// Comments are detected using the regex pattern `^[- \t]*(#|$)` which matches:
    /// - Lines starting with optional whitespace/dashes followed by '#' (comments)
    /// - Lines that are empty or whitespace-only (matches end of line via `$`)
    ///
    /// ## Returns
    ///
    /// - `Some(String)` - Trimmed content of the next valid line
    /// - `None` - EOF reached, no more content lines available
    ///
    /// ## Error Handling
    ///
    /// If a line read error occurs, iteration stops and returns None.
    ///
    /// # Examples
    ///
    /// With input:
    /// ```text
    /// # This is a comment
    ///
    /// ^ping
    /// conf.ping
    /// ```
    /// `next_content_line()` will skip the comment and blank line, returning `"^ping"`
    fn next_content_line(&mut self) -> Option<String> {
        // Regex pattern explanation:
        // ^[- \t]*(#|$)
        // - ^       : Start of line
        // - [- \t]* : Zero or more dashes, spaces, or tabs
        // - (#|$)   : Either a hash (comment start) or end of line (empty/whitespace only)
        //
        // This matches:
        // - Comment lines: "# comment" or "  # comment"
        // - Empty lines: "" or "   " (just whitespace)
        // But NOT:
        // - "^ping" (regex line)
        // - "conf.ping" (config path line)
        let re = Regex::new("^[- \t]*(#|$)").unwrap();
        for line in &mut self.inner {
            match line {
                Ok(line2) => {
                    // If line doesn't match the comment/empty pattern, it's a content line
                    if !re.is_match(&line2) {
                        return Some(line2.trim().to_string());
                    }
                }
                Err(_) => break, // Stop on read error
            }
        }
        None // No more content lines (EOF)
    }
}

/// Iterator yielding `(regex, config_file)` pairs from grc.conf.
impl<A: BufRead> Iterator for GrcConfigReader<A> {
    type Item = (CompiledRegex, String);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(regexp) = self.next_content_line() {
            if let Some(filename) = self.next_content_line() {
                match CompiledRegex::new(&regexp) {
                    Ok(re) => Some((re, filename)),
                    Err(_) => self.next(),
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Iterator over grcat config entries (regexp + colours + count + replace).
#[allow(dead_code)]
pub struct GrcatConfigReader<A> {
    inner: Lines<A>,
}

#[allow(dead_code)]
impl<A: BufRead> GrcatConfigReader<A> {
    /// Create a new grcat configuration reader from a line iterator.
    ///
    /// # Arguments
    ///
    /// * `inner` - A `Lines<A>` iterator yielding lines from a buffered reader
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// let file = File::open("~/.config/rgrc/conf.ping")?;
    /// let reader = BufReader::new(file);
    /// let grcat_reader = GrcatConfigReader::new(reader.lines());
    /// ```
    pub fn new(inner: Lines<A>) -> Self {
        GrcatConfigReader { inner }
    }

    /// Fetch the next alphanumeric line (skipping comments/blank lines).
    ///
    /// In grcat format, configuration entries start with alphanumeric characters (a-zA-Z0-9).
    /// All other lines (comments, blank lines) are ignored. This method is used to find the
    /// start of a new configuration entry.
    ///
    /// The regex pattern `^[a-zA-Z0-9]` matches lines that start with alphanumeric characters,
    /// which indicates the beginning of a key=value line.
    ///
    /// ## Returns
    ///
    /// - `Some(String)` - Next line starting with alphanumeric character (trimmed)
    /// - `None` - EOF reached, no more entries
    ///
    /// ## Error Handling
    ///
    /// If a line read error occurs, iteration stops and returns None.
    ///
    /// # Examples
    ///
    /// With input:
    /// ```text
    /// # Comment line
    ///
    /// regexp=^ERROR
    /// colours=bold red
    ///
    /// regexp=^WARN
    /// ```
    /// `next_alphanumeric()` will skip comments and blanks, returning:
    /// 1. `"regexp=^ERROR"`
    /// 2. `"colours=bold red"`
    /// 3. `"regexp=^WARN"`
    fn next_alphanumeric(&mut self) -> Option<String> {
        // Pattern ^[a-zA-Z0-9] matches lines starting with a letter or digit
        let alphanumeric = Regex::new("^[a-zA-Z0-9]").unwrap();
        for line in (&mut self.inner).flatten() {
            // Skip non-matching lines (comments, blanks)
            if alphanumeric.is_match(&line) {
                return Some(line.trim().to_string());
            }
        }
        None // No more alphanumeric lines (EOF)
    }

    /// Fetch the next line if it's alphanumeric, or None to signal end of entry.
    ///
    /// This method is used during entry parsing to continue reading key=value pairs
    /// that belong to the current configuration entry. As long as lines start with
    /// alphanumeric characters, they belong to the same entry. When a non-alphanumeric
    /// line is encountered (comment, blank line), it signals the end of the current
    /// entry, and the method returns None.
    ///
    /// ## Returns
    ///
    /// - `Some(String)` - Next line if it starts with alphanumeric (still in this entry)
    /// - `None` - End of entry or EOF (non-alphanumeric line or no more input)
    ///
    /// ## Implementation Details
    ///
    /// 1. Calls `self.inner.next()` to get the next line from the buffer
    /// 2. If EOF, returns None (no next line available)
    /// 3. If line starts with alphanumeric, returns it (still in entry)
    /// 4. If line doesn't start with alphanumeric, returns None (end of entry)
    ///
    /// ## Entry Boundary Detection
    ///
    /// This method implements entry boundary detection:
    /// - Alphanumeric start: still in current entry
    /// - Non-alphanumeric start: end of entry
    /// - Examples of entry-ending lines:
    ///   - Blank line: ""
    ///   - Comment: "# This is a comment"
    ///   - Whitespace: "   "
    ///   - Any line starting with non-alphanumeric: "---", "$", etc.
    fn following(&mut self) -> Option<String> {
        // Pattern ^[a-zA-Z0-9] matches lines starting with a letter or digit
        let alphanumeric = Regex::new("^[a-zA-Z0-9]").unwrap();
        if let Some(Ok(line)) = self.inner.next() {
            // If line starts with alphanumeric, it's part of this entry
            if alphanumeric.is_match(&line) {
                Some(line)
            } else {
                // Non-alphanumeric line marks end of entry
                None
            }
        } else {
            // EOF reached
            None
        }
    }
}

/// How many times a rule matches per line: once, every occurrence, or
/// once then stop all further rules for that line.
#[derive(Debug, Clone, PartialEq)]
pub enum GrcatConfigEntryCount {
    Once,
    More,
    Stop,
}

/// One colorization rule: regex, per-capture-group styles, and match control.
#[derive(Debug, Clone)]
pub struct GrcatConfigEntry {
    pub regex: CompiledRegex,
    pub colors: Vec<Style>,
    pub skip: bool,
    pub count: GrcatConfigEntryCount,
    pub replace: String,
}

impl GrcatConfigEntry {
    #[allow(dead_code)]
    pub fn new(regex: CompiledRegex, colors: Vec<Style>) -> Self {
        GrcatConfigEntry {
            regex,
            colors,
            skip: false,
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
        }
    }
}

impl<A: BufRead> Iterator for GrcatConfigReader<A> {
    type Item = GrcatConfigEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let re = Regex::new("^([a-z_]+)\\s*=\\s*(.*)$").unwrap();
        let mut ln: String;

        while let Some(line) = self.next_alphanumeric() {
            ln = line;
            let mut regex: Option<CompiledRegex> = None;
            let mut colors: Option<Vec<Style>> = None;
            let mut skip: Option<bool> = None;
            let mut count: Option<GrcatConfigEntryCount> = None;
            let mut replace: Option<String> = None;

            // an entry is a run of consecutive alphanumeric-starting lines;
            // a blank/comment line ends it.
            loop {
                let cap = re.captures(&ln).unwrap();
                let key = cap.get(1).unwrap().as_str();
                let value = cap.get(2).unwrap().as_str();

                match key {
                    "regexp" => match CompiledRegex::new(value) {
                        Ok(re) => regex = Some(re),
                        Err(_exc) => eprintln!("Failed regexp: {:?}", _exc),
                    },
                    // accept British/American spelling, singular/plural
                    "colours" | "colors" | "colour" => match styles_from_str(value) {
                        Ok(styles) => colors = Some(styles),
                        Err(e) => {
                            eprintln!("Error: Invalid style in configuration: {}", e);
                            eprintln!("Skipping this rule due to style error.");
                            break;
                        }
                    },
                    "count" => {
                        count = match value {
                            "once" => Some(GrcatConfigEntryCount::Once),
                            "more" => Some(GrcatConfigEntryCount::More),
                            "stop" => Some(GrcatConfigEntryCount::Stop),
                            _ => {
                                eprintln!("Unknown count value: {}", value);
                                None
                            }
                        };
                    }
                    "replace" => {
                        replace = Some(value.to_string());
                    }
                    "skip" => {
                        skip = match value.to_lowercase().as_str() {
                            "true" | "1" | "yes" => Some(true),
                            "false" | "0" | "no" => Some(false),
                            _ => {
                                eprintln!("Unknown skip value: {}, defaulting to false", value);
                                Some(false)
                            }
                        };
                    }
                    // unknown keys ignored for forward compat with grcat
                    _ => {}
                };

                if let Some(nline) = self.following() {
                    ln = nline;
                } else {
                    break;
                }
            }

            // entry is only valid if it had a compilable regexp
            if let Some(regex) = regex {
                return Some(GrcatConfigEntry {
                    regex,
                    colors: colors.unwrap_or_default(),
                    skip: skip.unwrap_or(false),
                    count: count.unwrap_or(GrcatConfigEntryCount::More),
                    replace: replace.unwrap_or_default(),
                });
            }
            // This entry lacked a valid regex; skip and try next entry
        }
        None // No more entries (EOF)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ansi(s: &str) -> Result<String, String> {
        style_from_str(s).map(|st| st.apply_to("").to_string())
    }

    #[test]
    fn style_colour_underscore() {
        assert_eq!(ansi("colour_140"), Ok("\x1b[38;5;140m\x1b[0m".to_string()));
    }

    #[test]
    fn style_color_underscore() {
        assert_eq!(ansi("color_140"), Ok("\x1b[38;5;140m\x1b[0m".to_string()));
    }

    #[test]
    fn style_on_colour() {
        assert_eq!(
            ansi("on_colour_140"),
            Ok("\x1b[48;5;140m\x1b[0m".to_string())
        );
    }

    #[test]
    fn style_rgb_hex() {
        assert_eq!(
            ansi("rgb:ff8800"),
            Ok("\x1b[38;2;255;136;0m\x1b[0m".to_string())
        );
    }

    #[test]
    fn style_on_rgb() {
        assert_eq!(
            style_from_str("on_rgb:ff8800").map(|st| st.apply_to("").to_string()),
            Ok("\x1b[48;2;255;136;0m\x1b[0m".to_string()),
        );
    }

    #[test]
    fn style_raw_ansi_256() {
        assert_eq!(
            style_from_str(r#""\033[38;5;140m""#).map(|st| st.apply_to("").to_string()),
            Ok("\x1b[38;5;140m\x1b[0m".to_string()),
        );
    }

    #[test]
    fn style_raw_ansi_truecolor() {
        assert_eq!(
            style_from_str(r#""\033[38;2;255;136;0m""#).map(|st| st.apply_to("").to_string()),
            Ok("\x1b[38;2;255;136;0m\x1b[0m".to_string()),
        );
    }

    #[test]
    fn style_bad_rgb() {
        assert!(style_from_str("rgb:xyz").is_err());
    }

    #[test]
    fn style_bad_colour_index() {
        assert!(style_from_str("colour_abc").is_err());
    }
}
