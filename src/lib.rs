//! pzsh: Performance-first shell framework
//!
//! Core invariant: No shell startup shall exceed 10ms.
//! This is not a goal—it is a hard constraint.

pub mod cli;
pub mod config;
pub mod executor;
pub mod parser;
pub mod prompt;

use std::time::Duration;

/// Maximum allowed startup time (hard constraint)
pub const MAX_STARTUP_MS: u64 = 10;

/// Maximum allowed prompt render time
pub const MAX_PROMPT_MS: u64 = 2;

/// Maximum allowed parser time
pub const MAX_PARSER_MS: u64 = 2;

/// Maximum allowed executor time
pub const MAX_EXECUTOR_MS: u64 = 2;

/// Core error types for pzsh
#[derive(Debug, thiserror::Error)]
pub enum PzshError {
    #[error("startup exceeded {0}ms budget (took {1}ms)")]
    StartupBudgetExceeded(u64, u64),

    #[error("parser exceeded {0}ms budget (took {1}ms)")]
    ParserBudgetExceeded(u64, u64),

    #[error("executor exceeded {0}ms budget (took {1}ms)")]
    ExecutorBudgetExceeded(u64, u64),

    #[error("prompt exceeded {0}ms budget (took {1}ms)")]
    PromptBudgetExceeded(u64, u64),

    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("forbidden pattern detected: {0}")]
    ForbiddenPattern(String),
}

/// Result type for pzsh operations
pub type Result<T> = std::result::Result<T, PzshError>;

/// Shell type supported by pzsh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShellType {
    #[default]
    Zsh,
    Bash,
}

/// Main pzsh shell instance
#[derive(Debug)]
pub struct Pzsh {
    config: config::CompiledConfig,
    parser: parser::Parser,
    executor: executor::Executor,
    prompt: prompt::Prompt,
    shell_type: ShellType,
}

impl Pzsh {
    /// Create a new pzsh instance with compiled configuration
    ///
    /// # Errors
    /// Returns error if initialization exceeds 10ms budget
    pub fn new(config: config::CompiledConfig) -> Result<Self> {
        let start = std::time::Instant::now();

        let shell_type = config.shell_type;
        let parser = parser::Parser::new(&config);
        let executor = executor::Executor::new(&config);
        let prompt = prompt::Prompt::new(&config);

        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(MAX_STARTUP_MS) {
            return Err(PzshError::StartupBudgetExceeded(
                MAX_STARTUP_MS,
                elapsed.as_millis() as u64,
            ));
        }

        Ok(Self {
            config,
            parser,
            executor,
            prompt,
            shell_type,
        })
    }

    /// Get startup time in microseconds
    #[must_use]
    pub fn measure_startup(&self) -> Duration {
        let start = std::time::Instant::now();
        let config = config::CompiledConfig::default();
        let _ = parser::Parser::new(&config);
        let _ = executor::Executor::new(&config);
        let _ = prompt::Prompt::new(&config);
        start.elapsed()
    }

    /// Get shell type
    #[must_use]
    pub const fn shell_type(&self) -> ShellType {
        self.shell_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    /// ANDON: This test MUST pass. If startup exceeds 10ms, we stop the line.
    #[test]
    fn test_startup_under_10ms() {
        let config = config::CompiledConfig::default();
        let start = Instant::now();
        let result = Pzsh::new(config);
        let elapsed = start.elapsed();

        assert!(result.is_ok(), "Pzsh initialization failed");
        assert!(
            elapsed < Duration::from_millis(MAX_STARTUP_MS),
            "ANDON: Startup exceeded 10ms budget: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_startup_is_deterministic() {
        let mut times = Vec::with_capacity(100);

        for _ in 0..100 {
            let config = config::CompiledConfig::default();
            let start = Instant::now();
            let _ = Pzsh::new(config);
            times.push(start.elapsed());
        }

        let mean: Duration = times.iter().sum::<Duration>() / times.len() as u32;
        let variance: f64 = times
            .iter()
            .map(|t| {
                let diff = t.as_nanos() as f64 - mean.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>()
            / times.len() as f64;
        let std_dev = variance.sqrt();

        // Standard deviation should be less than 1ms
        assert!(
            std_dev < 1_000_000.0,
            "Startup time variance too high: std_dev = {std_dev}ns"
        );
    }

    #[test]
    fn test_shell_type_default_is_zsh() {
        let config = config::CompiledConfig::default();
        let pzsh = Pzsh::new(config).unwrap();
        assert_eq!(pzsh.shell_type(), ShellType::Zsh);
    }

    #[test]
    fn test_shell_type_bash() {
        let mut config = config::CompiledConfig::default();
        config.shell_type = ShellType::Bash;
        let pzsh = Pzsh::new(config).unwrap();
        assert_eq!(pzsh.shell_type(), ShellType::Bash);
    }

    #[test]
    fn test_measure_startup() {
        let config = config::CompiledConfig::default();
        let pzsh = Pzsh::new(config).unwrap();
        let duration = pzsh.measure_startup();
        // Should be very fast
        assert!(
            duration < Duration::from_millis(MAX_STARTUP_MS),
            "measure_startup took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_error_display() {
        let err = PzshError::StartupBudgetExceeded(10, 15);
        assert!(err.to_string().contains("startup exceeded"));

        let err = PzshError::ParserBudgetExceeded(2, 5);
        assert!(err.to_string().contains("parser exceeded"));

        let err = PzshError::ExecutorBudgetExceeded(2, 4);
        assert!(err.to_string().contains("executor exceeded"));

        let err = PzshError::PromptBudgetExceeded(2, 3);
        assert!(err.to_string().contains("prompt exceeded"));

        let err = PzshError::ForbiddenPattern("test pattern".to_string());
        assert!(err.to_string().contains("forbidden pattern"));
    }

    #[test]
    fn test_shell_type_default() {
        let shell = ShellType::default();
        assert_eq!(shell, ShellType::Zsh);
    }
}
