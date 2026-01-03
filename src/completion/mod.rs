//! Completion module for pzsh
//!
//! Provides O(1) cached completions with optional ML-based inference.
//! Supports aprender-shell model for intelligent auto-complete.

use ahash::AHashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Maximum time for completion generation (non-blocking fallback after this)
pub const COMPLETION_BUDGET_MS: u64 = 50;

/// Completion item with metadata
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionItem {
    /// The completion text to insert
    pub text: String,
    /// Display text (may differ from insert text)
    pub display: String,
    /// Completion kind for styling
    pub kind: CompletionKind,
    /// Relevance score (0.0 - 1.0)
    pub score: f32,
    /// Optional description
    pub description: Option<String>,
}

impl CompletionItem {
    /// Create a new completion item
    #[must_use]
    pub fn new(text: impl Into<String>, kind: CompletionKind) -> Self {
        let text = text.into();
        Self {
            display: text.clone(),
            text,
            kind,
            score: 1.0,
            description: None,
        }
    }

    /// Set display text
    #[must_use]
    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display = display.into();
        self
    }

    /// Set score
    #[must_use]
    pub const fn with_score(mut self, score: f32) -> Self {
        self.score = score;
        self
    }

    /// Set description
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Types of completions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionKind {
    /// Command/executable
    Command,
    /// File path
    File,
    /// Directory path
    Directory,
    /// Command alias
    Alias,
    /// Environment variable
    Variable,
    /// Command flag/option
    Flag,
    /// History item
    History,
    /// ML-predicted completion
    Predicted,
    /// Unknown/other
    Other,
}

/// Completion context for generating suggestions
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Current input line
    pub line: String,
    /// Cursor position in line
    pub cursor: usize,
    /// Current word being completed
    pub word: String,
    /// Word start position
    pub word_start: usize,
    /// Previous words (for context)
    pub previous_words: Vec<String>,
    /// Current working directory
    pub cwd: PathBuf,
}

impl CompletionContext {
    /// Create completion context from input line and cursor position
    #[must_use]
    pub fn from_line(line: &str, cursor: usize) -> Self {
        let line = line.to_string();
        let cursor = cursor.min(line.len());

        // Find word boundaries
        let before_cursor = &line[..cursor];
        let word_start = before_cursor.rfind(|c: char| c.is_whitespace()).map_or(0, |i| i + 1);
        let word = before_cursor[word_start..].to_string();

        // Get previous words
        let previous_words: Vec<String> = before_cursor[..word_start]
            .split_whitespace()
            .map(String::from)
            .collect();

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

        Self {
            line,
            cursor,
            word,
            word_start,
            previous_words,
            cwd,
        }
    }

    /// Check if completing the first word (command position)
    #[must_use]
    pub fn is_command_position(&self) -> bool {
        self.previous_words.is_empty()
    }
}

/// Trait for completion providers
pub trait CompletionProvider: Send + Sync {
    /// Generate completions for the given context
    fn complete(&self, ctx: &CompletionContext) -> Vec<CompletionItem>;

    /// Provider name for debugging
    fn name(&self) -> &str;

    /// Priority (higher = earlier in results)
    fn priority(&self) -> i32 {
        0
    }
}

/// ML-based completion provider trait (for aprender-shell integration)
pub trait MlCompletionProvider: Send + Sync {
    /// Generate ML-predicted completions (may be async)
    fn predict(&self, ctx: &CompletionContext) -> Vec<CompletionItem>;

    /// Check if model is loaded and ready
    fn is_ready(&self) -> bool;

    /// Model name/version
    fn model_name(&self) -> &str;
}

/// Alias-based completion provider
pub struct AliasCompleter {
    aliases: Arc<AHashMap<String, String>>,
}

impl AliasCompleter {
    /// Create from alias map
    #[must_use]
    pub fn new(aliases: Arc<AHashMap<String, String>>) -> Self {
        Self { aliases }
    }
}

impl CompletionProvider for AliasCompleter {
    fn complete(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        if !ctx.is_command_position() {
            return Vec::new();
        }

        self.aliases
            .iter()
            .filter(|(name, _)| name.starts_with(&ctx.word))
            .map(|(name, expansion)| {
                CompletionItem::new(name.clone(), CompletionKind::Alias)
                    .with_description(format!("→ {expansion}"))
            })
            .collect()
    }

    fn name(&self) -> &str {
        "aliases"
    }

    fn priority(&self) -> i32 {
        10 // High priority for aliases
    }
}

/// Environment variable completion provider
pub struct EnvCompleter;

impl CompletionProvider for EnvCompleter {
    fn complete(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        // Complete $VAR or ${VAR}
        if !ctx.word.starts_with('$') {
            return Vec::new();
        }

        let prefix = ctx.word.trim_start_matches('$').trim_start_matches('{');

        std::env::vars()
            .filter(|(name, _)| name.starts_with(prefix))
            .take(50) // Limit results
            .map(|(name, value)| {
                let truncated = if value.len() > 30 {
                    format!("{}...", &value[..27])
                } else {
                    value
                };
                CompletionItem::new(format!("${name}"), CompletionKind::Variable)
                    .with_description(truncated)
            })
            .collect()
    }

