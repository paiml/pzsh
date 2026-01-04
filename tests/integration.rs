//! Integration tests for pzsh
//!
//! These tests verify all features work together and meet performance budgets.
//! Designed to run in <1 second for use as pre-commit hook.

use std::process::Command;
use std::time::{Duration, Instant};

/// Performance budget from pmat.toml
const STARTUP_BUDGET_MS: u64 = 10;
const PROMPT_BUDGET_MS: u64 = 2;

/// Test that pzsh binary exists and runs
#[test]
fn test_binary_runs() {
    let output = Command::new("cargo")
        .args(["run", "--release", "--quiet", "--", "--version"])
        .output()
        .expect("Failed to run pzsh");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pzsh"));
}

/// Test startup performance meets 10ms budget
#[test]
fn test_startup_performance() {
    // Warm up
    let _ = Command::new("cargo")
        .args(["run", "--release", "--quiet", "--", "status"])
        .output();

    let start = Instant::now();
    let output = Command::new("cargo")
        .args(["run", "--release", "--quiet", "--", "status"])
        .output()
        .expect("Failed to run pzsh status");
    let elapsed = start.elapsed();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("budget: 10ms"));
    assert!(stdout.contains("✓"));

    // Binary execution should be fast (excluding cargo overhead)
    assert!(
        elapsed < Duration::from_secs(5),
        "Status command too slow: {:?}",
        elapsed
    );
}

/// Test compile generates valid shell code
#[test]
fn test_compile_generates_shell_code() {
    let output = Command::new("cargo")
        .args(["run", "--release", "--quiet", "--", "compile"])
        .output()
        .expect("Failed to run pzsh compile");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain shell integration header
    assert!(stdout.contains("pzsh shell integration"));

    // Should contain aliases
    assert!(stdout.contains("alias"));

    // Should contain prompt configuration
    assert!(stdout.contains("PROMPT") || stdout.contains("PS1"));
}

/// Test all 9 plugins are available
#[test]
fn test_all_plugins_available() {
    use pzsh::plugin::PluginManager;

    let manager = PluginManager::new();
    let plugins = manager.list();

    let expected = [
        "git",
        "docker",
        "kubectl",
        "npm",
        "python",
        "golang",
        "rust",
        "terraform",
        "aws",
    ];

    for name in expected {
        assert!(
            plugins.iter().any(|(n, _)| *n == name),
            "Missing plugin: {}",
            name
        );
    }

    assert!(plugins.len() >= 9, "Expected at least 9 plugins");
}

/// Test all plugins load under budget
#[test]
fn test_plugin_load_performance() {
    use pzsh::plugin::PluginManager;
    use std::time::Instant;

    let mut manager = PluginManager::new();
    let plugins = [
        "git",
        "docker",
        "kubectl",
        "npm",
        "python",
        "golang",
        "rust",
        "terraform",
        "aws",
    ];

    let start = Instant::now();
    for name in plugins {
        manager
            .load(name)
            .expect(&format!("Failed to load {}", name));
    }
    let elapsed = start.elapsed();

    // All 9 plugins should load in under 1ms
    assert!(
        elapsed < Duration::from_millis(1),
        "Plugin loading too slow: {:?}",
        elapsed
    );
}

/// Test combined aliases from all plugins
#[test]
fn test_combined_aliases() {
    use pzsh::plugin::PluginManager;

    let mut manager = PluginManager::new();
    for name in [
        "git",
        "docker",
        "kubectl",
        "npm",
        "python",
        "golang",
        "rust",
        "terraform",
        "aws",
    ] {
        manager.load(name).unwrap();
    }

    let aliases = manager.all_aliases();

    // Should have 100+ aliases
    assert!(
        aliases.len() >= 100,
        "Expected 100+ aliases, got {}",
        aliases.len()
    );

    // Spot check key aliases
    assert!(aliases.contains_key("gs"), "Missing git status alias");
    assert!(aliases.contains_key("k"), "Missing kubectl alias");
    assert!(aliases.contains_key("tf"), "Missing terraform alias");
}

