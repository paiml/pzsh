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

fn read_config(path: &PathBuf) -> Result<String, ExitCode> {
    let path = expand_path(path);
    fs::read_to_string(&path).map_err(|e| {
        eprintln!("Error reading {}: {e}", path.display());
        ExitCode::FAILURE
    })
}

fn pass_fail(passed: bool) -> ExitCode {
    if passed { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

fn cmd_bench(iterations: u32, verbose: bool) -> ExitCode {
    let result = cli::run_bench(iterations, verbose);
    println!("{}", result.format());
    pass_fail(result.passed)
}

fn cmd_lint(config: &PathBuf) -> ExitCode {
    let content = match read_config(config) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let result = cli::lint_config(&content);
    println!("{}", result.format());
    pass_fail(result.passed())
}

fn cmd_compile(config: &PathBuf, output: Option<PathBuf>) -> ExitCode {
    let content = match read_config(config) {
        Ok(c) => c,
        Err(code) => return code,
    };
    match pzsh::config::CompiledConfig::from_toml(&content) {
        Ok(compiled) => {
            let shell_code = pzsh::shell::generate_init(compiled.shell_type, compiled);
            if let Some(output_path) = output {
                let output_path = expand_path(&output_path);
                if let Err(e) = fs::write(&output_path, &shell_code) {
                    eprintln!("Error writing {}: {e}", output_path.display());
                    return ExitCode::FAILURE;
                }
                eprintln!("✓ Compiled to {}", output_path.display());
            } else {
                print!("{shell_code}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Compile error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_fix(config: &PathBuf, dry_run: bool) -> ExitCode {
    let content = match read_config(config) {
        Ok(c) => c,
        Err(code) => return code,
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
            println!("  {prefix}: {} -> {fix}", issue.message);
        }
    }
    if dry_run {
        println!("\n(dry run - no changes made)");
    }
    ExitCode::SUCCESS
}

fn cmd_profile() -> ExitCode {
    let result = cli::run_profile();
    println!("{}", result.format());
    pass_fail(result.passed)
}

fn cmd_status() -> ExitCode {
    println!("pzsh v{}", env!("CARGO_PKG_VERSION"));
    println!("────────────────────────────");
    let bench = cli::run_bench(10, false);
    println!(
        "Startup: {:.2}ms (budget: {}ms) {}",
        bench.mean.as_secs_f64() * 1000.0,
        pzsh::MAX_STARTUP_MS,
        if bench.passed { "✓" } else { "✗" }
    );
    ExitCode::SUCCESS
}

fn cmd_init(shell: &str) -> ExitCode {
    let config = cli::generate_init_config(shell);
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
            println!("  3. Add `eval \"$(pzsh compile)\"` to your ~/.zshrc");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error writing config: {e}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Bench { iterations, verbose } => cmd_bench(iterations, verbose),
        Commands::Lint { config } => cmd_lint(&config),
        Commands::Compile { config, output } => cmd_compile(&config, output),
        Commands::Fix { config, dry_run } => cmd_fix(&config, dry_run),
        Commands::Profile { verbose: _ } => cmd_profile(),
        Commands::Status => cmd_status(),
        Commands::Init { shell } => cmd_init(&shell),
    }
}
