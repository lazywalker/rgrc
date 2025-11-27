use console::Style;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use fancy_regex::Regex;
use rgrc::colorizer::{colorize_regex, colorize_simd};
use rgrc::grc::{GrcatConfigEntry, GrcatConfigEntryCount};
use std::io::Cursor;

/// Generate test input data with various patterns
fn generate_test_data(lines: usize, _patterns_per_line: usize) -> String {
    let mut output = String::new();
    for i in 0..lines {
        output.push_str(&format!(
            "INFO: Processing line {} with ERROR markers and WARNING signs scattered throughout\n",
            i
        ));
        if i % 2 == 0 {
            output.push_str(&format!("ERROR: Critical failure on line {}\n", i));
        }
        if i % 3 == 0 {
            output.push_str(&format!(
                "WARNING: Minor issue detected at position {}\n",
                i
            ));
        }
    }
    output
}

/// Create simple literal pattern rules (optimized for SIMD)
fn create_literal_rules() -> Vec<GrcatConfigEntry> {
    vec![
        GrcatConfigEntry {
            regex: Regex::new("ERROR").unwrap(),
            colors: vec![Style::new().red().bold()],
            skip: false,
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
        },
        GrcatConfigEntry {
            regex: Regex::new("WARNING").unwrap(),
            colors: vec![Style::new().yellow()],
            skip: false,
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
        },
        GrcatConfigEntry {
            regex: Regex::new("INFO").unwrap(),
            colors: vec![Style::new().cyan()],
            skip: false,
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
        },
    ]
}

/// Create complex regex pattern rules (not optimized for SIMD)
fn create_regex_rules() -> Vec<GrcatConfigEntry> {
    vec![
        GrcatConfigEntry {
            regex: Regex::new(r"ERROR:\s+(.*)").unwrap(),
            colors: vec![Style::new().red().bold(), Style::new().red()],
            skip: false,
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
        },
        GrcatConfigEntry {
            regex: Regex::new(r"WARNING:\s+(.*)").unwrap(),
            colors: vec![Style::new().yellow(), Style::new().yellow().dim()],
            skip: false,
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
        },
        GrcatConfigEntry {
            regex: Regex::new(r"\d+").unwrap(),
            colors: vec![Style::new().green()],
            skip: false,
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
        },
    ]
}

fn bench_literal_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("literal_patterns");

    for size in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(*size, 3);
        let rules = create_literal_rules();

        group.bench_with_input(BenchmarkId::new("regex", size), size, |b, _| {
            b.iter(|| {
                let mut reader = Cursor::new(test_data.as_bytes());
                let mut writer = Vec::new();
                colorize_regex(&mut reader, &mut writer, &rules).unwrap();
                black_box(writer);
            });
        });

        group.bench_with_input(BenchmarkId::new("simd", size), size, |b, _| {
            b.iter(|| {
                let mut reader = Cursor::new(test_data.as_bytes());
                let mut writer = Vec::new();
                colorize_simd(&mut reader, &mut writer, &rules).unwrap();
                black_box(writer);
            });
        });
    }

    group.finish();
}

fn bench_complex_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_patterns");

    for size in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(*size, 3);
        let rules = create_regex_rules();

        group.bench_with_input(BenchmarkId::new("regex", size), size, |b, _| {
            b.iter(|| {
                let mut reader = Cursor::new(test_data.as_bytes());
                let mut writer = Vec::new();
                colorize_regex(&mut reader, &mut writer, &rules).unwrap();
                black_box(writer);
            });
        });

        group.bench_with_input(BenchmarkId::new("simd", size), size, |b, _| {
            b.iter(|| {
                let mut reader = Cursor::new(test_data.as_bytes());
                let mut writer = Vec::new();
                colorize_simd(&mut reader, &mut writer, &rules).unwrap();
                black_box(writer);
            });
        });
    }

    group.finish();
}

fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");

    // Mix of literal and regex rules
    let mut mixed_rules = create_literal_rules();
    mixed_rules.extend(vec![GrcatConfigEntry {
        regex: Regex::new(r"\b[A-Z]{3,}\b").unwrap(),
        colors: vec![Style::new().magenta()],
        skip: false,
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
    }]);

    let test_data = generate_test_data(5000, 3);

    group.bench_function("regex", |b| {
        b.iter(|| {
            let mut reader = Cursor::new(test_data.as_bytes());
            let mut writer = Vec::new();
            colorize_regex(&mut reader, &mut writer, &mixed_rules).unwrap();
            black_box(writer);
        });
    });

    group.bench_function("simd", |b| {
        b.iter(|| {
            let mut reader = Cursor::new(test_data.as_bytes());
            let mut writer = Vec::new();
            colorize_simd(&mut reader, &mut writer, &mixed_rules).unwrap();
            black_box(writer);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_literal_patterns,
    bench_complex_patterns,
    bench_mixed_workload
);
criterion_main!(benches);