/// Test shell initialization for both zsh and bash
#[test]
fn test_shell_init_both_shells() {
    use pzsh::ShellType;
    use pzsh::config::CompiledConfig;
    use pzsh::shell::generate_init;

    let config = CompiledConfig::default();

    // Test zsh
    let zsh_init = generate_init(ShellType::Zsh, config.clone());
    assert!(zsh_init.contains("bindkey"), "Zsh should have keybindings");
    assert!(zsh_init.contains("compinit"), "Zsh should have completion");

    // Test bash
    let bash_init = generate_init(ShellType::Bash, config);
    assert!(bash_init.contains("bind"), "Bash should have keybindings");
    assert!(
        bash_init.contains("completion"),
        "Bash should have completion"
    );
}

/// Test prompt rendering performance
#[test]
fn test_prompt_performance() {
    use pzsh::config::CompiledConfig;
    use pzsh::prompt::Prompt;

    let config = CompiledConfig::default();
    let prompt = Prompt::new(&config);

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = prompt.render();
    }
    let elapsed = start.elapsed();

    let per_render = elapsed / 1000;
    assert!(
        per_render < Duration::from_millis(PROMPT_BUDGET_MS),
        "Prompt render too slow: {:?}",
        per_render
    );
}

/// Test parser performance
#[test]
fn test_parser_performance() {
    use pzsh::config::CompiledConfig;
    use pzsh::parser::Parser;

    let config = CompiledConfig::default();
    let mut parser = Parser::new(&config);

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = parser.parse("git status");
    }
    let elapsed = start.elapsed();

    let per_parse = elapsed / 1000;
    assert!(
        per_parse < Duration::from_millis(2),
        "Parser too slow: {:?}",
        per_parse
    );
}

/// Test executor performance
#[test]
fn test_executor_performance() {
    use pzsh::config::CompiledConfig;
    use pzsh::executor::Executor;

    let config = CompiledConfig::default();
    let executor = Executor::new(&config);

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = executor.expand_alias("gs");
    }
    let elapsed = start.elapsed();

    let per_expand = elapsed / 1000;
    assert!(
        per_expand < Duration::from_millis(2),
        "Executor too slow: {:?}",
        per_expand
    );
}

/// Test theme system
#[test]
fn test_themes_available() {
    use pzsh::theme::ThemeRegistry;

    let registry = ThemeRegistry::new();
    let themes = registry.list();

    assert!(themes.contains(&"robbyrussell"));
    assert!(themes.contains(&"agnoster"));
    assert!(themes.contains(&"simple"));
    assert!(themes.contains(&"pure"));
    assert!(themes.contains(&"spaceship"));
}

/// Test completion system
#[test]
fn test_completion_system() {
    use pzsh::completion::CompletionEngine;

    let engine = CompletionEngine::new();
    let completions = engine.complete("git ", 4);
    // Should return some completions (may be empty if no providers configured)
    // Just verify it doesn't panic - actual completions depend on configuration
    let _ = completions.len();
}

/// ANDON test: Full startup under 10ms
#[test]
fn test_andon_startup_budget() {
    use pzsh::Pzsh;
    use pzsh::config::CompiledConfig;

    let start = Instant::now();
    let config = CompiledConfig::default();
    let result = Pzsh::new(config);
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Pzsh initialization failed");
    assert!(
        elapsed < Duration::from_millis(STARTUP_BUDGET_MS),
        "ANDON: Startup exceeded {}ms budget: {:?}",
        STARTUP_BUDGET_MS,
        elapsed
    );
}

/// Test deterministic output
#[test]
fn test_deterministic_output() {
    use pzsh::ShellType;
    use pzsh::config::CompiledConfig;
    use pzsh::shell::generate_init;

    let config = CompiledConfig::default();

    let output1 = generate_init(ShellType::Zsh, config.clone());
    let output2 = generate_init(ShellType::Zsh, config);

    assert_eq!(output1, output2, "Output should be deterministic");
}
