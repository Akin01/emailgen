//! Benchmark suite for email generation performance.
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use emailgen::{EmailBloomFilter, EmailGenerator};

/// Benchmark basic email generation
fn bench_basic_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_generation");

    group.bench_function("generate_single_email", |b| {
        let mut gen = EmailGenerator::new();
        b.iter(|| gen.generate())
    });

    group.bench_function("generate_with_custom_names", |b| {
        let names: Vec<String> = (0..100).map(|i| format!("Name{}", i)).collect();
        let domains: Vec<String> = vec!["example.com".to_string()];
        let mut gen = EmailGenerator::with_names_and_domains(names, domains);

        b.iter(|| gen.generate())
    });

    group.finish();
}

/// Benchmark bulk generation
fn bench_bulk_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_generation");
    group.throughput(Throughput::Elements(1));

    group.bench_function("generate_1000_emails", |b| {
        b.iter(|| {
            let mut gen = EmailGenerator::new();
            gen.generate_many(1000)
        })
    });

    group.bench_function("generate_10000_emails", |b| {
        b.iter(|| {
            let mut gen = EmailGenerator::new();
            gen.generate_many(10000)
        })
    });

    group.finish();
}

/// Benchmark Bloom filter operations
fn bench_bloom_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("bloom_filter");

    group.bench_function("bloom_insert_1m_items", |b| {
        b.iter(|| {
            let mut bloom = EmailBloomFilter::new(1_000_000, 0.01);
            for i in 0..10_000 {
                bloom.insert(&format!("email{}@example.com", i));
            }
            black_box(&bloom);
        })
    });

    group.bench_function("bloom_check_operation", |b| {
        let mut bloom = EmailBloomFilter::new(1_000_000, 0.01);
        for i in 0..100_000 {
            bloom.insert(&format!("email{}@example.com", i));
        }

        b.iter(|| black_box(bloom.contains("test@example.com")))
    });

    group.bench_function("bloom_check_and_insert", |b| {
        let mut bloom = EmailBloomFilter::new(1_000_000, 0.01);

        b.iter(|| black_box(bloom.check_and_insert(&format!("test{}@example.com", black_box(1)))))
    });

    group.finish();
}

/// Benchmark with different capacities
fn bench_different_capacities(c: &mut Criterion) {
    let mut group = c.benchmark_group("capacity_benchmark");

    for capacity in [10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(
            format!("generate_with_capacity_{}", capacity),
            capacity,
            |b, &cap| {
                b.iter(|| {
                    let mut gen = EmailGenerator::new().with_capacity(cap, 0.01);
                    gen.generate_many(1000)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    group.bench_function("bloom_memory_1m", |b| {
        b.iter(|| {
            let bloom = EmailBloomFilter::new(1_000_000, 0.01);
            black_box(bloom.memory_usage_mb())
        })
    });

    group.bench_function("bloom_memory_10m", |b| {
        b.iter(|| {
            let bloom = EmailBloomFilter::new(10_000_000, 0.01);
            black_box(bloom.memory_usage_mb())
        })
    });

    group.finish();
}

/// Benchmark pattern generation
fn bench_pattern_generation(c: &mut Criterion) {
    use emailgen::{EmailPattern, GeneratorConfig};

    let mut group = c.benchmark_group("pattern_generation");

    group.bench_function("first_last_pattern", |b| {
        let mut gen = EmailGenerator::new().with_config(GeneratorConfig {
            patterns: vec![(EmailPattern::FirstLast, 100)],
            ..Default::default()
        });
        b.iter(|| gen.generate())
    });

    group.bench_function("mixed_patterns", |b| {
        let mut gen = EmailGenerator::new().with_config(GeneratorConfig {
            patterns: vec![
                (EmailPattern::FirstLast, 35),
                (EmailPattern::FirstLastNoSep, 25),
                (EmailPattern::FirstInitialLast, 20),
                (EmailPattern::FirstUnderscoreLast, 10),
                (EmailPattern::FirstNumber, 10),
            ],
            ..Default::default()
        });
        b.iter(|| gen.generate())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_basic_generation,
    bench_bulk_generation,
    bench_bloom_filter,
    bench_different_capacities,
    bench_memory_usage,
    bench_pattern_generation,
);

criterion_main!(benches);
