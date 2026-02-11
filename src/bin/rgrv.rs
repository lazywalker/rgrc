// rgrc-validate: Standalone configuration validation tool
//
// This tool validates rgrc configuration files and reports errors
// in a user-friendly format with file locations and suggestions.

use rgrc::Style;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help(&args[0]);
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "grc" => validate_grc_config(&args),
        "conf" => validate_conf_files(&args),
        "--help" | "-h" => print_help(&args[0]),
        "--version" | "-V" => println!("rgrc-validate 0.1.0"),
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_help(&args[0]);
            std::process::exit(1);
        }
    }
}

/// Print help message
fn print_help(prog: &str) {
    println!("rgrc Configuration Validator");
    println!();
    println!("Usage: {} <COMMAND> [OPTIONS]", prog);
    println!();
    println!("Commands:");
    println!("  grc [PATH]        Validate grc.conf configuration file");
    println!("  conf [PATH ...]   Validate color configuration files (conf.*)");
    println!("  --help, -h        Show this help message");
    println!("  --version, -v     Show version");
    println!();
    println!("Examples:");
    println!(
        "  {} grc                    # Validate default grc.conf",
        prog
    );
    println!("  {} grc ~/.config/grc.conf # Validate custom config", prog);
    println!(
        "  {} conf share/conf.ping   # Validate single conf file",
        prog
    );
    println!(
        "  {} conf share/conf.*      # Validate all conf files",
        prog
    );
}

/// Validate grc.conf file
fn validate_grc_config(args: &[String]) {
    let config_path = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        // Try to find default grc.conf
        find_grc_conf()
    };

    println!("{}Validating grc.conf...", Style::new().bold().apply_to(""));
    println!("  Path: {}", config_path.display());
    println!();

    match fs::read_to_string(&config_path) {
        Ok(content) => {
            let mut errors = Vec::new();
            validate_grc_content(&content, &config_path, &mut errors);

            if errors.is_empty() {
                println!(
                    "{} {} configuration is valid",
                    Style::new().green().apply_to("✓"),
                    config_path.display()
                );
                std::process::exit(0);
            } else {
                print_errors(&errors);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!(
                "{} Failed to read {}: {}",
                Style::new().red().apply_to("✗"),
                config_path.display(),
                e
            );
            std::process::exit(1);
        }
    }
}

/// Validate conf.* files
fn validate_conf_files(args: &[String]) {
    let mut total_errors = 0;
    let mut validated_files = 0;

    // If specific files are provided, validate only those
    if args.len() > 2 {
        println!(
            "{}Validating color configuration files...",
            Style::new().bold().apply_to("")
        );
        println!();

        for arg in &args[2..] {
            let path = PathBuf::from(arg);

            if !path.exists() {
                eprintln!(
                    "  {} {} (file not found)",
                    Style::new().red().apply_to("✗"),
                    path.display()
                );
                total_errors += 1;
                continue;
            }

            match fs::read_to_string(&path) {
                Ok(content) => {
                    let mut errors = Vec::new();
                    validate_conf_content(&content, &path, &mut errors);

                    if errors.is_empty() {
                        println!(
                            "  {} {}",
                            Style::new().green().apply_to("✓"),
                            path.display()
                        );
                    } else {
                        println!("  {} {}", Style::new().red().apply_to("✗"), path.display());
                        print_errors(&errors);
                        total_errors += errors.len();
                    }
                    validated_files += 1;
                }
                Err(e) => {
                    eprintln!(
                        "  {} {} (read error: {})",
                        Style::new().red().apply_to("✗"),
                        path.display(),
                        e
                    );
                    total_errors += 1;
                }
            }
        }

        println!();
        println!(
            "Summary: {} files validated, {} errors",
            validated_files, total_errors
        );

        if total_errors > 0 {
            std::process::exit(1);
        }
        return;
    }

    // Otherwise, validate all conf.* files in the default directory
    let conf_dir = find_conf_dir();

    println!(
        "{}Validating color configuration files...",
        Style::new().bold().apply_to("")
    );
    println!("  Directory: {}", conf_dir.display());
    println!();

    match fs::read_dir(&conf_dir) {
        Ok(entries) => {
            let mut conf_files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map(|n| n.starts_with("conf."))
                        .unwrap_or(false)
                })
                .collect();

            conf_files.sort_by_key(|e| e.file_name());

            for entry in conf_files {
                let path = entry.path();
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        let mut errors = Vec::new();
                        validate_conf_content(&content, &path, &mut errors);

                        if errors.is_empty() {
                            println!(
                                "  {} {}",
                                Style::new().green().apply_to("✓"),
                                path.file_name().unwrap_or_default().to_string_lossy()
                            );
                        } else {
                            println!(
                                "  {} {}",
                                Style::new().red().apply_to("✗"),
                                path.file_name().unwrap_or_default().to_string_lossy()
                            );
                            print_errors(&errors);
                            total_errors += errors.len();
                        }
                        validated_files += 1;
                    }
                    Err(e) => {
                        println!(
                            "  {} {} (read error: {})",
                            Style::new().red().apply_to("✗"),
                            path.file_name().unwrap_or_default().to_string_lossy(),
                            e
                        );
                        total_errors += 1;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!(
                "{} Failed to read conf directory: {}",
                Style::new().red().apply_to("✗"),
                e
            );
            std::process::exit(1);
        }
    }

    println!();
    println!(
        "Summary: {} files validated, {} errors",
        validated_files, total_errors
    );

    if total_errors > 0 {
        std::process::exit(1);
    }
}

