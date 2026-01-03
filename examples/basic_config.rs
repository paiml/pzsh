//! Basic configuration example
//!
//! Run with: cargo run --example basic_config

use pzsh::config::CompiledConfig;

fn main() {
    // Parse a basic configuration
    let toml = r#"
[pzsh]
version = "0.1.0"
shell = "zsh"

[performance]
startup_budget_ms = 10
lazy_load = true

[aliases]
ll = "ls -la"
gs = "git status"
gp = "git push"

[env]
EDITOR = "vim"
GOROOT = "/usr/local/opt/go/libexec"
"#;

    match CompiledConfig::from_toml(toml) {
        Ok(config) => {
            println!("✓ Configuration parsed successfully");
            println!("  Shell: {:?}", config.shell_type);
            println!("  Aliases: {}", config.aliases.len());
            println!("  Env vars: {}", config.env.len());
            println!("  Startup budget: {}ms", config.startup_budget_ms);

            // Demonstrate O(1) lookup
            if let Some(alias) = config.get_alias("ll") {
                println!("  ll -> {}", alias);
            }
        }
        Err(e) => {
            eprintln!("✗ Configuration error: {}", e);
        }
    }
}
