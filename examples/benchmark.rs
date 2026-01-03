//! Benchmark example - measure startup time
//!
//! Run with: cargo run --example benchmark

use pzsh::config::CompiledConfig;
use pzsh::Pzsh;
use std::time::Instant;

fn main() {
    const ITERATIONS: u32 = 100;

    println!("pzsh Startup Benchmark");
    println!("══════════════════════");
    println!();

    let mut times = Vec::with_capacity(ITERATIONS as usize);

    // Warmup
    for _ in 0..10 {
        let config = CompiledConfig::default();
        let _ = Pzsh::new(config);
    }

    // Benchmark
    for _ in 0..ITERATIONS {
        let config = CompiledConfig::default();
        let start = Instant::now();
        let _ = Pzsh::new(config);
        times.push(start.elapsed());
    }

    // Sort for percentiles
    times.sort();

    let min = times[0];
    let max = times[times.len() - 1];
    let sum: std::time::Duration = times.iter().sum();
    let mean = sum / ITERATIONS;
    let p50 = times[(times.len() as f64 * 0.50) as usize];
    let p95 = times[(times.len() as f64 * 0.95) as usize];
    let p99 = times[(times.len() as f64 * 0.99) as usize];

    println!("Results ({} iterations):", ITERATIONS);
    println!("  min:  {:>8.3}ms", min.as_secs_f64() * 1000.0);
    println!("  max:  {:>8.3}ms", max.as_secs_f64() * 1000.0);
    println!("  mean: {:>8.3}ms", mean.as_secs_f64() * 1000.0);
    println!("  p50:  {:>8.3}ms", p50.as_secs_f64() * 1000.0);
    println!("  p95:  {:>8.3}ms", p95.as_secs_f64() * 1000.0);
    println!("  p99:  {:>8.3}ms", p99.as_secs_f64() * 1000.0);
    println!();

    let passed = p99.as_millis() < 10;
    if passed {
        println!("✓ PASSED: p99 < 10ms budget");
    } else {
        println!("✗ FAILED: p99 >= 10ms budget");
    }
}
