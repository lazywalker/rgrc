use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rgrc::grc::CompiledRegex;

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

criterion_group!(
    benches,
    benchmark_lookahead_patterns,
    benchmark_lookbehind_patterns,
    benchmark_complex_pattern,
    benchmark_ls_file_size,
    benchmark_find_all_matches,
    benchmark_docker_ps_pattern,
    benchmark_no_lookaround
);
criterion_main!(benches);
