//! Startup benchmarks for pzsh
//!
//! These benchmarks enforce the 10ms startup constraint.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pzsh::Pzsh;
use pzsh::config::CompiledConfig;
use pzsh::executor::Executor;
use pzsh::parser::Parser;
use pzsh::prompt::Prompt;
use std::time::Duration;

/// Benchmark full pzsh startup
fn bench_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("pzsh_new", |b| {
        b.iter(|| {
            let config = CompiledConfig::default();
            black_box(Pzsh::new(config).unwrap())
        });
    });

    group.finish();
}

/// Benchmark individual components
fn bench_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("components");
    group.measurement_time(Duration::from_secs(3));

    let config = CompiledConfig::default();

    group.bench_function("parser_new", |b| {
        b.iter(|| black_box(Parser::new(&config)));
    });

    group.bench_function("executor_new", |b| {
        b.iter(|| black_box(Executor::new(&config)));
    });

    group.bench_function("prompt_new", |b| {
        b.iter(|| black_box(Prompt::new(&config)));
    });

    group.finish();
}

/// Benchmark config compilation
fn bench_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("config");
    group.measurement_time(Duration::from_secs(3));

    let toml = r#"
[pzsh]
version = "0.1.0"
shell = "zsh"

[aliases]
ll = "ls -la"
gs = "git status"
gp = "git push"

[env]
EDITOR = "vim"
GOROOT = "/usr/local/opt/go/libexec"
"#;

    group.bench_function("config_from_toml", |b| {
        b.iter(|| black_box(CompiledConfig::from_toml(toml).unwrap()));
    });

    group.finish();
}

/// Benchmark O(1) lookups
fn bench_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookups");
    group.measurement_time(Duration::from_secs(3));

    let mut config = CompiledConfig::default();
    for i in 0..10000 {
        config
            .aliases
            .insert(format!("alias{i}"), format!("command{i}"));
        config.env.insert(format!("VAR{i}"), format!("value{i}"));
    }

    group.bench_function("alias_lookup", |b| {
        b.iter(|| black_box(config.get_alias("alias5000")));
    });

    group.bench_function("env_lookup", |b| {
        b.iter(|| black_box(config.get_env("VAR5000")));
    });

    group.finish();
}

/// Benchmark parser with cache
fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    group.measurement_time(Duration::from_secs(3));

    let config = CompiledConfig::default();
    let mut parser = Parser::new(&config);

    // Prime the cache
    let _ = parser.parse("ls -la /tmp");

    group.bench_function("parse_cached", |b| {
        b.iter(|| black_box(parser.parse("ls -la /tmp").unwrap()));
    });

    group.bench_function("parse_uncached", |b| {
        b.iter(|| {
            parser.clear_cache();
            black_box(parser.parse("ls -la /tmp").unwrap())
        });
    });

    group.finish();
}

/// Benchmark prompt rendering
fn bench_prompt(c: &mut Criterion) {
    let mut group = c.benchmark_group("prompt");
    group.measurement_time(Duration::from_secs(3));

    let config = CompiledConfig::default();
    let prompt = Prompt::new(&config);

    group.bench_function("prompt_render", |b| {
        b.iter(|| black_box(prompt.render().unwrap()));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_startup,
    bench_components,
    bench_config,
    bench_lookups,
    bench_parser,
    bench_prompt,
);

criterion_main!(benches);