/// Validate grc.conf format
fn validate_grc_content(content: &str, path: &Path, errors: &mut Vec<ValidationError>) {
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut i = 0;

    while i < lines.len() {
        let line_num = i + 1;
        let line = &lines[i];
        let trimmed = line.trim();

        // Skip empty lines, comments, and separator lines
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with('-')
            || trimmed.starts_with('=')
        {
            i += 1;
            continue;
        }

        // This is a regex pattern - next line should be the config file
        let regex_pattern = trimmed;

        // Validate regex using CompiledRegex (supports fancy regex features)
        if let Err(e) = rgrc::grc::CompiledRegex::new(regex_pattern) {
            errors.push(ValidationError {
                path: path.to_path_buf(),
                line: line_num,
                error_type: "RegexError".to_string(),
                message: format!("Invalid regex: {}", e),
                suggestion: Some(
                    "Check regex syntax (escape special characters with \\)".to_string(),
                ),
            });
            i += 1;
            continue;
        }

        // Next line should be config file name
        i += 1;

        if i >= lines.len() {
            errors.push(ValidationError {
                path: path.to_path_buf(),
                line: line_num,
                error_type: "FormatError".to_string(),
                message: "Missing config file reference after regex pattern".to_string(),
                suggestion: Some("Add config file name on next line, e.g., conf.ping".to_string()),
            });
            break;
        }

        let next_line_num = i + 1;
        let config_line = lines[i].trim();
        if config_line.is_empty() || config_line.starts_with('#') {
            errors.push(ValidationError {
                path: path.to_path_buf(),
                line: next_line_num,
                error_type: "FormatError".to_string(),
                message: "Expected config file reference after regex pattern".to_string(),
                suggestion: Some("Format:\n  regex_pattern\n  conf.name".to_string()),
            });
            i += 1;
            continue;
        }

        // Check if config file exists
        let config_path = Path::new(config_line);
        if !config_path.exists() && !config_line.starts_with("conf.") {
            // Try in share directory
            let share_path = Path::new("share").join(config_line);
            if !share_path.exists() {
                errors.push(ValidationError {
                    path: path.to_path_buf(),
                    line: next_line_num,
                    error_type: "FileNotFound".to_string(),
                    message: format!("Config file not found: {}", config_line),
                    suggestion: Some(format!("Create {} or check file name", config_line)),
                });
            }
        }

        i += 1;
    }
}

