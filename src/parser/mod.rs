//! Parser module for pzsh
//!
//! O(1) parsing with 2ms budget constraint.
//! Uses pre-compiled patterns and LRU caching.

use crate::config::CompiledConfig;
use crate::{MAX_PARSER_MS, PzshError, Result};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

/// Parsed command representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedCommand {
    /// Simple command (no pipes, no redirects)
    Simple { command: String, args: Vec<String> },
    /// Alias expansion
    Alias { name: String, expansion: String },
    /// Built-in command
    Builtin { name: String, args: Vec<String> },
    /// Empty input
    Empty,
}

/// Parser with O(1) lookup and LRU caching
#[derive(Debug)]
pub struct Parser {
    /// LRU cache for parsed commands
    cache: LruCache<String, ParsedCommand>,
    /// Alias table reference (O(1) lookup)
    aliases: ahash::AHashMap<String, String>,
    /// Built-in commands
    builtins: ahash::AHashSet<String>,
}

impl Parser {
    /// Create a new parser from compiled config
    #[must_use]
    pub fn new(config: &CompiledConfig) -> Self {
        let mut builtins = ahash::AHashSet::new();
        for builtin in &[
            "cd", "exit", "export", "source", "alias", "unalias", "set", "unset", "echo", "printf",
            "test", "[", "true", "false", "pwd", "pushd", "popd", "dirs", "history", "fg", "bg",
            "jobs", "kill", "wait", "trap", "eval", "exec", "builtin", "command", "type", "which",
            "hash", "help", "let", "local", "readonly", "return", "shift", "times", "ulimit",
            "umask",
        ] {
            builtins.insert((*builtin).to_string());
        }

        Self {
            cache: LruCache::new(NonZeroUsize::new(1024).unwrap()),
            aliases: config.aliases.clone(),
            builtins,
        }
    }

    /// Parse input with O(1) cache lookup
    ///
    /// # Errors
    /// Returns error if parsing exceeds 2ms budget
    pub fn parse(&mut self, input: &str) -> Result<ParsedCommand> {
        let start = Instant::now();

        // O(1) cache lookup
        if let Some(cached) = self.cache.get(input) {
            return Ok(cached.clone());
        }

        let result = self.parse_uncached(input);

        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(MAX_PARSER_MS) {
            return Err(PzshError::ParserBudgetExceeded(
                MAX_PARSER_MS,
                elapsed.as_millis() as u64,
            ));
        }

        // Cache the result
        if let Ok(ref parsed) = result {
            self.cache.put(input.to_string(), parsed.clone());
        }

        result
    }

    /// Parse without cache (internal)
    fn parse_uncached(&self, input: &str) -> Result<ParsedCommand> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(ParsedCommand::Empty);
        }

        // Split into words (simple tokenization)
        let words: Vec<&str> = trimmed.split_whitespace().collect();

        if words.is_empty() {
            return Ok(ParsedCommand::Empty);
        }

        let command = words[0];
        let args: Vec<String> = words[1..].iter().map(|s| (*s).to_string()).collect();

        // Check for alias (O(1))
        if let Some(expansion) = self.aliases.get(command) {
            return Ok(ParsedCommand::Alias {
                name: command.to_string(),
                expansion: expansion.clone(),
            });
        }

        // Check for builtin (O(1))
        if self.builtins.contains(command) {
            return Ok(ParsedCommand::Builtin {
                name: command.to_string(),
                args,
            });
        }

        // Simple command
        Ok(ParsedCommand::Simple {
            command: command.to_string(),
            args,
        })
    }

    /// Clear the parse cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache hit rate
    #[must_use]
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn test_config() -> CompiledConfig {
        let mut config = CompiledConfig::default();
        config
            .aliases
            .insert("ll".to_string(), "ls -la".to_string());
        config
            .aliases
            .insert("gs".to_string(), "git status".to_string());
        config
    }

    #[test]
    fn test_parse_under_2ms() {
        let config = test_config();
        let mut parser = Parser::new(&config);

        let start = Instant::now();
        let result = parser.parse("ls -la /tmp");
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(
            elapsed < Duration::from_millis(MAX_PARSER_MS),
            "ANDON: Parser exceeded 2ms budget: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_parse_simple_command() {
        let config = test_config();
        let mut parser = Parser::new(&config);

        let result = parser.parse("ls -la /tmp").unwrap();
        assert!(matches!(result, ParsedCommand::Simple { .. }));

        if let ParsedCommand::Simple { command, args } = result {
            assert_eq!(command, "ls");
            assert_eq!(args, vec!["-la", "/tmp"]);
        }
    }

    #[test]
    fn test_parse_alias() {
        let config = test_config();
        let mut parser = Parser::new(&config);

        let result = parser.parse("ll").unwrap();
        assert!(matches!(result, ParsedCommand::Alias { .. }));

        if let ParsedCommand::Alias { name, expansion } = result {
            assert_eq!(name, "ll");
            assert_eq!(expansion, "ls -la");
        }
    }

    #[test]
    fn test_parse_builtin() {
        let config = test_config();
        let mut parser = Parser::new(&config);

        let result = parser.parse("cd /tmp").unwrap();
        assert!(matches!(result, ParsedCommand::Builtin { .. }));

        if let ParsedCommand::Builtin { name, args } = result {
            assert_eq!(name, "cd");
            assert_eq!(args, vec!["/tmp"]);
        }
    }

    #[test]
    fn test_parse_empty() {
        let config = test_config();
        let mut parser = Parser::new(&config);

        let result = parser.parse("").unwrap();
        assert!(matches!(result, ParsedCommand::Empty));

        let result = parser.parse("   ").unwrap();
        assert!(matches!(result, ParsedCommand::Empty));
    }

    #[test]
    fn test_cache_hit() {
        let config = test_config();
        let mut parser = Parser::new(&config);

        // First parse (cache miss)
        let _ = parser.parse("ls -la");
        assert_eq!(parser.cache_len(), 1);

        // Second parse (cache hit)
        let start = Instant::now();
        let _ = parser.parse("ls -la");
        let elapsed = start.elapsed();

        // Cache hit should be extremely fast (< 1 microsecond ideally)
        assert!(
            elapsed.as_micros() < 100,
            "Cache hit too slow: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_parser_is_o1_with_many_aliases() {
        let mut config = CompiledConfig::default();

        // Add 10000 aliases
        for i in 0..10000 {
            config
                .aliases
                .insert(format!("alias{i}"), format!("command{i}"));
        }

        let mut parser = Parser::new(&config);

        // Measure parse time for first alias
        let start = Instant::now();
        let _ = parser.parse("alias0");
        let time_first = start.elapsed();

        // Measure parse time for last alias
        let start = Instant::now();
        let _ = parser.parse("alias9999");
        let time_last = start.elapsed();

        // Both should be under 1ms (O(1))
        assert!(
            time_first.as_millis() < 1,
            "First parse too slow: {:?}",
            time_first
        );
        assert!(
            time_last.as_millis() < 1,
            "Last parse too slow: {:?}",
            time_last
        );
    }

    #[test]
    fn test_parse_deterministic() {
        let config = test_config();
        let mut parser = Parser::new(&config);

        let input = "ls -la /tmp";
        let result1 = parser.parse(input).unwrap();

        parser.clear_cache();

        let result2 = parser.parse(input).unwrap();

        assert_eq!(result1, result2, "Parser must be deterministic");
    }
}
