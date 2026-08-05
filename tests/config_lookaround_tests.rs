use rgrc::grc::CompiledRegex;

#[test]
fn test_conf_df_patterns() {
    let pattern = r"\d+(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();

    let test_cases = vec![
        ("1234567 used", true),
        ("98765", true),
        ("123 available", true),
        ("500 total", true),
    ];

    for (text, should_match) in test_cases {
        assert_eq!(regex.is_match(text), should_match, "Failed on: {}", text);
    }
}

#[test]
fn test_conf_dockerimages_patterns() {
    let pattern = r"\d+(?:\.\d+)?(?:[KMG]B)?(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();

    let test_cases = vec![
        ("123MB ", true),
        ("45.6GB", true),
        ("789KB ", true),
        ("1024", true),
    ];

    for (text, should_match) in test_cases {
        assert_eq!(regex.is_match(text), should_match, "Failed on: {}", text);
    }
}

#[test]
fn test_conf_dockerps_patterns() {
    let pattern = r".*(?=(?:Up|Exited|Created))";
    let regex = CompiledRegex::new(pattern).unwrap();

    let test_cases = vec![
        ("container Up", true),
        ("abc Exited", true),
        ("test Created", true),
        ("running", false),
    ];

    for (text, should_match) in test_cases {
        assert_eq!(regex.is_match(text), should_match, "Failed on: {}", text);
    }
}

#[test]
fn test_conf_ls_patterns() {
    let pattern1 = r"\s+(\d{7}|\d(?:[,.]?\d+)?[KM])(?=\s[A-Z][a-z]{2}\s)";
    let regex1 = CompiledRegex::new(pattern1).unwrap();

    assert!(regex1.is_match("  1234567 Nov 30 "));
    assert!(regex1.is_match("  123K Nov 29 "));
    assert!(regex1.is_match("  45M Dec 01 "));

    let pattern2 = r"[drwxl-]{10}(?=\s)";
    let regex2 = CompiledRegex::new(pattern2).unwrap();

    assert!(regex2.is_match("drwxr-xr-x "));
    assert!(regex2.is_match("-rw-r--r-- "));
    assert!(regex2.is_match("lrwxrwxrwx "));
}

#[test]
fn test_conf_ps_patterns() {
    let pattern = r"\d+(?=\s)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("12345 process"));
    assert!(regex.is_match("999 cmd"));
    assert!(!regex.is_match("12345"));
}

#[test]
fn test_conf_sockstat_patterns() {
    let pattern = r"(?<=[,<])[^,]+?(?=[,>])";
    let regex = CompiledRegex::new(pattern).unwrap();

    let test_cases = vec![
        (",value1,", true),
        ("<value2,", true), // < followed by text then ,
        (",abc,def,", true),
        ("no-delimiters", false),
    ];

    for (text, should_match) in test_cases {
        assert_eq!(regex.is_match(text), should_match, "Failed on: {}", text);
    }
}

#[test]
fn test_conf_ifconfig_patterns() {
    let pattern = r"(?<=inet\s)\d+\.\d+\.\d+\.\d+";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("inet 192.168.1.1"));
    assert!(regex.is_match("inet 10.0.0.1"));
    assert!(!regex.is_match("192.168.1.1"));
}

#[test]
fn test_conf_netstat_patterns() {
    let pattern = r"\d+(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("8080 LISTEN"));
    assert!(regex.is_match("443"));
    assert!(regex.is_match("22 ESTABLISHED"));
}

#[test]
fn test_conf_mount_patterns() {
    let pattern = r"/[\w/]+(?=\s)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("/dev/sda1 "));
    assert!(regex.is_match("/mnt/data "));
    assert!(regex.is_match("/home/user "));
}

#[test]
fn test_conf_lsblk_patterns() {
    let pattern = r"\d+(?:\.\d+)?[KMGT]?(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("256G "));
    assert!(regex.is_match("128M"));
    assert!(regex.is_match("1.5T "));
}

#[test]
fn test_conf_iostat_sar_patterns() {
    let pattern = r"\d+\.\d+(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("12.34 "));
    assert!(regex.is_match("99.99"));
    assert!(regex.is_match("0.01 avg"));
}

