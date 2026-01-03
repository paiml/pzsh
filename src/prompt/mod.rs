//! Prompt module for pzsh
//!
//! O(1) prompt rendering with 2ms budget constraint.
//! Git status is async-updated, never blocks.

use crate::config::CompiledConfig;
use crate::{MAX_PROMPT_MS, PzshError, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Compiled prompt segment (pre-rendered where possible)
#[derive(Debug, Clone)]
pub enum PromptSegment {
    /// Literal text (pre-computed)
    Literal(String),
    /// User name
    User,
    /// Hostname
    Host,
    /// Current working directory
    Cwd,
    /// Git branch (cached, async-updated)
    Git,
    /// Prompt character ($ or #)
    Char,
    /// Custom segment
    Custom(String),
}

/// Cached git status (updated asynchronously)
#[derive(Debug, Clone, Default)]
pub struct GitCache {
    /// Current branch name
    pub branch: Option<String>,
    /// Is dirty
    pub dirty: bool,
    /// Cache valid flag
    valid: Arc<AtomicBool>,
}

impl GitCache {
    /// Create empty cache
    #[must_use]
    pub fn new() -> Self {
        Self {
            branch: None,
            dirty: false,
            valid: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if cache is valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.valid.load(Ordering::Relaxed)
    }

    /// Invalidate cache
    pub fn invalidate(&self) {
        self.valid.store(false, Ordering::Relaxed);
    }

    /// Render git status string
    #[must_use]
    pub fn render(&self) -> String {
        match &self.branch {
            Some(branch) => {
                let dirty_marker = if self.dirty { "*" } else { "" };
                format!("({branch}{dirty_marker})")
            }
            None => String::new(),
        }
    }
}

/// Prompt renderer with O(1) segment rendering
#[derive(Debug)]
pub struct Prompt {
    /// Pre-compiled segments
    segments: Vec<PromptSegment>,
    /// Git cache (async-updated)
    git_cache: GitCache,
    /// Cached values
    user: String,
    host: String,
}

impl Prompt {
    /// Create a new prompt from compiled config
    #[must_use]
    pub fn new(config: &CompiledConfig) -> Self {
        let segments = Self::parse_format(&config.prompt_format);

        // Pre-compute static values
        let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        let host = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "localhost".to_string());

        Self {
            segments,
            git_cache: GitCache::new(),
            user,
            host,
        }
    }

    /// Parse format string into segments
    fn parse_format(format: &str) -> Vec<PromptSegment> {
        let mut segments = Vec::new();
        let mut current_literal = String::new();
        let mut in_brace = false;
        let mut brace_content = String::new();

        for ch in format.chars() {
            match ch {
                '{' if !in_brace => {
                    if !current_literal.is_empty() {
                        segments.push(PromptSegment::Literal(std::mem::take(&mut current_literal)));
                    }
                    in_brace = true;
                }
                '}' if in_brace => {
                    let segment = match brace_content.as_str() {
                        "user" => PromptSegment::User,
                        "host" => PromptSegment::Host,
                        "cwd" => PromptSegment::Cwd,
                        "git" => PromptSegment::Git,
                        "char" => PromptSegment::Char,
                        other => PromptSegment::Custom(other.to_string()),
                    };
                    segments.push(segment);
                    brace_content.clear();
                    in_brace = false;
                }
                _ if in_brace => {
                    brace_content.push(ch);
                }
                _ => {
                    current_literal.push(ch);
                }
            }
        }

        if !current_literal.is_empty() {
            segments.push(PromptSegment::Literal(current_literal));
        }

        segments
    }

    /// Render prompt in O(1) time
    ///
    /// # Errors
    /// Returns error if rendering exceeds 2ms budget
    pub fn render(&self) -> Result<String> {
        let start = Instant::now();

        let mut output = String::with_capacity(128);

        for segment in &self.segments {
            match segment {
                PromptSegment::Literal(s) => output.push_str(s),
                PromptSegment::User => output.push_str(&self.user),
                PromptSegment::Host => output.push_str(&self.host),
                PromptSegment::Cwd => {
                    // Use PWD or current_dir (no subprocess!)
                    let cwd = std::env::var("PWD")
                        .or_else(|_| std::env::current_dir().map(|p| p.display().to_string()))
                        .unwrap_or_else(|_| "~".to_string());
                    output.push_str(&cwd);
                }
                PromptSegment::Git => {
                    // Use cached git status (never blocks)
                    output.push_str(&self.git_cache.render());
                }
                PromptSegment::Char => {
                    let is_root = self.user == "root";
                    output.push(if is_root { '#' } else { '$' });
                }
                PromptSegment::Custom(name) => {
                    output.push_str(&format!("{{{name}}}"));
                }
            }
        }

        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(MAX_PROMPT_MS) {
            return Err(PzshError::PromptBudgetExceeded(
                MAX_PROMPT_MS,
                elapsed.as_millis() as u64,
            ));
        }

        Ok(output)
    }

    /// Update git cache (called asynchronously)
    pub fn update_git_cache(&mut self, branch: Option<String>, dirty: bool) {
        self.git_cache.branch = branch;
        self.git_cache.dirty = dirty;
        self.git_cache.valid.store(true, Ordering::Relaxed);
    }

    /// Invalidate git cache
    pub fn invalidate_git_cache(&self) {
        self.git_cache.invalidate();
    }

    /// Get number of segments
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn test_config() -> CompiledConfig {
        let mut config = CompiledConfig::default();
        config.prompt_format = "{user}@{host} {cwd} {git} {char} ".to_string();
        config
    }

    #[test]
    fn test_prompt_render_under_2ms() {
        let config = test_config();
        let prompt = Prompt::new(&config);

        let start = Instant::now();
        let result = prompt.render();
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(
            elapsed < Duration::from_millis(MAX_PROMPT_MS),
            "ANDON: Prompt exceeded 2ms budget: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_parse_format() {
        let segments = Prompt::parse_format("{user}@{host} {cwd} {char}");

        assert_eq!(segments.len(), 7);
        assert!(matches!(segments[0], PromptSegment::User));
        assert!(matches!(segments[1], PromptSegment::Literal(ref s) if s == "@"));
        assert!(matches!(segments[2], PromptSegment::Host));
        assert!(matches!(segments[3], PromptSegment::Literal(ref s) if s == " "));
        assert!(matches!(segments[4], PromptSegment::Cwd));
        assert!(matches!(segments[5], PromptSegment::Literal(ref s) if s == " "));
        assert!(matches!(segments[6], PromptSegment::Char));
    }

    #[test]
    fn test_git_cache_render() {
        let mut cache = GitCache::new();

        // Empty cache
        assert_eq!(cache.render(), "");

        // With branch
        cache.branch = Some("main".to_string());
        assert_eq!(cache.render(), "(main)");

        // With dirty flag
        cache.dirty = true;
        assert_eq!(cache.render(), "(main*)");
    }

    #[test]
    fn test_git_cache_invalidation() {
        let cache = GitCache::new();

        assert!(!cache.is_valid());

        cache.valid.store(true, Ordering::Relaxed);
        assert!(cache.is_valid());

        cache.invalidate();
        assert!(!cache.is_valid());
    }

    #[test]
    fn test_prompt_contains_expected_parts() {
        let config = test_config();
        let prompt = Prompt::new(&config);

        let rendered = prompt.render().unwrap();

        // Should contain user
        assert!(
            rendered.contains(&prompt.user),
            "Prompt should contain user"
        );

        // Should contain host
        assert!(
            rendered.contains(&prompt.host),
            "Prompt should contain host"
        );

        // Should contain $ or #
        assert!(
            rendered.contains('$') || rendered.contains('#'),
            "Prompt should contain char"
        );
    }

    #[test]
    fn test_prompt_with_git_cache() {
        let config = test_config();
        let mut prompt = Prompt::new(&config);

        // Update git cache
        prompt.update_git_cache(Some("feature-branch".to_string()), true);

        let rendered = prompt.render().unwrap();

        assert!(
            rendered.contains("(feature-branch*)"),
            "Prompt should show git status: {}",
            rendered
        );
    }

    #[test]
    fn test_prompt_render_is_o1() {
        // Create prompts with different segment counts
        let config1 = CompiledConfig {
            prompt_format: "{user}".to_string(),
            ..Default::default()
        };
        let config2 = CompiledConfig {
            prompt_format: "{user}@{host} {cwd} {git} {char}".to_string(),
            ..Default::default()
        };

        let prompt1 = Prompt::new(&config1);
        let prompt2 = Prompt::new(&config2);

        // Measure render time for simple prompt
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = prompt1.render();
        }
        let time1 = start.elapsed();

        // Measure render time for complex prompt
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = prompt2.render();
        }
        let time2 = start.elapsed();

        // Complex should be at most 5x slower (O(k) where k is small constant)
        // Note: variance is expected in micro-benchmarks
        assert!(
            time2 < time1 * 5,
            "Complex prompt too slow: {:?} vs {:?}",
            time2,
            time1
        );
    }

    #[test]
    fn test_prompt_deterministic() {
        let config = test_config();
        let prompt = Prompt::new(&config);

        let render1 = prompt.render().unwrap();
        let render2 = prompt.render().unwrap();

        assert_eq!(render1, render2, "Prompt must be deterministic");
    }
}
