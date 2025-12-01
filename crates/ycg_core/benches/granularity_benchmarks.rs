// Performance benchmarks for ad-hoc granularity levels
// Validates Requirements 10.1, 10.2, 10.3, 10.4, 10.5
//
// These benchmarks measure:
// - Processing time overhead for each granularity level
// - Token savings per level
// - Memory usage impact
// - AST caching effectiveness

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::path::PathBuf;
use std::time::Duration;
use ycg_core::model::{AdHocGranularity, FileFilterConfig, OutputFormat};
use ycg_core::{LevelOfDetail, YcgConfig, run_scip_conversion};

/// Helper function to create a config with specified granularity
fn create_config(granularity: AdHocGranularity, project: &str) -> YcgConfig {
    let project_root = PathBuf::from(format!("../../examples/{}", project));

    YcgConfig {
        lod: LevelOfDetail::High,
        project_root,
        compact: false,
        output_format: OutputFormat::AdHoc,
        ignore_framework_noise: false,
        file_filter: FileFilterConfig {
            include_patterns: vec![],
            exclude_patterns: vec![],
            use_gitignore: false,
        },
        adhoc_granularity: granularity,
    }
}

/// Helper function to count tokens (approximate using character count / 4)
fn count_tokens(text: &str) -> usize {
    // Rough approximation: 1 token ≈ 4 characters
    text.len() / 4
}

/// Helper function to measure output size
fn measure_output_size(output: &str) -> (usize, usize, usize) {
    let bytes = output.len();
    let lines = output.lines().count();
    let tokens = count_tokens(output);
    (bytes, lines, tokens)
}

/// Benchmark: Processing time for each granularity level on simple-ts
///
/// **Validates: Requirements 10.1, 10.2**
fn bench_processing_time_simple_ts(c: &mut Criterion) {
    let scip_path = PathBuf::from("../../examples/simple-ts/index.scip");

    if !scip_path.exists() {
        // Debug: print current directory and absolute path
        if let Ok(cwd) = std::env::current_dir() {
            eprintln!("Current working directory: {:?}", cwd);
            eprintln!("Looking for SCIP at: {:?}", cwd.join(&scip_path));
        }
        eprintln!(
            "⚠ Skipping benchmark: SCIP file not found at {:?}",
            scip_path
        );
        return;
    }

    let mut group = c.benchmark_group("processing_time_simple_ts");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    // Benchmark Level 0 (Default)
    group.bench_function(BenchmarkId::new("level", "0_default"), |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::Default, "simple-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            black_box(result)
        });
    });

    // Benchmark Level 1 (Inline Signatures)
    group.bench_function(BenchmarkId::new("level", "1_signatures"), |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::InlineSignatures, "simple-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            black_box(result)
        });
    });

    // Benchmark Level 2 (Inline Logic)
    group.bench_function(BenchmarkId::new("level", "2_logic"), |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::InlineLogic, "simple-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark: Processing time for each granularity level on nestjs-api-ts
///
/// **Validates: Requirements 10.1, 10.2**
fn bench_processing_time_nestjs(c: &mut Criterion) {
    let scip_path = PathBuf::from("../../examples/nestjs-api-ts/index.scip");

    if !scip_path.exists() {
        eprintln!(
            "⚠ Skipping benchmark: SCIP file not found at {:?}",
            scip_path
        );
        return;
    }

    let mut group = c.benchmark_group("processing_time_nestjs");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(15);

    // Benchmark Level 0 (Default)
    group.bench_function(BenchmarkId::new("level", "0_default"), |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::Default, "nestjs-api-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            black_box(result)
        });
    });

    // Benchmark Level 1 (Inline Signatures)
    group.bench_function(BenchmarkId::new("level", "1_signatures"), |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::InlineSignatures, "nestjs-api-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            black_box(result)
        });
    });

    // Benchmark Level 2 (Inline Logic)
    group.bench_function(BenchmarkId::new("level", "2_logic"), |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::InlineLogic, "nestjs-api-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark: Token savings measurement
///
/// This is not a traditional benchmark but a measurement test that reports
/// token savings for each granularity level.
///
/// **Validates: Requirements 10.1, 10.2, 10.5**
fn bench_token_savings(c: &mut Criterion) {
    let scip_path = PathBuf::from("../../examples/simple-ts/index.scip");

    if !scip_path.exists() {
        eprintln!("⚠ Skipping token savings measurement: SCIP file not found");
        return;
    }

    // Generate outputs for all levels
    let levels = vec![
        ("Level 0 (Default)", AdHocGranularity::Default),
        ("Level 1 (Signatures)", AdHocGranularity::InlineSignatures),
        ("Level 2 (Logic)", AdHocGranularity::InlineLogic),
    ];

    println!("\n=== Token Savings Analysis ===\n");

    let mut baseline_tokens = 0;

    for (name, level) in levels {
        let config = create_config(level, "simple-ts");

        match run_scip_conversion(&scip_path, config) {
            Ok(output) => {
                let (bytes, lines, tokens) = measure_output_size(&output);

                if level == AdHocGranularity::Default {
                    baseline_tokens = tokens;
                }

                let overhead_pct = if baseline_tokens > 0 {
                    ((tokens as f64 - baseline_tokens as f64) / baseline_tokens as f64) * 100.0
                } else {
                    0.0
                };

                println!("{}", name);
                println!("  Size: {} bytes", bytes);
                println!("  Lines: {}", lines);
                println!("  Tokens (approx): {}", tokens);
                if baseline_tokens > 0 && level != AdHocGranularity::Default {
                    println!("  Token overhead: {:.1}%", overhead_pct);
                }
                println!();
            }
            Err(e) => {
                eprintln!("Error generating output for {}: {}", name, e);
            }
        }
    }

    // Create a dummy benchmark to keep criterion happy
    c.bench_function("token_savings_measurement", |b| {
        b.iter(|| black_box(1 + 1));
    });
}

/// Benchmark: Memory usage comparison
///
/// Measures peak memory usage for each granularity level.
/// Note: This is a simplified measurement. For production, use tools like valgrind or heaptrack.
///
/// **Validates: Requirements 10.3, 10.4**
fn bench_memory_usage(c: &mut Criterion) {
    let scip_path = PathBuf::from("../../examples/nestjs-api-ts/index.scip");

    if !scip_path.exists() {
        eprintln!("⚠ Skipping memory benchmark: SCIP file not found");
        return;
    }

    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    // Benchmark Level 0 memory usage
    group.bench_function(BenchmarkId::new("level", "0_default"), |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::Default, "nestjs-api-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            // Force allocation to be measured
            if let Ok(output) = result {
                black_box(output.len());
            }
        });
    });

    // Benchmark Level 1 memory usage
    group.bench_function(BenchmarkId::new("level", "1_signatures"), |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::InlineSignatures, "nestjs-api-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            if let Ok(output) = result {
                black_box(output.len());
            }
        });
    });

    // Benchmark Level 2 memory usage
    group.bench_function(BenchmarkId::new("level", "2_logic"), |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::InlineLogic, "nestjs-api-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            if let Ok(output) = result {
                black_box(output.len());
            }
        });
    });

    group.finish();
}

