use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell cargo to rerun this build script if any of the config files change
    println!("cargo:rerun-if-changed=etc/rgrc.conf");
    println!("cargo:rerun-if-changed=share/");

    // Pre-process configurations at build time
    preprocess_configs();
}

fn preprocess_configs() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let config_path = Path::new(&out_dir).join("preprocessed_configs.rs");

    let mut output = String::new();
    output.push_str("// Auto-generated file - do not edit\n\n");

    // Process rgrc.conf
    output.push_str("pub static PRECOMPILED_GRC_RULES: &[(&str, &str)] = &[\n");
    if let Ok(content) = fs::read_to_string("etc/rgrc.conf") {
        let mut lines = content.lines();
        while let Some(line) = lines.next() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let regex_str = line;
            if let Some(config_line) = lines.next() {
                let config_file = config_line.trim();
                output.push_str(&format!("    (r\"{}\", \"{}\"),\n", regex_str, config_file));
            }
        }
    }
    output.push_str("];\n\n");

    // Process individual config files
    output.push_str("pub static PRECOMPILED_CONFIGS: &[(&str, &str)] = &[\n");
    if let Ok(entries) = fs::read_dir("share/") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.starts_with("conf.") {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            // Escape quotes and backslashes for Rust string literal
                            let escaped_content = content
                                .replace("\\", "\\\\")
                                .replace("\"", "\\\"")
                                .replace("\n", "\\n");
                            output.push_str(&format!("    (\"{}\", \"{}\"),\n", file_name, escaped_content));
                        }
                    }
                }
            }
        }
    }
    output.push_str("];\n");

    fs::write(config_path, output).unwrap();
}