//! Zsh-specific features example
//!
//! Run with: cargo run --example zsh_features

use pzsh::zsh::{
    ZshCompletion, CompletionSpec, AutoSuggestWidget, SyntaxHighlighter,
    HistorySearch, DirectoryJump,
};

fn main() {
    println!("=== pzsh Zsh Features ===\n");
    println!("Fish-style features for zsh with O(1) performance\n");

    // Completion generation
    println!("=== Zsh Completion (compdef) ===");
    let mut completion = ZshCompletion::new();

    // Register custom completions
    completion.register("myapp", vec![
        CompletionSpec::flag("-v", "Verbose output"),
        CompletionSpec::flag("--help", "Show help"),
        CompletionSpec::flag("--config", "Config file path"),
        CompletionSpec::value("command", vec![
            "build".to_string(),
            "test".to_string(),
            "run".to_string(),
        ]),
    ]);

    println!("Generated completion for 'myapp':");
    if let Some(script) = completion.generate_completion_function("myapp") {
        for line in script.lines().take(15) {
            println!("  {}", line);
        }
    }
    println!();

    // Generate all completions (includes built-in git/docker)
    println!("=== Built-in Completions ===");
    let all = completion.generate_all();
    println!("Total output: {} bytes", all.len());
    println!("First 10 lines:");
    for line in all.lines().take(10) {
        println!("  {}", line);
    }
    println!();

    // Auto-suggestions (fish-style)
    println!("=== Auto-Suggest Widget ===");
    let mut widget = AutoSuggestWidget::new();

    // Load some history
    widget.load_history(vec![
        "git status".to_string(),
        "git commit -m 'fix'".to_string(),
        "git push origin main".to_string(),
        "cargo build".to_string(),
        "cargo test".to_string(),
    ]);

    // Get suggestions
    println!("Suggestions:");
    if let Some(suggestion) = widget.suggest("git") {
        println!("  'git' -> {}", suggestion);
    }
    if let Some(suggestion) = widget.suggest("cargo") {
        println!("  'cargo' -> {}", suggestion);
    }
    println!();

    // Generate widget code
    println!("Widget code (first 10 lines):");
    let widget_code = AutoSuggestWidget::generate_widget_code();
    for line in widget_code.lines().take(10) {
        println!("  {}", line);
    }
    println!("  ... ({} total lines)\n", widget_code.lines().count());

    // Syntax highlighting
    println!("=== Syntax Highlighter ===");
    let highlighter = SyntaxHighlighter::new();

    println!("Style configuration:");
    println!("  Command: {}", highlighter.command_color);
    println!("  Alias: {}", highlighter.alias_color);
    println!("  Builtin: {}", highlighter.builtin_color);
    println!("  Error: {}", highlighter.error_color);
    println!("  Path: {}", highlighter.path_color);
    println!();

    // Generate highlighting code
    println!("Highlight code (first 15 lines):");
    let highlight_code = highlighter.generate_highlight_code();
    for line in highlight_code.lines().take(15) {
        println!("  {}", line);
    }
    println!();

    // History search
    println!("=== History Search ===");
    println!("Widget code (first 10 lines):");
    let search_code = HistorySearch::generate_widget_code();
    for line in search_code.lines().take(10) {
        println!("  {}", line);
    }
    println!("  ... ({} total lines)\n", search_code.lines().count());

    // Directory jump (z-style)
    println!("=== Directory Jump (z-style) ===");
    let mut jump = DirectoryJump::new();

    // Record directory visits (frecency-based)
    jump.record("/home/user/src/pzsh");
    jump.record("/home/user/src/project");
    jump.record("/home/user/src/pzsh");  // visited again (higher score)
    jump.record("/home/user/documents");

    println!("Find 'pzsh':");
    if let Some(path) = jump.find("pzsh") {
        println!("  -> {}", path);
    }

    println!("Find 'proj':");
    if let Some(path) = jump.find("proj") {
        println!("  -> {}", path);
    }

    println!("Find 'doc':");
    if let Some(path) = jump.find("doc") {
        println!("  -> {}", path);
    }
    println!();

    // Generate z command
    println!("Z command code (first 10 lines):");
    let z_code = DirectoryJump::generate_z_command();
    for line in z_code.lines().take(10) {
        println!("  {}", line);
    }
    println!();

    println!("=== Performance Notes ===");
    println!("All features are designed for sub-millisecond operation:");
    println!("  - Completions: O(1) lookup via AHashMap");
    println!("  - Suggestions: O(n) history scan, cached");
    println!("  - Highlighting: O(n) where n = line length");
    println!("  - History search: Native zsh widget");
    println!("  - Directory jump: O(1) frecency lookup");
}
