use criterion::{Criterion, criterion_group, criterion_main};
use rgrc::grc::CompiledRegex;
use std::hint::black_box;

fn benchmark_lookahead_patterns(c: &mut Criterion) {
    let pattern = r"\d+(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "123 456 789 012 345 678 901 234 567 890 123 456 789 012 345 678 901 234 567 890";

    c.bench_function("lookahead_boundary", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_lookbehind_patterns(c: &mut Criterion) {
    let pattern = r"(?<=\s)-[\w\d]+(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = " -verbose -output -debug -test -flag -option -param -value -arg -switch";

    c.bench_function("lookbehind_options", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_complex_pattern(c: &mut Criterion) {
    let pattern = r"(?<=[,<])[^,]+?(?=[,>])";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "<value1>,<value2>,<value3>,<value4>,<value5>,<value6>,<value7>,<value8>";

    c.bench_function("character_class_lookbehind", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_ls_file_size(c: &mut Criterion) {
    let pattern = r"\s+(\d{7}|\d(?:[,.]?\d+)?[KM])(?=\s[A-Z][a-z]{2}\s)";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "     1234567 Nov 30 file1.txt     123K Nov 29 file2.txt     45.6M Nov 28 file3.txt";

    c.bench_function("ls_file_size", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_find_all_matches(c: &mut Criterion) {
    let pattern = r"\d+(?=\s|$)";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "123 456 789 012 345 678 901 234 567 890 123 456 789 012 345 678 901 234 567 890";

    c.bench_function("find_all_numbers", |b| {
        b.iter(|| {
            // Just check if pattern matches repeatedly
            regex.is_match(black_box(text))
        });
    });
}

fn benchmark_docker_ps_pattern(c: &mut Criterion) {
    // This pattern from conf.dockerps line 5
    let pattern = r".*(?=(?:Up|Exited|Created))";
    let regex = CompiledRegex::new(pattern).unwrap();
    // Make sure text actually matches - need to have status keyword
    let text = "Up 2 hours";

    c.bench_function("docker_ps_status", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_no_lookaround(c: &mut Criterion) {
    let pattern = r"\d+";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "123 456 789 012 345 678 901 234 567 890";

    c.bench_function("no_lookaround_baseline", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_fast_path_whitespace(c: &mut Criterion) {
    let pattern = r"\d+(?=\s)";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "123 456 789 012 345 678 901 234 567 890";

    c.bench_function("fast_path_whitespace", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_fast_path_end_of_line(c: &mut Criterion) {
    let pattern = r"\d+(?=$)";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "123 456 789";

    c.bench_function("fast_path_end_of_line", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_fast_path_month(c: &mut Criterion) {
    let pattern = r"\d+(?=\s[A-Z][a-z]{2}\s)";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "123 Nov 30 456 Dec 25 789 Jan 01";

    c.bench_function("fast_path_month", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_fast_path_colon_slash(c: &mut Criterion) {
    let pattern = r"\w+(?=[:/])";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "http://example.com:8080/path";

    c.bench_function("fast_path_colon_slash", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_fast_path_uppercase(c: &mut Criterion) {
    let pattern = r"\w+(?=\s[A-Z])";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "test WORD another VALUE";

    c.bench_function("fast_path_uppercase", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_fast_path_ipv4(c: &mut Criterion) {
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "192.168.1.1 10.0.0.1 172.16.0.1";

    c.bench_function("fast_path_ipv4", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_fast_path_size_unit(c: &mut Criterion) {
    let pattern = r"\d+(?=[KMG]B?)";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "1024KB 256MB 16GB 512K 2M 1G";

    c.bench_function("fast_path_size_unit", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

fn benchmark_fast_path_size_unit_simple(c: &mut Criterion) {
    let pattern = r"\d+(?=[KMGT])";
    let regex = CompiledRegex::new(pattern).unwrap();
    let text = "100K 50M 2G 1T";

    c.bench_function("fast_path_size_unit_simple", |b| {
        b.iter(|| regex.is_match(black_box(text)));
    });
}

criterion_group!(
    benches,
    benchmark_lookahead_patterns,
    benchmark_lookbehind_patterns,
    benchmark_complex_pattern,
    benchmark_ls_file_size,
    benchmark_find_all_matches,
    benchmark_docker_ps_pattern,
    benchmark_no_lookaround,
    benchmark_fast_path_whitespace,
    benchmark_fast_path_end_of_line,
    benchmark_fast_path_month,
    benchmark_fast_path_colon_slash,
    benchmark_fast_path_uppercase,
    benchmark_fast_path_ipv4,
    benchmark_fast_path_size_unit,
    benchmark_fast_path_size_unit_simple
);
criterion_main!(benches);
