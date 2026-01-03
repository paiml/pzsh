//! Configuration module for pzsh
//!
//! Provides O(1) compiled configuration with no runtime parsing overhead.

use crate::ShellType;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid configuration: {0}")]
    Invalid(String),

    #[error("forbidden pattern: {0}")]
    ForbiddenPattern(String),

    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Source configuration (human-readable .pzshrc)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    #[serde(default)]
    pub pzsh: PzshSection,
    #[serde(default)]
    pub performance: PerformanceSection,
    #[serde(default)]
    pub prompt: PromptSection,
    #[serde(default)]
    pub aliases: AHashMap<String, String>,
    #[serde(default)]
    pub env: AHashMap<String, String>,
    #[serde(default)]
    pub plugins: PluginsSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PzshSection {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub shell: ShellTypeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ShellTypeConfig {
    #[default]
    Zsh,
    Bash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSection {
    #[serde(default = "default_startup_budget")]
    pub startup_budget_ms: u64,
    #[serde(default = "default_prompt_budget")]
    pub prompt_budget_ms: u64,
    #[serde(default = "default_lazy_load")]
    pub lazy_load: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptSection {
    #[serde(default = "default_prompt_format")]
    pub format: String,
    #[serde(default = "default_true")]
    pub git_async: bool,
    #[serde(default = "default_git_cache_ms")]
    pub git_cache_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginsSection {
    #[serde(default)]
    pub enabled: Vec<String>,
    #[serde(default)]
    pub lazy: Vec<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_startup_budget() -> u64 {
    10
}

fn default_prompt_budget() -> u64 {
    2
}

fn default_lazy_load() -> bool {
    true
}

fn default_prompt_format() -> String {
    "{user}@{host} {cwd} {git} {char}".to_string()
}

fn default_true() -> bool {
    true
}

fn default_git_cache_ms() -> u64 {
    1000
}

impl Default for PerformanceSection {
    fn default() -> Self {
        Self {
            startup_budget_ms: default_startup_budget(),
            prompt_budget_ms: default_prompt_budget(),
            lazy_load: default_lazy_load(),
        }
    }
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            pzsh: PzshSection::default(),
            performance: PerformanceSection::default(),
            prompt: PromptSection::default(),
            aliases: AHashMap::new(),
            env: AHashMap::new(),
            plugins: PluginsSection::default(),
        }
    }
}

/// Compiled configuration (O(1) lookup, no parsing at runtime)
#[derive(Debug, Clone)]
pub struct CompiledConfig {
    pub shell_type: ShellType,
    pub startup_budget_ms: u64,
    pub prompt_budget_ms: u64,
    pub lazy_load: bool,
    pub prompt_format: String,
    pub git_async: bool,
    pub git_cache_ms: u64,
    /// O(1) alias lookup via perfect hash
    pub aliases: AHashMap<String, String>,
    /// O(1) environment lookup
    pub env: AHashMap<String, String>,
    pub plugins_enabled: Vec<String>,
    pub plugins_lazy: Vec<String>,
}

impl Default for CompiledConfig {
    fn default() -> Self {
        Self {
            shell_type: ShellType::Zsh,
            startup_budget_ms: 10,
            prompt_budget_ms: 2,
            lazy_load: true,
            prompt_format: default_prompt_format(),
            git_async: true,
            git_cache_ms: 1000,
            aliases: AHashMap::new(),
            env: AHashMap::new(),
            plugins_enabled: Vec::new(),
            plugins_lazy: Vec::new(),
        }
    }
}

impl CompiledConfig {
    /// Compile from source configuration
    ///
    /// # Errors
    /// Returns error if forbidden patterns detected
    pub fn compile(source: SourceConfig) -> Result<Self, ConfigError> {
        // Check for forbidden patterns in env
        for (key, value) in &source.env {
            Self::check_forbidden_patterns(key, value)?;
        }

        // Check for forbidden patterns in aliases
        for (key, value) in &source.aliases {
            Self::check_forbidden_patterns(key, value)?;
        }

        let shell_type = match source.pzsh.shell {
            ShellTypeConfig::Zsh => ShellType::Zsh,
            ShellTypeConfig::Bash => ShellType::Bash,
        };

        Ok(Self {
            shell_type,
            startup_budget_ms: source.performance.startup_budget_ms,
            prompt_budget_ms: source.performance.prompt_budget_ms,
            lazy_load: source.performance.lazy_load,
            prompt_format: source.prompt.format,
            git_async: source.prompt.git_async,
            git_cache_ms: source.prompt.git_cache_ms,
            aliases: source.aliases,
            env: source.env,
            plugins_enabled: source.plugins.enabled,
            plugins_lazy: source.plugins.lazy,
        })
    }

    /// Check for forbidden patterns that would violate O(1) constraint
    fn check_forbidden_patterns(_key: &str, value: &str) -> Result<(), ConfigError> {
        // Forbidden: subprocess calls
        if value.contains("$(") || value.contains("`") {
            return Err(ConfigError::ForbiddenPattern(
                "subprocess call $() or backticks not allowed at startup".to_string(),
            ));
        }

        // Forbidden: brew --prefix (common slow pattern)
        if value.contains("brew --prefix") {
            return Err(ConfigError::ForbiddenPattern(
                "brew --prefix is slow; use hardcoded path".to_string(),
            ));
        }

        // Forbidden: eval
        if value.contains("eval ") {
            return Err(ConfigError::ForbiddenPattern(
                "eval not allowed for safety".to_string(),
            ));
        }

        Ok(())
    }

    /// O(1) alias lookup
    #[must_use]
    #[inline]
    pub fn get_alias(&self, name: &str) -> Option<&String> {
        self.aliases.get(name)
    }

    /// O(1) env lookup
    #[must_use]
    #[inline]
    pub fn get_env(&self, name: &str) -> Option<&String> {
        self.env.get(name)
    }

    /// Parse from TOML string
    ///
    /// # Errors
    /// Returns error on parse failure or forbidden patterns
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        let source: SourceConfig = toml::from_str(content)?;
        Self::compile(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_default_config_is_valid() {
        let config = CompiledConfig::default();
        assert_eq!(config.startup_budget_ms, 10);
        assert_eq!(config.prompt_budget_ms, 2);
        assert!(config.lazy_load);
        assert!(config.git_async);
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

        // Measure lookup time for first alias
        let start = Instant::now();
        let _ = config.get_alias("alias0");
        let time_first = start.elapsed();

        // Measure lookup time for last alias
        let start = Instant::now();
        let _ = config.get_alias("alias9999");
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
    fn test_forbidden_subprocess_pattern() {
        let toml = r#"
[env]
GOROOT = "$(brew --prefix golang)/libexec"
"#;
        let result = CompiledConfig::from_toml(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("subprocess"));
    }

    #[test]
    fn test_forbidden_brew_prefix() {
        let toml = r#"
[env]
PATH = "/usr/bin:$(brew --prefix)/bin"
"#;
        let result = CompiledConfig::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_forbidden_eval() {
        let toml = r#"
[aliases]
dangerous = "eval $SOME_VAR"
"#;
        let result = CompiledConfig::from_toml(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("eval"));
    }

    #[test]
    fn test_valid_config_parses() {
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

[env]
EDITOR = "vim"
GOROOT = "/usr/local/opt/go/libexec"
"#;
        let config = CompiledConfig::from_toml(toml).unwrap();
        assert_eq!(config.get_alias("ll"), Some(&"ls -la".to_string()));
        assert_eq!(config.get_env("EDITOR"), Some(&"vim".to_string()));
    }

    #[test]
    fn test_config_compile_is_fast() {
        let toml = r#"
[aliases]
ll = "ls -la"

[env]
EDITOR = "vim"
"#;
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = CompiledConfig::from_toml(toml).unwrap();
        }
        let elapsed = start.elapsed();

        // 1000 compiles should take less than 100ms
        assert!(
            elapsed.as_millis() < 100,
            "Config compile too slow: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_env_lookup() {
        let mut config = CompiledConfig::default();
        config.env.insert("TEST".to_string(), "value".to_string());
        assert_eq!(config.get_env("TEST"), Some(&"value".to_string()));
        assert_eq!(config.get_env("NONEXISTENT"), None);
    }

    #[test]
    fn test_alias_lookup_miss() {
        let config = CompiledConfig::default();
        assert_eq!(config.get_alias("nonexistent"), None);
    }

    #[test]
    fn test_shell_type_bash_config() {
        let toml = r#"
[pzsh]
shell = "bash"
"#;
        let config = CompiledConfig::from_toml(toml).unwrap();
        assert_eq!(config.shell_type, crate::ShellType::Bash);
    }

    #[test]
    fn test_forbidden_backticks_in_alias() {
        let toml = r#"
[aliases]
date = "`date`"
"#;
        let result = CompiledConfig::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::Invalid("test".to_string());
        assert!(err.to_string().contains("invalid"));

        let err = ConfigError::ForbiddenPattern("test".to_string());
        assert!(err.to_string().contains("forbidden"));
    }

    #[test]
    fn test_source_config_defaults() {
        let source = SourceConfig::default();
        assert!(source.aliases.is_empty());
        assert!(source.env.is_empty());
        assert_eq!(source.performance.startup_budget_ms, 10);
    }

    #[test]
    fn test_plugins_config() {
        let toml = r#"
[plugins]
enabled = ["git", "docker"]
lazy = ["kubectl"]
"#;
        let config = CompiledConfig::from_toml(toml).unwrap();
        assert_eq!(config.plugins_enabled.len(), 2);
        assert_eq!(config.plugins_lazy.len(), 1);
    }

    #[test]
    fn test_prompt_config() {
        let toml = r#"
[prompt]
format = "{user}@{host}"
git_async = false
git_cache_ms = 500
"#;
        let config = CompiledConfig::from_toml(toml).unwrap();
        assert_eq!(config.prompt_format, "{user}@{host}");
        assert!(!config.git_async);
        assert_eq!(config.git_cache_ms, 500);
    }
}
