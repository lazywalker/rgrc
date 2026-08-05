//! Apply regex-based color rules to input text line by line.

use std::io::{BufRead, BufReader, Read, Write};

use crate::grc::GrcatConfigEntry;
use crate::style::Style;

// Decode a raw line read by read_until(b'\n'). Trims trailing \n and \r,
// replaces invalid UTF-8 with U+FFFD so binary output doesn't abort (#31).
pub fn decode_line(raw: &[u8]) -> String {
    let mut end = raw.len();
    if end > 0 && raw[end - 1] == b'\n' {
        end -= 1;
        if end > 0 && raw[end - 1] == b'\r' {
            end -= 1;
        }
    }
    String::from_utf8_lossy(&raw[..end]).into_owned()
}

// read_until + lossy decode instead of BufRead::lines(), which rejects
// non-UTF-8 (see #31).
#[allow(dead_code)]
pub fn colorize_regex<R, W>(
    reader: &mut R,
    writer: &mut W,
    rules: &[GrcatConfigEntry],
) -> Result<(), Box<dyn std::error::Error>>
where
    R: Read,
    W: Write,
{
    let mut reader = BufReader::new(reader);
    let mut raw_buf: Vec<u8> = Vec::new();

    if rules.is_empty() {
        loop {
            raw_buf.clear();
            let read = reader.read_until(b'\n', &mut raw_buf)?;
            if read == 0 {
                break;
            }
            let line = decode_line(&raw_buf);
            writeln!(writer, "{}", line)?;
        }
        return Ok(());
    }

    let default_style = Style::new();

    loop {
        raw_buf.clear();
        let read = reader.read_until(b'\n', &mut raw_buf)?;
        if read == 0 {
            break;
        }
        let mut line = decode_line(&raw_buf);

        if line.is_empty() {
            writeln!(writer)?;
            continue;
        }

        let mut style_ranges: Vec<(usize, usize, &Style)> = Vec::new();
        let mut stop_line_processing = false;

        'outer_loop: for rule in rules {
            if rule.skip {
                continue;
            }

            if stop_line_processing {
                break;
            }

            let mut offset = 0;
            let mut last_end = 0;
            let mut rule_matched_once = false;

            while offset < line.len() && !rule_matched_once {
                // skip past the end of a previous match within this rule to
                // avoid re-styling overlapping capture groups.
                if offset < last_end {
                    offset = last_end;
                    continue;
                }

                if let Some(matches) = rule.regex.captures_from_pos(&line, offset) {
                    for (i, mmatch) in matches.iter().into_iter().enumerate() {
                        // capture group 0 is the full match; 1+ are subgroups.
                        // each group gets its own style from rule.colors[i].
                        if let Some(mmatch) = mmatch {
                            let start = mmatch.start();
                            let end = mmatch.end();

                            if i < rule.colors.len() {
                                let style = &rule.colors[i];
                                style_ranges.push((start, end, style));
                                last_end = last_end.max(end);
                            }
                        }

                        let full_match = matches.get(0).unwrap();

                        if !rule.replace.is_empty() {
                            let mut replacement = rule.replace.clone();
                            for (i, capture) in matches.iter().into_iter().enumerate() {
                                if let Some(capture_match) = capture {
                                    let capture_text =
                                        &line[capture_match.start()..capture_match.end()];
                                    let placeholder = format!("\\{}", i);
                                    replacement = replacement.replace(&placeholder, capture_text);
                                }
                            }

                            let before = &line[..full_match.start()];
                            let after = &line[full_match.end()..];
                            line = format!("{}{}{}", before, replacement, after);

                            // line changed; restart rule matching from scratch.
                            break 'outer_loop;
                        }

                        match rule.count {
                            crate::grc::GrcatConfigEntryCount::Once => {
                                rule_matched_once = true;
                            }
                            crate::grc::GrcatConfigEntryCount::More => {}
                            crate::grc::GrcatConfigEntryCount::Stop => {
                                stop_line_processing = true;
                                rule_matched_once = true;
                            }
                        }
                    }

                    let full_match = matches.get(0).unwrap();

                    if full_match.end() > full_match.start() {
                        offset = full_match.end();
                    } else {
                        // zero-width match: advance one byte to avoid infinite loop
                        offset = full_match.end() + 1;
                    }
                } else {
                    break;
                }
            }
        }

        if style_ranges.is_empty() {
            writeln!(writer, "{}", line)?;
            continue;
        }

        // flatten style ranges into a per-byte array. regex offsets are byte
        // positions, so indexing by byte (not char) keeps slice boundaries valid.
        // later ranges override earlier ones on overlap.
        let mut char_styles: Vec<&Style> = vec![&default_style; line.len()];

        for (start, end, style) in style_ranges {
            for item in char_styles.iter_mut().take(end.min(line.len())).skip(start) {
                *item = style;
            }
        }

        // run-length encode: emit one styled segment per contiguous run of
        // identical styles, minimizing ANSI escape sequences.
        let mut prev_style = &default_style;
        let mut offset = 0;

        for i in 0..line.len() {
            let this_style = char_styles[i];

            if this_style != prev_style {
                if i > 0 {
                    write!(writer, "{}", prev_style.apply_to(&line[offset..i]))?;
                }
                prev_style = this_style;
                offset = i;
            }
        }

        if offset < line.len() {
            write!(writer, "{}", prev_style.apply_to(&line[offset..]))?;
        }

        writeln!(writer)?;
    }

    Ok(())
}
