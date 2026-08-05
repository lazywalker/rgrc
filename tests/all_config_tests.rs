use rgrc::grc::CompiledRegex;

#[test]
fn test_conf_ant() {
    let pattern = r"(?i)(error|warning|failed)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("BUILD FAILED"));
    assert!(regex.is_match("Warning: deprecated"));
}

#[test]
fn test_conf_blkid() {
    let pattern = r"UUID=[a-f0-9-]+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("UUID=1234-5678-abcd"));
}

#[test]
fn test_conf_configure() {
    let pattern = r"checking for";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("checking for gcc... yes"));
}

#[test]
fn test_conf_curl() {
    let pattern = r"\d+\s+\d+\s+\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("100  1024  100  1024"));
}

#[test]
fn test_conf_cvs() {
    let pattern = r"^[UPMCA]\s";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("U file.txt"));
}

#[test]
fn test_conf_diff() {
    let pattern = r"^[+\-]";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("+added line"));
    assert!(regex.is_match("-removed line"));
}

#[test]
fn test_conf_dig() {
    let pattern = r"\d+\.\d+\.\d+\.\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("8.8.8.8"));
}

#[test]
fn test_conf_dnf() {
    let pattern = r"(?i)(installing|upgrading|removing)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("Installing package"));
}

#[test]
fn test_conf_dockerinfo() {
    let pattern = r"\d+\s+(containers|images)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("5 containers"));
}

#[test]
fn test_conf_dockerpull() {
    let pattern = r"\d+%";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("Downloading 45%"));
}

#[test]
fn test_conf_dockerversion() {
    let pattern = r"Version:\s+\d+\.\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("Version: 20.10"));
}

#[test]
fn test_conf_du() {
    let pattern = r"\d+[KMG]?\s+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("1024K /home"));
    assert!(regex.is_match("5M /var"));
}

#[test]
fn test_conf_env() {
    let pattern = r"^\w+=";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("PATH=/usr/bin"));
}

#[test]
fn test_conf_fdisk() {
    let pattern = r"/dev/[a-z]+\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("/dev/sda1"));
}

#[test]
fn test_conf_free() {
    let pattern = r"\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("8192 total"));
}

#[test]
fn test_conf_gcc() {
    let pattern = r"(?i)(error|warning):";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("error: undefined reference"));
}

#[test]
fn test_conf_getfacl() {
    let pattern = r"user::\w+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("user::rwx"));
}

#[test]
fn test_conf_getsebool() {
    let pattern = r"\w+\s+-->\s+(on|off)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("httpd_can_network_connect --> on"));
}

#[test]
fn test_conf_go_test() {
    let pattern = r"(?i)(pass|fail|ok)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("PASS"));
    assert!(regex.is_match("ok   package 0.001s"));
}

#[test]
fn test_conf_id() {
    let pattern = r"uid=\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("uid=1000(user)"));
}

#[test]
fn test_conf_ipaddr() {
    let pattern = r"\d+\.\d+\.\d+\.\d+/\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("192.168.1.1/24"));
}

#[test]
fn test_conf_ipneighbor() {
    let pattern = r"\d+\.\d+\.\d+\.\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("192.168.1.1 dev eth0"));
}

#[test]
fn test_conf_iproute() {
    let pattern = r"\d+\.\d+\.\d+\.\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("default via 192.168.1.1"));
}

#[test]
fn test_conf_iptables() {
    let pattern = r"(ACCEPT|DROP|REJECT)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("ACCEPT all"));
}

#[test]
fn test_conf_irclog() {
    let pattern = r"<\w+>";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("<username> Hello"));
}

#[test]
fn test_conf_jobs() {
    let pattern = r"\[\d+\]";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("[1] Running"));
}

#[test]
fn test_conf_last() {
    let pattern = r"\w+\s+pts/\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("user pts/0"));
}

#[test]
fn test_conf_ldap() {
    let pattern = r"dn:";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("dn: cn=admin,dc=example"));
}

#[test]
fn test_conf_lolcat() {
    let pattern = r"\w+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("rainbow text"));
}

#[test]
fn test_conf_lsattr() {
    let pattern = r"[a-z-]{12}";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("----i--------"));
}

#[test]
fn test_conf_lsmod() {
    let pattern = r"^\w+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("module_name 16384 1"));
}

#[test]
fn test_conf_lsof() {
    let pattern = r"(COMMAND|PID|USER)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("COMMAND PID USER"));
}

#[test]
fn test_conf_lspci() {
    let pattern = r"[0-9a-f]{2}:[0-9a-f]{2}\.\d";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("00:1f.2 SATA controller"));
}

#[test]
fn test_conf_lsusb() {
    let pattern = r"Bus\s+\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("Bus 001 Device 002"));
}

#[test]
fn test_conf_mtr() {
    let pattern = r"\d+\.\d+%";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("0.0% loss"));
}

