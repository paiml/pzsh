//! Shell initialization example
//!
//! Run with: `cargo run --example shell_init`

use pzsh::ShellType;
use pzsh::config::CompiledConfig;
use pzsh::shell::generate_init;

fn main() {
    println!("=== pzsh Shell Initialization ===\n");

    // Create a sample configuration
    let mut config = CompiledConfig {
        colors_enabled: true,
        aliases: [
            ("ll".to_string(), "ls -la".to_string()),
            ("gs".to_string(), "git status".to_string()),
            ("gp".to_string(), "git push".to_string()),
        ]
        .into_iter()
        .collect(),
        env: std::iter::once(("EDITOR".to_string(), "vim".to_string())).collect(),
        plugins_enabled: vec!["git".to_string()],
        ..CompiledConfig::default()
    };

    // Generate zsh init script
    println!("=== ZSH Init Script ===");
    println!("Source with: eval \"$(pzsh compile)\"\n");

    let zsh_init = generate_init(ShellType::Zsh, config.clone());

    // Print first 50 lines
    for (i, line) in zsh_init.lines().take(50).enumerate() {
        println!("{:3}: {}", i + 1, line);
    }
    println!("... ({} total lines)\n", zsh_init.lines().count());

    // Generate bash init script
    println!("=== BASH Init Script ===");
    config.shell_type = ShellType::Bash;
    let bash_init = generate_init(ShellType::Bash, config);

    // Print first 30 lines
    for (i, line) in bash_init.lines().take(30).enumerate() {
        println!("{:3}: {}", i + 1, line);
    }
    println!("... ({} total lines)\n", bash_init.lines().count());

    // Performance note
    println!("=== Performance ===");
    println!("The generated script is designed for O(1) sourcing:");
    println!("  - No subprocess calls ($() or ``)");
    println!("  - Pre-computed aliases and env vars");
    println!("  - Lazy-loaded completions (compinit -C)");
    println!("  - Cached git info function");
}
