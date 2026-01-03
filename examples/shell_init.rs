//! Shell initialization example
//!
//! Run with: cargo run --example shell_init

use pzsh::config::CompiledConfig;
use pzsh::shell::generate_init;
use pzsh::ShellType;

fn main() {
    println!("=== pzsh Shell Initialization ===\n");

    // Create a sample configuration
    let mut config = CompiledConfig::default();
    config.colors_enabled = true;
    config.aliases.insert("ll".to_string(), "ls -la".to_string());
    config.aliases.insert("gs".to_string(), "git status".to_string());
    config.aliases.insert("gp".to_string(), "git push".to_string());
    config.env.insert("EDITOR".to_string(), "vim".to_string());
    config.plugins_enabled = vec!["git".to_string()];

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