#[test]
fn test_conf_findmnt_patterns() {
    let pattern = r"[/\w]+(?=\s)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("/dev/sda "));
    assert!(regex.is_match("tmpfs "));
}

#[test]
fn test_conf_kubectl_patterns() {
    let pattern = r"\w+(?=\s)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("pod "));
    assert!(regex.is_match("deployment "));
    assert!(regex.is_match("service "));
}

#[test]
fn test_conf_stat_patterns() {
    let pattern = r"\d+(?=\s)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("1024 bytes"));
    assert!(regex.is_match("755 mode"));
}

#[test]
fn test_conf_uptime_patterns() {
    let pattern = r"\d+(?=:\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("12:34"));
    assert!(regex.is_match("8:05"));
}

#[test]
fn test_conf_traceroute_patterns() {
    let pattern = r"\d+\.\d+\.\d+\.\d+(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("192.168.1.1 "));
    assert!(regex.is_match("8.8.8.8"));
}

#[test]
fn test_conf_sysctl_patterns() {
    let pattern = r"\w+(?==)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("kernel.hostname=localhost"));
    assert!(regex.is_match("net.ipv4.ip_forward=1"));
}

#[test]
fn test_conf_iwconfig_patterns() {
    let pattern = r"\d+(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("54 Mb/s"));
    assert!(regex.is_match("100"));
}

#[test]
fn test_conf_yaml_patterns() {
    let pattern = r"\w+(?=:)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("key: value"));
    assert!(regex.is_match("name: test"));
}

#[test]
fn test_conf_esperanto_patterns() {
    let pattern = r"\w+(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("word "));
    assert!(regex.is_match("test"));
}

#[test]
fn test_conf_docker_machinels_patterns() {
    let pattern = r"\w+(?=\s)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("default "));
    assert!(regex.is_match("machine1 "));
}

#[test]
fn test_conf_dockernetwork_patterns() {
    let pattern = r"[a-f0-9]+(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("abc123 "));
    assert!(regex.is_match("def456"));
}

#[test]
fn test_conf_dockersearch_patterns() {
    let pattern = r"\d+(?=\s)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("1234 stars"));
    assert!(regex.is_match("56 official"));
}

#[test]
fn test_conf_pv_patterns() {
    let pattern = r"\d+(?:\.\d+)?%(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("45.5% "));
    assert!(regex.is_match("100%"));
}

#[test]
fn test_all_lookaround_configs_load() {
    let configs = vec![
        "conf.df",
        "conf.dockerimages",
        "conf.dockerps",
        "conf.ls",
        "conf.ps",
        "conf.sockstat",
        "conf.ifconfig",
        "conf.mount",
        "conf.lsblk",
        "conf.iostat_sar",
        "conf.findmnt",
        "conf.kubectl",
        "conf.stat",
        "conf.uptime",
        "conf.traceroute",
        "conf.sysctl",
        "conf.iwconfig",
        "conf.yaml",
        "conf.esperanto",
        "conf.docker-machinels",
        "conf.dockernetwork",
        "conf.dockersearch",
        "conf.pv",
    ];

    assert_eq!(
        configs.len(),
        23,
        "Should have tests for all 23 lookaround configs"
    );
}

#[test]
fn test_fast_path_patterns() {
    let fast_patterns = vec![
        (r"\s|$", "test ", true),
        (r"\s|$", "test", true),
        (r"\s", "test ", true),
        (r"\s", "test", false),
        (r"$", "test", true),
        (r"$", "test ", false),
        (r"\s[A-Z]", "test A", true),
        (r"\s[A-Z]", "test a", false),
        (r"\s[A-Z][a-z]{2}\s", "123 Nov  ", true), // Need space after "Nov "
        (r"\s[A-Z][a-z]{2}\s", "123 nov ", false),
        (r"[:/]", "test:", true),
        (r"[:/]", "test/", true),
        (r"[:/]", "test ", false),
    ];

    for (pattern, text, expected) in fast_patterns {
        let full_pattern = format!(r"\w+(?={})", pattern);
        let regex = CompiledRegex::new(&full_pattern).unwrap();
        assert_eq!(
            regex.is_match(text),
            expected,
            "Fast-path failed for pattern: {} with text: {}",
            pattern,
            text
        );
    }
}