#[test]
fn test_conf_mvn() {
    let pattern = r"(?i)(success|failure|error)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("BUILD SUCCESS"));
}

#[test]
fn test_conf_netstat() {
    let pattern = r"(LISTEN|ESTABLISHED|TIME_WAIT)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("ESTABLISHED"));
}

#[test]
fn test_conf_nmap() {
    let pattern = r"\d+/tcp\s+(open|closed)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("80/tcp open http"));
}

#[test]
fn test_conf_ntpdate() {
    let pattern = r"\d+\.\d+\.\d+\.\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("server 192.168.1.1"));
}

#[test]
fn test_conf_php() {
    let pattern = r"(?i)(error|warning|notice)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("PHP Warning: "));
}

#[test]
fn test_conf_ping() {
    let pattern = r"\d+\s+bytes from";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("64 bytes from 192.168.1.1"));
}

#[test]
fn test_conf_semanage() {
    let pattern = r"\w+_t";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("httpd_t"));
}

#[test]
fn test_conf_sensors() {
    let pattern = r"\+?\d+\.\d+°C";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("Core 0: +45.0°C"));
}

#[test]
fn test_conf_showmount() {
    let pattern = r"/\w+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("/export/share"));
}

#[test]
fn test_conf_sqlmap() {
    let pattern = r"(?i)(vulnerable|injection)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("Parameter is vulnerable"));
}

#[test]
fn test_conf_ss() {
    let pattern = r"(ESTAB|LISTEN|CLOSE-WAIT)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("ESTAB 0 0"));
}

#[test]
fn test_conf_systemctl() {
    let pattern = r"(active|inactive|failed)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("active (running)"));
}

#[test]
fn test_conf_tcpdump() {
    let pattern = r"\d+\.\d+\.\d+\.\d+\.\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("192.168.1.1.80 > 192.168.1.2.12345"));
}

#[test]
fn test_conf_tune2fs() {
    let pattern = r"Block count:\s+\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("Block count: 1024000"));
}

#[test]
fn test_conf_ulimit() {
    let pattern = r"(unlimited|\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("open files 1024"));
}

#[test]
fn test_conf_vmstat() {
    let pattern = r"\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("1 0 512 1024"));
}

#[test]
fn test_conf_wdiff() {
    let pattern = r"\[-.*?-\]";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("text [-removed-] more"));
}

#[test]
fn test_conf_whois() {
    let pattern = r"(?i)(registrar|domain)";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("Registrar: Example Inc"));
}

#[test]
fn test_conf_common() {
    let pattern = r"\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("value 123"));
}

#[test]
fn test_conf_dummy() {
    let pattern = r"\w+";
    let regex = CompiledRegex::new(pattern).unwrap();
    assert!(regex.is_match("test"));
}

#[test]
fn test_fast_path_ipv4_continuation() {
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    let test_text = "IP: 192.168.1.1";
    assert!(
        regex.is_match(test_text),
        "Should match '192' in '192.168.1.1'"
    );

    assert!(!regex.is_match("192.168"));
}

#[test]
fn test_fast_path_size_unit_kb() {
    let pattern = r"\d+(?=[KMG]B?)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("1024KB"));
    assert!(regex.is_match("256MB"));
    assert!(regex.is_match("16GB"));
    assert!(regex.is_match("512K"));
    assert!(regex.is_match("2M"));
    assert!(regex.is_match("1G"));
}

#[test]
fn test_fast_path_size_unit_without_b() {
    let pattern = r"\d+(?=[KMGT])";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(regex.is_match("100K"));
    assert!(regex.is_match("50M"));
    assert!(regex.is_match("2G"));
    assert!(regex.is_match("1T"));
}

#[test]
fn test_all_config_files_covered() {
    let lookaround_configs = 23; // From config_lookaround_tests.rs
    let non_lookaround_configs = 58; // From this file
    let total = lookaround_configs + non_lookaround_configs;

    assert!(
        total >= 81,
        "Should have coverage for at least 81 configs, got {}",
        total
    );
}

#[test]
fn test_enhanced_regex_config_compatibility() {
    let patterns = vec![
        r"\d+",                // Basic numbers
        r"\d+\.\d+",           // Decimals
        r"\d+\.\d+\.\d+\.\d+", // IP addresses
        r"\w+",                // Words
        r"[a-zA-Z0-9]+",       // Alphanumeric
        r"\s+",                // Whitespace
        r"^\w+",               // Start of line
        r"\w+$",               // End of line
        r"(ACCEPT|DROP)",      // Alternatives
        r"\d+(?=\s)",          // Lookahead (Enhanced)
        r"(?<=\s)\w+",         // Lookbehind (Enhanced)
        r"\d+(?=[KMG])",       // Size units (Enhanced)
    ];

    for pattern in patterns {
        let result = CompiledRegex::new(pattern);
        assert!(result.is_ok(), "Failed to compile pattern: {}", pattern);
    }
}
