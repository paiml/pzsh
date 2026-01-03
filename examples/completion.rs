//! Completion system example
//!
//! Run with: cargo run --example completion

use ahash::AHashMap;
use pzsh::completion::{
    AliasCompleter, CompletionContext, CompletionEngine, CompletionItem, CompletionKind,
    CompletionProvider,
};
use std::path::PathBuf;
use std::sync::Arc;

fn main() {
    println!("=== pzsh Completion System ===\n");

    // Create alias map
    let mut aliases = AHashMap::new();
    aliases.insert("gs".to_string(), "git status".to_string());
    aliases.insert("gp".to_string(), "git push".to_string());
    aliases.insert("ga".to_string(), "git add".to_string());
    aliases.insert("gc".to_string(), "git commit".to_string());
    aliases.insert("gd".to_string(), "git diff".to_string());
    aliases.insert("ll".to_string(), "ls -la".to_string());
    let aliases = Arc::new(aliases);

    // Create completion engine
    let engine = CompletionEngine::new();

    // Test completion
    println!("=== Completion Engine ===");
    let completions = engine.complete("g", 1);
    println!("Input: 'g' at cursor 1");
    println!("Completions ({}):", completions.len());
    for item in completions.iter().take(10) {
        let desc = item.description.as_deref().unwrap_or("-");
        println!("  {} - {} ({:?})", item.text, desc, item.kind);
    }
    println!();

    // Alias completer with completion context
    println!("=== Alias Completer ===");
    let alias_completer = AliasCompleter::new(aliases.clone());

    let ctx = CompletionContext {
        line: "gc".to_string(),
        cursor: 2,
        word: "gc".to_string(),
        word_start: 0,
        previous_words: vec![],
        cwd: PathBuf::from("/home/user"),
    };

    println!("Complete 'gc':");
    for item in alias_completer.complete(&ctx) {
        let desc = item.description.as_deref().unwrap_or("-");
        println!("  {} -> {}", item.text, desc);
    }
    println!();

    // Completion with 'g' prefix
    let ctx = CompletionContext {
        line: "g".to_string(),
        cursor: 1,
        word: "g".to_string(),
        word_start: 0,
        previous_words: vec![],
        cwd: PathBuf::from("/home/user"),
    };

    println!("Complete 'g':");
    for item in alias_completer.complete(&ctx).iter().take(5) {
        let desc = item.description.as_deref().unwrap_or("-");
        println!("  {} -> {}", item.text, desc);
    }
    println!();

    // Completion item creation
    println!("=== Completion Item Builder ===");
    let item = CompletionItem::new("git", CompletionKind::Command)
        .with_display("git (version control)")
        .with_score(1.0)
        .with_description("Distributed version control system");
    println!("Created item: {:?}", item);
    println!();

    // Completion kinds
    println!("=== Completion Kinds ===");
    let kinds = [
        CompletionKind::Command,
        CompletionKind::File,
        CompletionKind::Directory,
        CompletionKind::Alias,
        CompletionKind::Variable,
        CompletionKind::Flag,
        CompletionKind::History,
        CompletionKind::Predicted,
        CompletionKind::Other,
    ];
    for kind in kinds {
        println!("  {:?}", kind);
    }
    println!();

    // Context structure
    println!("=== Completion Context ===");
    let ctx = CompletionContext {
        line: "git commit -m 'fix'".to_string(),
        cursor: 10,
        word: "commit".to_string(),
        word_start: 4,
        previous_words: vec!["git".to_string()],
        cwd: PathBuf::from("/home/user/project"),
    };
    println!("Line: {:?}", ctx.line);
    println!("Cursor: {}", ctx.cursor);
    println!("Word: {:?} (starts at {})", ctx.word, ctx.word_start);
    println!("Previous: {:?}", ctx.previous_words);
    println!("CWD: {:?}", ctx.cwd);
}
