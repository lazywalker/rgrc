// rgrc-validate: Standalone configuration validation tool
//
// This tool validates rgrc configuration files and reports errors
// in a user-friendly format with file locations and suggestions.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use rgrc::Style;

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
        "all" => {
            validate_grc_config(&["validate".to_string(), "grc".to_string()]);
            validate_conf_files(&["validate".to_string(), "conf".to_string()]);
        }
        "--help" | "-h" => print_help(&args[0]),
        "--version" | "-v" => println!("rgrc-validate 0.1.0"),
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
    println!("  all               Validate all configurations");
    println!("  --help, -h        Show this help message");
    println!("  --version, -v     Show version");
    println!();
    println!("Examples:");
    println!("  {} grc                    # Validate default grc.conf", prog);
    println!("  {} grc ~/.config/grc.conf # Validate custom config", prog);
    println!("  {} conf share/conf.ping   # Validate single conf file", prog);
    println!("  {} conf share/conf.*      # Validate all conf files", prog);
    println!("  {} all                    # Validate everything", prog);
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
                println!("{} {} configuration is valid", Style::new().green().apply_to("✓"), config_path.display());
                std::process::exit(0);
            } else {
                print_errors(&errors);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{} Failed to read {}: {}", Style::new().red().apply_to("✗"), config_path.display(), e);
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
        println!("{}Validating color configuration files...", Style::new().bold().apply_to(""));
        println!();

        for arg in &args[2..] {
            let path = PathBuf::from(arg);
            
            if !path.exists() {
                eprintln!("  {} {} (file not found)", 
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
                        println!("  {} {}", 
                            Style::new().green().apply_to("✓"),
                            path.display()
                        );
                    } else {
                        println!("  {} {}", 
                            Style::new().red().apply_to("✗"),
                            path.display()
                        );
                        print_errors(&errors);
                        total_errors += errors.len();
                    }
                    validated_files += 1;
                }
                Err(e) => {
                    eprintln!("  {} {} (read error: {})", 
                        Style::new().red().apply_to("✗"),
                        path.display(),
                        e
                    );
                    total_errors += 1;
                }
            }
        }

        println!();
        println!("Summary: {} files validated, {} errors", validated_files, total_errors);
        
        if total_errors > 0 {
            std::process::exit(1);
        }
        return;
    }

    // Otherwise, validate all conf.* files in the default directory
    let conf_dir = find_conf_dir();

    println!("{}Validating color configuration files...", Style::new().bold().apply_to(""));
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
                            println!("  {} {}", 
                                Style::new().green().apply_to("✓"),
                                path.file_name().unwrap_or_default().to_string_lossy()
                            );
                        } else {
                            println!("  {} {}", 
                                Style::new().red().apply_to("✗"),
                                path.file_name().unwrap_or_default().to_string_lossy()
                            );
                            print_errors(&errors);
                            total_errors += errors.len();
                        }
                        validated_files += 1;
                    }
                    Err(e) => {
                        println!("  {} {} (read error: {})", 
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
            eprintln!("{} Failed to read conf directory: {}", Style::new().red().apply_to("✗"), e);
            std::process::exit(1);
        }
    }

    println!();
    println!("Summary: {} files validated, {} errors", validated_files, total_errors);
    
    if total_errors > 0 {
        std::process::exit(1);
    }
}

/// Validate grc.conf format
fn validate_grc_content(content: &str, path: &Path, errors: &mut Vec<ValidationError>) {
    let reader = BufReader::new(content.as_bytes());
    let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
    let mut i = 0;

    while i < lines.len() {
        let line = &lines[i];
        let trimmed = line.trim();
        let line_num = i + 1;

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            i += 1;
            continue;
        }

        // Skip separator lines (lines consisting only of = or - characters)
        if trimmed.chars().all(|c| c == '=' || c == '-') {
            i += 1;
            continue;
        }

        // This is a regex pattern - next line should be the config file
        let regex_pattern = trimmed;

        // Validate regex using CompiledRegex (supports lookahead/lookbehind)
        if let Err(e) = rgrc::grc::CompiledRegex::new(regex_pattern) {
            errors.push(ValidationError {
                path: path.to_path_buf(),
                line: line_num,
                error_type: "RegexError".to_string(),
                message: format!("Invalid regex: {}", e),
                suggestion: Some("Check regex syntax (escape special characters with \\)".to_string()),
            });
            i += 1;
            continue;
        }

        // Find next content line (skip comments and empty lines)
        i += 1;
        while i < lines.len() {
            let next_line = lines[i].trim();
            if next_line.is_empty() || next_line.starts_with('#') || next_line.chars().all(|c| c == '=' || c == '-') {
                i += 1;
                continue;
            }
            break;
        }

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

        let config_line = lines[i].trim();
        let config_line_num = i + 1;

        // Validate config file reference
        if !config_line.starts_with("conf.") {
            errors.push(ValidationError {
                path: path.to_path_buf(),
                line: config_line_num,
                error_type: "FormatError".to_string(),
                message: format!("Config file should start with 'conf.': {}", config_line),
                suggestion: Some("Format: conf.name".to_string()),
            });
        }

        i += 1;
    }
}