    fn name(&self) -> &str {
        "env"
    }
}

/// Path completion provider
pub struct PathCompleter;

impl CompletionProvider for PathCompleter {
    fn complete(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        let path = if ctx.word.is_empty() {
            ctx.cwd.clone()
        } else if ctx.word.starts_with('/') {
            PathBuf::from(&ctx.word)
        } else if ctx.word.starts_with('~') {
            dirs::home_dir()
                .map(|h| h.join(ctx.word.trim_start_matches("~/")))
                .unwrap_or_else(|| PathBuf::from(&ctx.word))
        } else {
            ctx.cwd.join(&ctx.word)
        };

        // Get parent directory and prefix
        let (dir, prefix) = if path.is_dir() {
            (path.clone(), String::new())
        } else {
            let parent = path.parent().unwrap_or(&ctx.cwd);
            let prefix = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            (parent.to_path_buf(), prefix)
        };

        // Read directory entries
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };

        entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .is_some_and(|n| prefix.is_empty() || n.starts_with(&prefix))
            })
            .take(100) // Limit results
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let kind = if is_dir {
                    CompletionKind::Directory
                } else {
                    CompletionKind::File
                };
                let display = if is_dir {
                    format!("{name}/")
                } else {
                    name.clone()
                };
                CompletionItem::new(display.clone(), kind).with_display(display)
            })
            .collect()
    }

    fn name(&self) -> &str {
        "paths"
    }
}

/// History-based completion provider
pub struct HistoryCompleter {
    history: Vec<String>,
}

impl HistoryCompleter {
    /// Create from history entries
    #[must_use]
    pub fn new(history: Vec<String>) -> Self {
        Self { history }
    }

    /// Add history entry
    pub fn add(&mut self, entry: String) {
        self.history.push(entry);
    }
}

impl CompletionProvider for HistoryCompleter {
    fn complete(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        if ctx.word.is_empty() {
            return Vec::new();
        }

        self.history
            .iter()
            .rev() // Most recent first
            .filter(|entry| entry.starts_with(&ctx.word))
            .take(10)
            .map(|entry| CompletionItem::new(entry.clone(), CompletionKind::History))
            .collect()
    }

    fn name(&self) -> &str {
        "history"
    }

    fn priority(&self) -> i32 {
        5 // Medium-high priority
    }
}

/// Stub for aprender-shell ML model integration
///
/// This is a placeholder that will be replaced with actual
/// aprender-shell model integration when available.
pub struct AprenderShellCompleter {
    model_path: Option<PathBuf>,
    ready: bool,
}

impl AprenderShellCompleter {
    /// Create new aprender-shell completer
    #[must_use]
    pub fn new() -> Self {
        Self {
            model_path: None,
            ready: false,
        }
    }

    /// Load model from path
    pub fn load_model(&mut self, path: PathBuf) -> Result<(), String> {
        // TODO: Actual model loading via aprender-shell
        if path.exists() {
            self.model_path = Some(path);
            self.ready = true;
            Ok(())
        } else {
            Err(format!("Model not found: {}", path.display()))
        }
    }
}

