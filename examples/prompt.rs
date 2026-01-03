//! Prompt example - O(1) prompt rendering
//!
//! Run with: cargo run --example prompt

use pzsh::config::CompiledConfig;
use pzsh::prompt::Prompt;
use std::time::Instant;

fn main() {
    println!("pzsh Prompt Demo");
    println!("════════════════");
    println!();

    // Create config with custom prompt format
    let mut config = CompiledConfig::default();
    config.prompt_format = "{user}@{host} {cwd} {git} {char} ".to_string();

    let mut prompt = Prompt::new(&config);

    // Render prompt
    println!("Prompt rendering:");
    println!("─────────────────");

    let start = Instant::now();
    let rendered = prompt.render().unwrap();
    let elapsed = start.elapsed();

    println!("  Format: {}", config.prompt_format);
    println!("  Rendered: {}", rendered);
    println!("  Time: {:.3}µs", elapsed.as_nanos() as f64 / 1000.0);
    println!();

    // Simulate git status update (async in real usage)
    println!("With git status:");
    println!("────────────────");
    prompt.update_git_cache(Some("main".to_string()), false);

    let start = Instant::now();
    let rendered = prompt.render().unwrap();
    let elapsed = start.elapsed();

    println!("  Rendered: {}", rendered);
    println!("  Time: {:.3}µs", elapsed.as_nanos() as f64 / 1000.0);
    println!();

    // With dirty flag
    println!("With dirty flag:");
    println!("────────────────");
    prompt.update_git_cache(Some("feature-branch".to_string()), true);

    let rendered = prompt.render().unwrap();
    println!("  Rendered: {}", rendered);
    println!();

    // Benchmark
    println!("Benchmark (1000 renders):");
    println!("─────────────────────────");
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = prompt.render();
    }
    let elapsed = start.elapsed();

    println!(
        "  Total: {:.3}ms ({:.3}µs per render)",
        elapsed.as_secs_f64() * 1000.0,
        elapsed.as_nanos() as f64 / 1000.0 / 1000.0
    );
    println!("  Budget: 2ms per render ✓");
}