/// Validate conf.* file format (supports both GRC key=value format and simple format)
fn validate_conf_content(content: &str, path: &Path, errors: &mut Vec<ValidationError>) {
    let reader = BufReader::new(content.as_bytes());
    // Regex pattern to parse key=value lines
    let kv_re = regex::Regex::new(r"^([a-z_]+)\s*=\s*(.*)$").unwrap();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line_num = line_num + 1;
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                errors.push(ValidationError {
                    path: path.to_path_buf(),
                    line: line_num,
                    error_type: "IOError".to_string(),
                    message: format!("Failed to read line: {}", e),
                    suggestion: None,
                });
                continue;
            }
        };

        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip separator lines (lines consisting only of separator characters)
        if trimmed.chars().all(|c| c == '=' || c == '-' || c == '.' || c == '%') {
            continue;
        }

        // Try to parse as GRC key=value format first
        if let Some(caps) = kv_re.captures(trimmed) {
            let key = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str();

            match key {
                "regexp" => {
                    // Validate regex - try standard regex first, then enhanced
                    if regex::Regex::new(value).is_err() {
                        // Try enhanced regex (for lookahead/lookbehind patterns)
                        if rgrc::grc::CompiledRegex::new(value).is_err() {
                            errors.push(ValidationError {
                                path: path.to_path_buf(),
                                line: line_num,
                                error_type: "RegexError".to_string(),
                                message: format!("Invalid regex pattern: {}", value),
                                suggestion: Some("Check regex syntax (escape special characters with \\)".to_string()),
                            });
                        }
                    }
                }
                "colours" | "colors" => {
                    // Validate colour definitions (comma-separated styles)
                    validate_colours_definition(value, line_num, path, errors);
                }
                "count" => {
                    // Validate count value
                    if !["once", "more", "stop", "previous", "block", "unblock"].contains(&value) {
                        errors.push(ValidationError {
                            path: path.to_path_buf(),
                            line: line_num,
                            error_type: "ValueError".to_string(),
                            message: format!("Unknown count value: '{}'", value),
                            suggestion: Some("Valid values: once, more, stop".to_string()),
                        });
                    }
                }
                "skip" => {
                    // Validate skip value
                    let lower = value.to_lowercase();
                    if !["true", "false", "yes", "no", "1", "0"].contains(&lower.as_str()) {
                        errors.push(ValidationError {
                            path: path.to_path_buf(),
                            line: line_num,
                            error_type: "ValueError".to_string(),
                            message: format!("Unknown skip value: '{}'", value),
                            suggestion: Some("Valid values: true, false, yes, no".to_string()),
                        });
                    }
                }
                "replace" => {
                    // Replace value is free-form, no validation needed
                }
                _ => {
                    // Unknown key - this is acceptable, GRC format may have other keys
                }
            }
        } else {
            // Try simple format: regex whitespace styles
            if !trimmed.contains(' ') && !trimmed.contains('\t') {
                errors.push(ValidationError {
                    path: path.to_path_buf(),
                    line: line_num,
                    error_type: "FormatError".to_string(),
                    message: "Missing style definition (regex must be separated from style by whitespace)".to_string(),
                    suggestion: Some("Format: regex white bold red".to_string()),
                });
                continue;
            }

            // Split regex from styles
            let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
            if parts.len() < 2 {
                continue;
            }

            let regex_part = parts[0];
            let style_part = parts[1];

            // Validate regex
            if regex::Regex::new(regex_part).is_err() {
                // Try enhanced regex
                if rgrc::grc::CompiledRegex::new(regex_part).is_err() {
                    errors.push(ValidationError {
                        path: path.to_path_buf(),
                        line: line_num,
                        error_type: "RegexError".to_string(),
                        message: format!("Invalid regex: {}", regex_part),
                        suggestion: Some("Check regex syntax (escape special characters with \\)".to_string()),
                    });
                    continue;
                }
            }

            // Validate styles (simple format uses space-separated styles)
            validate_simple_style_definition(style_part, line_num, path, errors);
        }
    }
}