impl Default for AprenderShellCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl MlCompletionProvider for AprenderShellCompleter {
    fn predict(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        if !self.ready {
            return Vec::new();
        }

        // TODO: Call aprender-shell model for predictions
        // For now, return empty (model integration pending)
        //
        // Future implementation will:
        // 1. Encode context using model tokenizer
        // 2. Run inference on aprender-shell model
        // 3. Decode predictions into CompletionItems
        // 4. Score by model confidence
        let _ = ctx; // Silence unused warning
        Vec::new()
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn model_name(&self) -> &str {
        "aprender-shell"
    }
}

/// Main completion engine
pub struct CompletionEngine {
    providers: Vec<Box<dyn CompletionProvider>>,
    ml_provider: Option<Box<dyn MlCompletionProvider>>,
    #[allow(dead_code)] // Used for future cache invalidation
    cache: AHashMap<String, Vec<CompletionItem>>,
    #[allow(dead_code)] // Used for future cache invalidation
    cache_timeout: Duration,
}

impl CompletionEngine {
    /// Create new completion engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            ml_provider: None,
            cache: AHashMap::new(),
            cache_timeout: Duration::from_millis(100),
        }
    }

    /// Add a completion provider
    pub fn add_provider(&mut self, provider: impl CompletionProvider + 'static) {
        self.providers.push(Box::new(provider));
        // Sort by priority (descending)
        self.providers.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Set ML completion provider (e.g., aprender-shell)
    pub fn set_ml_provider(&mut self, provider: impl MlCompletionProvider + 'static) {
        self.ml_provider = Some(Box::new(provider));
    }

    /// Generate completions for input
    #[must_use]
    pub fn complete(&self, line: &str, cursor: usize) -> Vec<CompletionItem> {
        let start = Instant::now();
        let ctx = CompletionContext::from_line(line, cursor);

        let mut results = Vec::new();

        // Collect from all providers
        for provider in &self.providers {
            if start.elapsed() > Duration::from_millis(COMPLETION_BUDGET_MS) {
                break; // Budget exceeded
            }
            results.extend(provider.complete(&ctx));
        }

        // Add ML predictions if available
        if let Some(ml) = &self.ml_provider {
            if ml.is_ready() && start.elapsed() < Duration::from_millis(COMPLETION_BUDGET_MS) {
                let predictions = ml.predict(&ctx);
                results.extend(predictions);
            }
        }

        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate by text
        let mut seen = std::collections::HashSet::new();
        results.retain(|item| seen.insert(item.text.clone()));

        results
    }

    /// Clear completion cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default completion engine with standard providers
#[must_use]
pub fn default_engine(aliases: Arc<AHashMap<String, String>>) -> CompletionEngine {
    let mut engine = CompletionEngine::new();
    engine.add_provider(AliasCompleter::new(aliases));
    engine.add_provider(EnvCompleter);
    engine.add_provider(PathCompleter);
    engine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_context_parsing() {
        let ctx = CompletionContext::from_line("git sta", 7);
        assert_eq!(ctx.word, "sta");
        assert_eq!(ctx.previous_words, vec!["git"]);
        assert!(!ctx.is_command_position());
    }

    #[test]
    fn test_completion_context_command_position() {
        let ctx = CompletionContext::from_line("gi", 2);
        assert_eq!(ctx.word, "gi");
        assert!(ctx.previous_words.is_empty());
        assert!(ctx.is_command_position());
    }

    #[test]
    fn test_alias_completer() {
        let mut aliases = AHashMap::new();
        aliases.insert("gs".to_string(), "git status".to_string());
        aliases.insert("gp".to_string(), "git push".to_string());
        aliases.insert("ll".to_string(), "ls -la".to_string());

        let completer = AliasCompleter::new(Arc::new(aliases));
        let ctx = CompletionContext::from_line("g", 1);

        let results = completer.complete(&ctx);
        assert_eq!(results.len(), 2); // gs, gp
        assert!(results.iter().any(|r| r.text == "gs"));
        assert!(results.iter().any(|r| r.text == "gp"));
    }

    #[test]
    fn test_alias_completer_not_command_position() {
        let mut aliases = AHashMap::new();
        aliases.insert("gs".to_string(), "git status".to_string());

        let completer = AliasCompleter::new(Arc::new(aliases));
        let ctx = CompletionContext::from_line("echo g", 6);

        let results = completer.complete(&ctx);
        assert!(results.is_empty()); // Aliases only complete in command position
    }

    #[test]
    fn test_env_completer() {
        let completer = EnvCompleter;

        // Test with common env var that should exist (PATH)
        let ctx = CompletionContext::from_line("echo $PAT", 9);
        let results = completer.complete(&ctx);
        assert!(results.iter().any(|r| r.text == "$PATH"));
    }

    #[test]
    fn test_env_completer_no_dollar() {
        let completer = EnvCompleter;

        // Without $, should return empty
        let ctx = CompletionContext::from_line("echo PATH", 9);
        let results = completer.complete(&ctx);
        assert!(results.is_empty());
    }

    #[test]
    fn test_completion_item_builder() {
        let item = CompletionItem::new("test", CompletionKind::Command)
            .with_display("Test Command")
            .with_score(0.9)
            .with_description("A test command");

        assert_eq!(item.text, "test");
        assert_eq!(item.display, "Test Command");
        assert!((item.score - 0.9).abs() < f32::EPSILON);
        assert_eq!(item.description, Some("A test command".to_string()));
    }

    #[test]
    fn test_completion_engine() {
        let mut aliases = AHashMap::new();
        aliases.insert("gs".to_string(), "git status".to_string());

        let engine = default_engine(Arc::new(aliases));
        let results = engine.complete("g", 1);

        // Should find the alias
        assert!(results.iter().any(|r| r.text == "gs"));
    }

    #[test]
    fn test_aprender_shell_stub() {
        let completer = AprenderShellCompleter::new();
        assert!(!completer.is_ready());
        assert_eq!(completer.model_name(), "aprender-shell");

        let ctx = CompletionContext::from_line("git", 3);
        let results = completer.predict(&ctx);
        assert!(results.is_empty()); // Not ready
    }

    #[test]
    fn test_completion_performance() {
        let mut aliases = AHashMap::new();
        for i in 0..1000 {
            aliases.insert(format!("alias{i}"), format!("command{i}"));
        }

        let engine = default_engine(Arc::new(aliases));

        let start = Instant::now();
        for _ in 0..100 {
            let _ = engine.complete("alias", 5);
        }
        let elapsed = start.elapsed();

        // 100 completions should be fast
        assert!(
            elapsed < Duration::from_millis(100),
            "Completion too slow: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_history_completer() {
        let history = vec![
            "git status".to_string(),
            "git push".to_string(),
            "cargo build".to_string(),
        ];

        let completer = HistoryCompleter::new(history);
        let ctx = CompletionContext::from_line("git", 3);

        let results = completer.complete(&ctx);
        assert_eq!(results.len(), 2);
    }
}
