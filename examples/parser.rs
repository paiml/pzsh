//! Parser example - O(1) command parsing with LRU cache
//!
//! Run with: cargo run --example parser

use pzsh::config::CompiledConfig;
use pzsh::parser::{ParsedCommand, Parser};
use std::time::Instant;

fn main() {
    println!("pzsh Parser Demo");
    println!("════════════════");
    println!();

    // Create config with aliases
    let mut config = CompiledConfig::default();
    config
        .aliases
        .insert("ll".to_string(), "ls -la".to_string());
    config
        .aliases
        .insert("gs".to_string(), "git status".to_string());
    config
        .aliases
        .insert("gp".to_string(), "git push".to_string());

    let mut parser = Parser::new(&config);

    // Demo parsing different command types
    let commands = vec![
        "ls -la /tmp",          // Simple command
        "ll",                   // Alias
        "cd /home",             // Builtin
        "git commit -m 'test'", // Simple command
        "gs",                   // Alias
        "",                     // Empty
    ];

    println!("Parsing commands:");
    println!("─────────────────");
    for cmd in &commands {
        let start = Instant::now();
        let result = parser.parse(cmd).unwrap();
        let elapsed = start.elapsed();

        let kind = match &result {
            ParsedCommand::Simple { command, args } => {
                format!("Simple({} {:?})", command, args)
            }
            ParsedCommand::Alias { name, expansion } => {
                format!("Alias({} -> {})", name, expansion)
            }
            ParsedCommand::Builtin { name, args } => {
                format!("Builtin({} {:?})", name, args)
            }
            ParsedCommand::Empty => "Empty".to_string(),
        };

        println!(
            "  {:25} => {:40} [{:.3}µs]",
            format!("\"{}\"", cmd),
            kind,
            elapsed.as_nanos() as f64 / 1000.0
        );
    }

    println!();
    println!("Cache demonstration:");
    println!("────────────────────");

    // First parse (cache miss)
    let cmd = "ls -la /home";
    let start = Instant::now();
    let _ = parser.parse(cmd);
    let first = start.elapsed();

    // Second parse (cache hit)
    let start = Instant::now();
    let _ = parser.parse(cmd);
    let second = start.elapsed();

    println!(
        "  First parse:  {:.3}µs (cache miss)",
        first.as_nanos() as f64 / 1000.0
    );
    println!(
        "  Second parse: {:.3}µs (cache hit)",
        second.as_nanos() as f64 / 1000.0
    );
    println!("  Cache entries: {}", parser.cache_len());
}
