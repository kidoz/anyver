// Criterion benchmarks for the Rust core.
//
// Run: `cargo bench --features bench`
// or:  `just bench-rust`
//
// Measures pure Rust paths (no PyO3 marshaling) so regressions in
// parsing/comparison algorithms surface cleanly.
//
// Located under `benchmarks/` (not Cargo's default `benches/`) for symmetry
// with Python's `tests/benchmarks/`; wired via `[[bench]] path = ...`.

use std::hint::black_box;

use anyver::__bench::{Parsed, autodetect, compare, eco_from_str, parse_for, parse_generic};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

const SEMVER_SIMPLE: &str = "1.2.3";
const SEMVER_PRERELEASE: &str = "1.0.0-alpha.1+build.42";
const PEP440_COMPLEX: &str = "1!2.3.4.post1.dev2+local.build";
const DEBIAN_EPOCH: &str = "2:1.0.2l-2+deb9u1";
const RPM_KERNEL: &str = "5.14.0-362.13.1.el9_3";
const MAVEN_LIFECYCLE: &str = "4.0.0-alpha-13-SNAPSHOT";

fn bench_parse_generic(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_generic");
    for (label, input) in [
        ("semver_simple", SEMVER_SIMPLE),
        ("semver_prerelease", SEMVER_PRERELEASE),
        ("pep440_complex", PEP440_COMPLEX),
        ("debian_epoch", DEBIAN_EPOCH),
        ("rpm_kernel", RPM_KERNEL),
        ("maven_lifecycle", MAVEN_LIFECYCLE),
    ] {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), input, |b, s| {
            b.iter(|| parse_generic(black_box(s)));
        });
    }
    group.finish();
}

fn bench_parse_for_ecosystem(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_for_ecosystem");
    for (label, input, eco_name) in [
        ("semver", SEMVER_PRERELEASE, "semver"),
        ("pep440", PEP440_COMPLEX, "pep440"),
        ("debian", DEBIAN_EPOCH, "debian"),
        ("rpm", RPM_KERNEL, "rpm"),
        ("maven", MAVEN_LIFECYCLE, "maven"),
    ] {
        let eco = eco_from_str(eco_name).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(label), input, |b, s| {
            b.iter(|| parse_for(eco, black_box(s)).unwrap());
        });
    }
    group.finish();
}

fn bench_compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare");

    let pairs: &[(&str, &str, &str, &str)] = &[
        ("semver_prerelease_ids", "semver", "1.0.0-alpha.1", "1.0.0-alpha.beta"),
        ("debian_deb_revision", "debian", "1.0.2l-2+deb9u1", "1.0.2l-2+deb9u3"),
        ("rpm_kernel", "rpm", "5.14.0-362.13.1.el9_3", "5.14.0-427.31.1.el9_4"),
        ("maven_qualifier", "maven", "4.0.0-alpha-13", "4.0.0-beta-3"),
    ];

    for (label, eco_name, a, b) in pairs.iter().copied() {
        let eco = eco_from_str(eco_name).unwrap();
        let pa = parse_for(eco, a).unwrap();
        let pb = parse_for(eco, b).unwrap();
        group.bench_function(label, |bn| {
            bn.iter(|| compare(eco, black_box(&pa), black_box(&pb)));
        });
    }
    group.finish();
}

fn bench_autodetect(c: &mut Criterion) {
    let mut group = c.benchmark_group("autodetect");
    for (label, input) in [
        ("plain", "1.2.3"),
        ("pep440_bang", "1!0.1"),
        ("debian_tilde", "1.0~alpha"),
        ("rpm_caret", "1.0^git1"),
        ("rpm_fedora", "252-14.fc40"),
        ("go_incompatible", "v2.0.0+incompatible"),
        ("maven_snapshot", "1.0-SNAPSHOT"),
        ("alpine_rc", "1.0_rc1"),
        ("calver_year", "2024.01"),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), input, |b, s| {
            b.iter(|| autodetect(black_box(s)));
        });
    }
    group.finish();
}

fn bench_sort(c: &mut Criterion) {
    // Sort 1000 parsed SemVer versions — mimics a realistic "pick latest" workload.
    let eco = eco_from_str("semver").unwrap();
    let parsed: Vec<Parsed> =
        (0..1000).map(|i| parse_for(eco, &format!("1.{}.{}", i / 10, i % 10)).unwrap()).collect();

    c.bench_function("sort_1000_semver", |b| {
        b.iter_batched(
            || parsed.clone(),
            |mut v| {
                v.sort_by(|a, b| match compare(eco, a, b) {
                    x if x < 0 => std::cmp::Ordering::Less,
                    x if x > 0 => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                });
                black_box(v);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_parse_generic,
    bench_parse_for_ecosystem,
    bench_compare,
    bench_autodetect,
    bench_sort
);
criterion_main!(benches);
