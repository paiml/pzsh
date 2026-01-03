//! Executor module for pzsh
//!
//! O(1) execution with 2ms budget constraint.
//! No subprocess spawning at startup.

use crate::config::CompiledConfig;
use crate::{MAX_EXECUTOR_MS, PzshError, Result};
use ahash::AHashMap;
use std::time::{Duration, Instant};

/// Frozen environment (immutable after startup)
#[derive(Debug, Clone)]
pub struct FrozenEnv {
    vars: AHashMap<String, String>,
}

impl FrozenEnv {
    /// Create frozen environment from config
    #[must_use]
    pub fn new(config: &CompiledConfig) -> Self {
        Self {
            vars: config.env.clone(),
        }
    }

    /// O(1) lookup
    #[must_use]
    #[inline]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }

    /// Get all variables
    #[must_use]
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.vars.iter()
    }

    /// Number of environment variables
    #[must_use]
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }
}

/// Executor with O(1) operations
#[derive(Debug)]
pub struct Executor {
    /// Frozen environment
    env: FrozenEnv,
    /// Alias table (O(1) lookup)
    aliases: AHashMap<String, String>,
    /// Initialized flag
    initialized: bool,
}

impl Executor {
    /// Create a new executor from compiled config
    #[must_use]
    pub fn new(config: &CompiledConfig) -> Self {
        Self {
            env: FrozenEnv::new(config),
            aliases: config.aliases.clone(),
            initialized: false,
        }
    }

    /// Initialize the executor
    ///
    /// # Errors
    /// Returns error if initialization exceeds 2ms budget
    pub fn initialize(&mut self) -> Result<()> {
        let start = Instant::now();

        // No subprocess spawning!
        // All paths pre-resolved at compile time
        // All environment variables pre-expanded

        self.initialized = true;

        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(MAX_EXECUTOR_MS) {
            return Err(PzshError::ExecutorBudgetExceeded(
                MAX_EXECUTOR_MS,
                elapsed.as_millis() as u64,
            ));
        }

        Ok(())
    }

    /// Check if initialized
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get environment variable (O(1))
    #[must_use]
    #[inline]
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env.get(key)
    }

    /// Get alias expansion (O(1))
    #[must_use]
    #[inline]
    pub fn get_alias(&self, name: &str) -> Option<&String> {
        self.aliases.get(name)
    }

    /// Expand alias if exists, otherwise return original
    #[must_use]
    pub fn expand_alias<'a>(&'a self, command: &'a str) -> &'a str {
        self.aliases.get(command).map_or(command, String::as_str)
    }

    /// Generate shell export statements
    #[must_use]
    pub fn generate_exports(&self) -> String {
        let mut output = String::with_capacity(self.env.len() * 50);

        for (key, value) in self.env.iter() {
            output.push_str("export ");
            output.push_str(key);
            output.push_str("=\"");
            output.push_str(value);
            output.push_str("\"\n");
        }

        output
    }

    /// Generate shell alias statements
    #[must_use]
    pub fn generate_aliases(&self) -> String {
        let mut output = String::with_capacity(self.aliases.len() * 30);

        for (name, expansion) in &self.aliases {
            output.push_str("alias ");
            output.push_str(name);
            output.push_str("=\"");
            output.push_str(expansion);
            output.push_str("\"\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn test_config() -> CompiledConfig {
        let mut config = CompiledConfig::default();
        config.env.insert("EDITOR".to_string(), "vim".to_string());
        config.env.insert(
            "GOROOT".to_string(),
            "/usr/local/opt/go/libexec".to_string(),
        );
        config
            .aliases
            .insert("ll".to_string(), "ls -la".to_string());
        config
            .aliases
            .insert("gs".to_string(), "git status".to_string());
        config
    }

    #[test]
    fn test_executor_init_under_2ms() {
        let config = test_config();
        let mut executor = Executor::new(&config);

        let start = Instant::now();
        let result = executor.initialize();
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(
            elapsed < Duration::from_millis(MAX_EXECUTOR_MS),
            "ANDON: Executor exceeded 2ms budget: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_env_lookup_is_o1() {
        let mut config = CompiledConfig::default();

        // Add 10000 env vars
        for i in 0..10000 {
            config.env.insert(format!("VAR{i}"), format!("value{i}"));
        }

        let executor = Executor::new(&config);

        // Measure lookup time for first var
        let start = Instant::now();
        let _ = executor.get_env("VAR0");
        let time_first = start.elapsed();

        // Measure lookup time for last var
        let start = Instant::now();
        let _ = executor.get_env("VAR9999");
        let time_last = start.elapsed();

        // Both should be under 1 microsecond (O(1))
        assert!(
            time_first.as_micros() < 10,
            "First lookup too slow: {:?}",
            time_first
        );
        assert!(
            time_last.as_micros() < 10,
            "Last lookup too slow: {:?}",
            time_last
        );
    }

    #[test]
    fn test_alias_lookup_is_o1() {
        let mut config = CompiledConfig::default();

        // Add 10000 aliases
        for i in 0..10000 {
            config
                .aliases
                .insert(format!("alias{i}"), format!("command{i}"));
        }

        let executor = Executor::new(&config);

        // Measure lookup time for first alias
        let start = Instant::now();
        let _ = executor.get_alias("alias0");
        let time_first = start.elapsed();

        // Measure lookup time for last alias
        let start = Instant::now();
        let _ = executor.get_alias("alias9999");
        let time_last = start.elapsed();

        // Both should be under 1 microsecond (O(1))
        assert!(
            time_first.as_micros() < 10,
            "First lookup too slow: {:?}",
            time_first
        );
        assert!(
            time_last.as_micros() < 10,
            "Last lookup too slow: {:?}",
            time_last
        );
    }

    #[test]
    fn test_expand_alias() {
        let config = test_config();
        let executor = Executor::new(&config);

        assert_eq!(executor.expand_alias("ll"), "ls -la");
        assert_eq!(executor.expand_alias("nonexistent"), "nonexistent");
    }

    #[test]
    fn test_generate_exports() {
        let config = test_config();
        let executor = Executor::new(&config);

        let exports = executor.generate_exports();

        assert!(exports.contains("export EDITOR=\"vim\""));
        assert!(exports.contains("export GOROOT=\"/usr/local/opt/go/libexec\""));
    }

    #[test]
    fn test_generate_aliases() {
        let config = test_config();
        let executor = Executor::new(&config);

        let aliases = executor.generate_aliases();

        assert!(aliases.contains("alias ll=\"ls -la\""));
        assert!(aliases.contains("alias gs=\"git status\""));
    }

    #[test]
    fn test_frozen_env_immutable() {
        let config = test_config();
        let env = FrozenEnv::new(&config);

        // FrozenEnv should have no mutation methods
        // This is enforced by the type system
        assert_eq!(env.get("EDITOR"), Some(&"vim".to_string()));
        assert_eq!(env.len(), 2);
        assert!(!env.is_empty());
    }

    #[test]
    fn test_executor_deterministic() {
        let config = test_config();

        let executor1 = Executor::new(&config);
        let executor2 = Executor::new(&config);

        let exports1 = executor1.generate_exports();
        let exports2 = executor2.generate_exports();

        // Sort lines for comparison (hash map order may vary)
        let mut lines1: Vec<_> = exports1.lines().collect();
        let mut lines2: Vec<_> = exports2.lines().collect();
        lines1.sort();
        lines2.sort();

        assert_eq!(lines1, lines2, "Executor must be deterministic");
    }
}
