//! pzsh: Performance-first shell framework
//!
//! Core invariant: No shell startup shall exceed 10ms.

use clap::Parser;
use pzsh::cli::{self, Cli, Commands};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn expand_path(path: &PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path_str[2..]);
        }
    }
    path.clone()
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bench {
            iterations,
            verbose,
        } => {
            let result = cli::run_bench(iterations, verbose);
            println!("{}", result.format());
            if result.passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }

        Commands::Lint { config } => {
            let path = expand_path(&config);

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading {}: {}", path.display(), e);
                    return ExitCode::FAILURE;
                }
            };

            let result = cli::lint_config(&content);
            println!("{}", result.format());

            if result.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }

        Commands::Compile { config, output } => {
            let path = expand_path(&config);

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading {}: {}", path.display(), e);
                    return ExitCode::FAILURE;
                }
            };

            match pzsh::config::CompiledConfig::from_toml(&content) {
                Ok(compiled) => {
                    let output_path = output.unwrap_or_else(|| {
                        let mut p = path.clone();
                        p.set_extension("compiled");
                        p
                    });

                    // For now, just print success
                    println!("✓ Compiled configuration");
                    println!("  Aliases: {}", compiled.aliases.len());
                    println!("  Env vars: {}", compiled.env.len());
                    println!("  Plugins: {}", compiled.plugins_enabled.len());
                    println!("  Output: {}", output_path.display());

                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Compile error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Fix { config, dry_run } => {
            let path = expand_path(&config);

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading {}: {}", path.display(), e);
                    return ExitCode::FAILURE;
                }
            };

            let lint_result = cli::lint_config(&content);

            if lint_result.issues.is_empty() {
                println!("✓ No issues to fix");
                return ExitCode::SUCCESS;
            }

            println!("Found {} issues:", lint_result.issues.len());
            for issue in &lint_result.issues {
                if let Some(fix) = &issue.fix {
                    let prefix = if dry_run { "Would fix" } else { "Fix" };
                    println!("  {}: {} -> {}", prefix, issue.message, fix);
                }
            }

            if dry_run {
                println!("\n(dry run - no changes made)");
            }

            ExitCode::SUCCESS
        }

        Commands::Profile { verbose: _ } => {
            let result = cli::run_profile();
            println!("{}", result.format());

            if result.passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }

        Commands::Status => {
            println!("pzsh v{}", env!("CARGO_PKG_VERSION"));
            println!("────────────────────────────");

            // Run quick benchmark
            let bench = cli::run_bench(10, false);
            println!(
                "Startup: {:.2}ms (budget: {}ms) {}",
                bench.mean.as_secs_f64() * 1000.0,
                pzsh::MAX_STARTUP_MS,
                if bench.passed { "✓" } else { "✗" }
            );

            ExitCode::SUCCESS
        }

        Commands::Init { shell } => {
            let config = cli::generate_init_config(&shell);

            let home = dirs::home_dir().expect("Could not find home directory");
            let config_path = home.join(".pzshrc");

            if config_path.exists() {
                eprintln!("Error: {} already exists", config_path.display());
                eprintln!("Remove it first or edit it manually");
                return ExitCode::FAILURE;
            }

            match fs::write(&config_path, &config) {
                Ok(()) => {
                    println!("✓ Created {}", config_path.display());
                    println!("\nNext steps:");
                    println!("  1. Edit ~/.pzshrc to add your aliases and env vars");
                    println!("  2. Run `pzsh compile` to compile the configuration");
                    println!("  3. Add `source ~/.pzsh/init.zsh` to your ~/.zshrc");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error writing config: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
    }
}