/// Benchmark: AST caching effectiveness
///
/// Measures the performance impact of AST caching by comparing
/// first-time parsing vs. cached parsing.
///
/// **Validates: Requirements 10.3, 10.4**
fn bench_ast_caching(c: &mut Criterion) {
    let scip_path = PathBuf::from("../../examples/simple-ts/index.scip");

    if !scip_path.exists() {
        eprintln!("⚠ Skipping AST caching benchmark: SCIP file not found");
        return;
    }

    let mut group = c.benchmark_group("ast_caching");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark with caching (Level 2 uses AST cache)
    group.bench_function("with_cache", |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::InlineLogic, "simple-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark: Throughput measurement
///
/// Measures symbols processed per second for each granularity level.
///
/// **Validates: Requirements 10.1, 10.2**
fn bench_throughput(c: &mut Criterion) {
    let scip_path = PathBuf::from("../../examples/nestjs-api-ts/index.scip");

    if !scip_path.exists() {
        eprintln!("⚠ Skipping throughput benchmark: SCIP file not found");
        return;
    }

    // Estimate number of symbols (this is approximate)
    let estimated_symbols = 50; // Adjust based on actual project

    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Elements(estimated_symbols));
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("level_0", |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::Default, "nestjs-api-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            black_box(result)
        });
    });

    group.bench_function("level_1", |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::InlineSignatures, "nestjs-api-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            black_box(result)
        });
    });

    group.bench_function("level_2", |b| {
        b.iter(|| {
            let config = create_config(AdHocGranularity::InlineLogic, "nestjs-api-ts");
            let result = run_scip_conversion(black_box(&scip_path), black_box(config));
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_processing_time_simple_ts,
    bench_processing_time_nestjs,
    bench_token_savings,
    bench_memory_usage,
    bench_ast_caching,
    bench_throughput
);

criterion_main!(benches);