/// Validate simple format style definition (space-separated styles on same line as regex)
fn validate_simple_style_definition(style_def: &str, line_num: usize, path: &Path, errors: &mut Vec<ValidationError>) {
    // Valid style keywords for simple format (includes hyphenated variants)
    let valid_styles = vec![
        // No-op keywords
        "", "unchanged", "default", "dark", "none",
        // Foreground colors
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
        // Background colors
        "on_black", "on_red", "on_green", "on_yellow", "on_blue",
        "on_magenta", "on_cyan", "on_white",
        // Text attributes
        "bold", "dim", "italic", "underline", "blink", "reverse",
        // Bright color variants (underscore)
        "bright_black", "bright_red", "bright_green", "bright_yellow",
        "bright_blue", "bright_magenta", "bright_cyan", "bright_white",
        // Bright color variants (hyphen - for backward compatibility)
        "bright-black", "bright-red", "bright-green", "bright-yellow",
        "bright-blue", "bright-magenta", "bright-cyan", "bright-white",
    ];

    for style in style_def.split_whitespace() {
        if !valid_styles.contains(&style) {
            errors.push(ValidationError {
                path: path.to_path_buf(),
                line: line_num,
                error_type: "StyleError".to_string(),
                message: format!("Unknown style: '{}'", style),
                suggestion: Some("Valid styles: black, red, green, yellow, blue, magenta, cyan, white, bold, underline, etc.".to_string()),
            });
        }
    }
}

/// Validate colours definition (comma-separated style groups)
fn validate_colours_definition(colours_def: &str, line_num: usize, path: &Path, errors: &mut Vec<ValidationError>) {
    // Valid style keywords (matching grc.rs style_from_str)
    let valid_styles = vec![
        // No-op keywords
        "", "unchanged", "default", "dark", "none",
        // Foreground colors
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
        // Background colors
        "on_black", "on_red", "on_green", "on_yellow", "on_blue",
        "on_magenta", "on_cyan", "on_white",
        // Text attributes
        "bold", "dim", "italic", "underline", "blink", "reverse",
        // Bright color variants
        "bright_black", "bright_red", "bright_green", "bright_yellow",
        "bright_blue", "bright_magenta", "bright_cyan", "bright_white",
    ];

    // Split by comma to get individual style groups (for capture groups)
    for style_group in colours_def.split(',') {
        let style_group = style_group.trim();
        
        // Skip ANSI escape sequences (e.g., "\033[38;5;140m")
        if style_group.starts_with('"') && style_group.contains("\\033[") {
            continue;
        }
        
        // Validate each space-separated style within the group
        for style in style_group.split_whitespace() {
            if !valid_styles.contains(&style) {
                errors.push(ValidationError {
                    path: path.to_path_buf(),
                    line: line_num,
                    error_type: "StyleError".to_string(),
                    message: format!("Unknown style: '{}'", style),
                    suggestion: Some("Valid styles: black, red, green, yellow, blue, magenta, cyan, white, bold, underline, etc.".to_string()),
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
    let candidates = vec![
        "share/",
        "~/.config/rgrc/",
        "/etc/rgrc/",
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
        eprintln!("  {}: {}", 
            Style::new().red().bold().apply_to("Error"),
            Style::new().red().apply_to(&error.error_type)
        );
        eprintln!("    {}:{}",
            Style::new().yellow().apply_to(&error.path.display().to_string()),
            Style::new().yellow().bold().apply_to(&error.line.to_string())
        );
        eprintln!("    {}", error.message);
        if let Some(suggestion) = &error.suggestion {
            eprintln!("    {}: {}", 
                Style::new().cyan().bold().apply_to("Suggestion"),
                Style::new().cyan().apply_to(suggestion)
            );
        }
    }
}