/// Validate conf.* file format
fn validate_conf_content(content: &str, path: &Path, errors: &mut Vec<ValidationError>) {
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut i = 0;

    while i < lines.len() {
        let line_num = i + 1;
        let line = &lines[i];

        // Skip empty lines and comments
        if line.trim().is_empty() || line.trim().starts_with('#') {
            i += 1;
            continue;
        }

        // Check for regexp= line
        if let Some(regex_part) = line.strip_prefix("regexp=") {
            // Validate regex using CompiledRegex (supports fancy regex features)
            if let Err(e) = rgrc::grc::CompiledRegex::new(regex_part) {
                errors.push(ValidationError {
                    path: path.to_path_buf(),
                    line: line_num,
                    error_type: "RegexError".to_string(),
                    message: format!("Invalid regex: {}", e),
                    suggestion: Some(
                        "Check regex syntax (escape special characters with \\)".to_string(),
                    ),
                });
            }

            // Look for colours= line
            i += 1;

            while i < lines.len() {
                let next_line_num = i + 1;
                let next_line = &lines[i];

                if next_line.trim().is_empty() {
                    i += 1;
                    continue;
                }

                if next_line.starts_with("colours=") || next_line.starts_with("colour=") {
                    let style_part = if let Some(stripped) = next_line.strip_prefix("colours=") {
                        stripped
                    } else if let Some(stripped) = next_line.strip_prefix("colour=") {
                        stripped
                    } else {
                        // This shouldn't happen due to the starts_with check above
                        next_line
                    };

                    // Validate styles
                    validate_style_definition(style_part, next_line_num, path, errors);
                    i += 1;

                    // Continue to check for additional config lines after colours=
                    while i < lines.len() {
                        let config_line_num = i + 1;
                        let config_line = &lines[i];

                        if config_line.trim().is_empty() {
                            i += 1;
                            continue;
                        }

                        if config_line.starts_with("count=")
                            || config_line.starts_with("skip=")
                            || config_line.starts_with("replace=")
                            || config_line.starts_with("#")
                        {
                            // Valid config lines or comments after colours=, skip them
                            i += 1;
                        } else if config_line.starts_with("regexp=") {
                            // New regexp= starts a new rule, break out to handle it
                            break;
                        } else if config_line.starts_with("=======")
                            || config_line.starts_with("-")
                            || config_line.starts_with(".........")
                            || config_line.starts_with("==")
                            || config_line.starts_with("%%%%%%%")
                        {
                            // End of rule
                            break;
                        } else {
                            // Unexpected line after colours=
                            errors.push(ValidationError {
                                path: path.to_path_buf(),
                                line: config_line_num,
                                error_type: "FormatError".to_string(),
                                message: format!("Unexpected line after colours=: {}", config_line),
                                suggestion: Some(
                                    "Expected count=, skip=, replace=, regexp= lines or separator"
                                        .to_string(),
                                ),
                            });
                            i += 1;
                        }
                    }
                    break;
                } else if next_line.starts_with("count=")
                    || next_line.starts_with("skip=")
                    || next_line.starts_with("replace=")
                    || next_line.starts_with("#")
                {
                    // Valid config lines or comments, skip them
                    i += 1;
                } else if next_line.starts_with("regexp=") {
                    // New regexp= starts a new rule, break out to handle it
                    break;
                } else if next_line.starts_with("=======")
                    || next_line.starts_with("-")
                    || next_line.starts_with(".........")
                    || next_line.starts_with("==")
                    || next_line.starts_with("%%%%%%%")
                {
                    // End of rule
                    break;
                } else {
                    // Unexpected line
                    errors.push(ValidationError {
                        path: path.to_path_buf(),
                        line: next_line_num,
                        error_type: "FormatError".to_string(),
                        message: format!("Unexpected line after regexp: {}", next_line),
                        suggestion: Some("Expected colours=, count=, skip=, replace=, regexp= lines or ======= / - / ......... / == / %%%%%%% separator".to_string()),
                    });
                    i += 1;
                }
            }
        } else if line.starts_with("=======")
            || line.starts_with("-")
            || line.starts_with(".........")
            || line.starts_with("==")
            || line.starts_with("%%%%%%%")
        {
            // Rule separator, continue
            i += 1;
        } else {
            // Support legacy compact format: "pattern <whitespace or tab> style1 style2"
            // Split on tab first, then on first whitespace if needed.
            let trimmed = line.trim();
            let mut pattern = trimmed;
            let mut style_part: Option<&str> = None;

            // Look for tab separator first (most common in conf files)
            if let Some(idx) = trimmed.find('\t') {
                pattern = trimmed[..idx].trim();
                style_part = Some(trimmed[idx + 1..].trim());
            } else {
                // Look for first whitespace
                if let Some(idx) = trimmed.find(char::is_whitespace) {
                    pattern = &trimmed[..idx];
                    style_part = Some(trimmed[idx..].trim());
                }
            }

            if let Some(styles) = style_part
                && !styles.is_empty()
            {
                // Validate regex
                if let Err(e) = rgrc::grc::CompiledRegex::new(pattern) {
                    errors.push(ValidationError {
                        path: path.to_path_buf(),
                        line: line_num,
                        error_type: "RegexError".to_string(),
                        message: format!("Invalid regex: {}", e),
                        suggestion: Some(
                            "Check regex syntax (escape special characters with \\)".to_string(),
                        ),
                    });
                }

                // Validate styles on the same line
                validate_style_definition(styles, line_num, path, errors);
                i += 1;
                continue;
            }

            // If we get here, it's an unexpected line format
            errors.push(ValidationError {
                path: path.to_path_buf(),
                line: line_num,
                error_type: "FormatError".to_string(),
                message: format!("Unexpected line format: {}", line),
                suggestion: Some(
                    "Expected regexp= line or pattern<tab>styles or ======= / - / ......... / == separator".to_string(),
                ),
            });
            i += 1;
        }
    }
}

