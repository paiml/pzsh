//! Completion benchmarks for pzsh.
//!
//! Wall-clock completion cost is tracked HERE and not in a `cargo test --lib`
//! unit test. A `Duration` budget asserted inside the unit suite measures how
//! contended the runner is: `completion::tests::test_completion_performance`
//! asserted `< 500ms` and failed the clean-room sweep at 529.5ms while the
//! identical code measured 85ms on an idle box. Widening the budget until it
//! survives the worst contended case leaves it too wide to catch a regression.
//!
//! WHAT THIS BENCH IS AND IS NOT: criterion reports a regression, it does not
//! fail the build on one, and the nightly bench lane does not gate merges. This
//! is a trend signal for a human to read. The *enforced* guarantee about
//! completion cost is the algorithmic-work assertion in
//! `completion::tests::test_completion_work_scales_subquadratically`, which is
//! deterministic and therefore runs in the gating lane.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use pzsh::completion::{AliasCompleter, CompletionEngine, default_engine};
use std::hint::black_box;
use std::sync::Arc;

/// Build an alias table of `n` entries that all share the "alias" prefix.
fn alias_table(n: usize) -> Arc<ahash::AHashMap<String, String>> {
    let mut aliases = ahash::AHashMap::new();
    for i in 0..n {
        aliases.insert(format!("alias{i}"), format!("command{i}"));
    }
    Arc::new(aliases)
}

/// Completion over alias tables of increasing size.
///
/// The interesting shape is how the curve grows across `n`, not the absolute
/// numbers, which depend on the machine.
fn bench_alias_completion(c: &mut Criterion) {
    let mut group = c.benchmark_group("completion_alias_table");

    for n in [100_usize, 1_000, 4_000] {
        let mut engine = CompletionEngine::new();
        engine.add_provider(AliasCompleter::new(alias_table(n)));

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(engine.complete(black_box("alias"), 5)));
        });
    }

    group.finish();
}

/// Completion through the full default provider stack (aliases + env + paths).
fn bench_default_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("completion_default_engine");

    let engine = default_engine(alias_table(1_000));
    group.bench_function("alias_prefix_1000", |b| {
        b.iter(|| black_box(engine.complete(black_box("alias"), 5)));
    });

    // The miss case, for contrast: nothing matches, so the cost is the
    // provider walk without the candidate construction.
    group.bench_function("no_match_1000", |b| {
        b.iter(|| black_box(engine.complete(black_box("zzzzz"), 5)));
    });

    group.finish();
}

criterion_group!(benches, bench_alias_completion, bench_default_engine);
criterion_main!(benches);