/// Validate style definition
fn validate_style_definition(
    style_def: &str,
    line_num: usize,
    path: &Path,
    errors: &mut Vec<ValidationError>,
) {
    let valid_styles = vec![
        // Special keywords
        "unchanged",
        "default",
        "dark",
        "none",
        // Foreground colors
        "black",
        "red",
        "green",
        "yellow",
        "blue",
        "magenta",
        "cyan",
        "white",
        // Bright colors
        "bright_black",
        "bright_red",
        "bright_green",
        "bright_yellow",
        "bright_blue",
        "bright_magenta",
        "bright_cyan",
        "bright_white",
        // Background colors
        "on_black",
        "on_red",
        "on_green",
        "on_yellow",
        "on_blue",
        "on_magenta",
        "on_cyan",
        "on_white",
        // Text attributes
        "bold",
        "dim",
        "italic",
        "underline",
        "blink",
        "reverse",
    ];

    // Split by comma first, then by space for each style group
    for style_group in style_def.split(',') {
        for style in style_group.split_whitespace() {
            let trimmed_style = style.trim();
            if trimmed_style.is_empty() {
                continue;
            }
            // Allow empty quotes for no styling
            if trimmed_style == "''" {
                continue;
            }
            // Allow ANSI escape sequences
            if trimmed_style.starts_with('"') && trimmed_style.contains("\\033[") {
                continue;
            }
            // Normalize hyphenated style names to underscored versions
            let normalized_style = trimmed_style.replace('-', "_");
            if !valid_styles.contains(&normalized_style.as_str()) {
                errors.push(ValidationError {
                    path: path.to_path_buf(),
                    line: line_num,
                    error_type: "StyleError".to_string(),
                    message: format!("Unknown style: '{}'", trimmed_style),
                    suggestion: Some(format!(
                        "Valid styles include: {}",
                        valid_styles[0..12].join(", ")
                    )),
                });
            }
        }
    }
}

/// Find grc.conf file
fn find_grc_conf() -> PathBuf {
    let candidates = vec![
        "etc/rgrc.conf",
        "~/.config/rgrc/rgrc.conf",
        "/etc/rgrc/rgrc.conf",
    ];

    for candidate in candidates {
        let path = if candidate.starts_with("~") {
            if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(candidate.replace("~", &home))
            } else {
                continue;
            }
        } else {
            PathBuf::from(candidate)
        };

        if path.exists() {
            return path;
        }
    }

    PathBuf::from("etc/rgrc.conf")
}

/// Find conf directory
fn find_conf_dir() -> PathBuf {
    let candidates = vec!["share/", "~/.config/rgrc/", "/etc/rgrc/"];

    for candidate in candidates {
        let path = if candidate.starts_with("~") {
            if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(candidate.replace("~", &home))
            } else {
                continue;
            }
        } else {
            PathBuf::from(candidate)
        };

        if path.exists() && path.is_dir() {
            return path;
        }
    }

    PathBuf::from("share/")
}

/// Validation error structure
struct ValidationError {
    path: PathBuf,
    line: usize,
    error_type: String,
    message: String,
    suggestion: Option<String>,
}

/// Print validation errors
fn print_errors(errors: &[ValidationError]) {
    for error in errors {
        eprintln!();
        eprintln!(
            "  {}: {}",
            Style::new().red().bold().apply_to("Error"),
            Style::new().red().apply_to(&error.error_type)
        );
        eprintln!(
            "    {}:{}",
            Style::new()
                .yellow()
                .apply_to(&error.path.display().to_string()),
            Style::new()
                .yellow()
                .bold()
                .apply_to(&error.line.to_string())
        );
        eprintln!("    {}", error.message);
        if let Some(suggestion) = &error.suggestion {
            eprintln!(
                "    {}: {}",
                Style::new().cyan().bold().apply_to("Suggestion"),
                Style::new().cyan().apply_to(suggestion)
            );
        }
    }
}
