//! CLI module for pzsh
//!
//! Commands: bench, lint, compile, fix, profile, status

use crate::config::CompiledConfig;
use crate::{MAX_STARTUP_MS, Pzsh};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// pzsh: Performance-first shell framework
#[derive(Parser, Debug)]
#[command(name = "pzsh")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Benchmark shell startup time
    Bench {
        /// Number of iterations
        #[arg(short, long, default_value = "100")]
        iterations: u32,

        /// Show detailed statistics
        #[arg(short, long)]
        verbose: bool,
    },

    /// Lint configuration for slow patterns
    Lint {
        /// Path to configuration file
        #[arg(short, long, default_value = "~/.pzshrc")]
        config: PathBuf,
    },

    /// Compile configuration to optimized form
    Compile {
        /// Path to source configuration
        #[arg(short, long, default_value = "~/.pzshrc")]
        config: PathBuf,

        /// Output path for compiled config
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Auto-fix slow patterns
    Fix {
        /// Path to configuration file
        #[arg(short, long, default_value = "~/.pzshrc")]
        config: PathBuf,

        /// Dry run (show fixes without applying)
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Profile detailed startup breakdown
    Profile {
        /// Show timing for each component
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show pzsh status
    Status,

    /// Initialize pzsh configuration
    Init {
        /// Shell type (zsh or bash)
        #[arg(short, long, default_value = "zsh")]
        shell: String,
    },
}

/// Benchmark result
#[derive(Debug)]
pub struct BenchResult {
    pub iterations: u32,
    pub min: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub std_dev: Duration,
    pub passed: bool,
}

impl BenchResult {
    /// Format as string
    #[must_use]
    pub fn format(&self) -> String {
        let status = if self.passed { "✓" } else { "✗" };
        format!(
            "Startup Benchmark ({} iterations)\n\
             ────────────────────────────────\n\
             min:    {:>8.3}ms\n\
             max:    {:>8.3}ms\n\
             mean:   {:>8.3}ms\n\
             p50:    {:>8.3}ms\n\
             p95:    {:>8.3}ms\n\
             p99:    {:>8.3}ms\n\
             stddev: {:>8.3}ms\n\
             ────────────────────────────────\n\
             Budget: {}ms {} (p99 < {}ms)",
            self.iterations,
            self.min.as_secs_f64() * 1000.0,
            self.max.as_secs_f64() * 1000.0,
            self.mean.as_secs_f64() * 1000.0,
            self.p50.as_secs_f64() * 1000.0,
            self.p95.as_secs_f64() * 1000.0,
            self.p99.as_secs_f64() * 1000.0,
            self.std_dev.as_secs_f64() * 1000.0,
            MAX_STARTUP_MS,
            status,
            MAX_STARTUP_MS,
        )
    }
}

/// Run benchmark
pub fn run_bench(iterations: u32, _verbose: bool) -> BenchResult {
    let mut times: Vec<Duration> = Vec::with_capacity(iterations as usize);

    // Warmup
    for _ in 0..10 {
        let config = CompiledConfig::default();
        let _ = Pzsh::new(config);
    }

    // Benchmark
    for _ in 0..iterations {
        let config = CompiledConfig::default();
        let start = Instant::now();
        let _ = Pzsh::new(config);
        times.push(start.elapsed());
    }

    // Sort for percentiles
    times.sort();

    let min = times[0];
    let max = times[times.len() - 1];

    let sum: Duration = times.iter().sum();
    let mean = sum / iterations;

    let p50_idx = (times.len() as f64 * 0.50) as usize;
    let p95_idx = (times.len() as f64 * 0.95) as usize;
    let p99_idx = (times.len() as f64 * 0.99) as usize;

    let p50 = times[p50_idx.min(times.len() - 1)];
    let p95 = times[p95_idx.min(times.len() - 1)];
    let p99 = times[p99_idx.min(times.len() - 1)];

    // Calculate standard deviation
    let mean_nanos = mean.as_nanos() as f64;
    let variance: f64 = times
        .iter()
        .map(|t| {
            let diff = t.as_nanos() as f64 - mean_nanos;
            diff * diff
        })
        .sum::<f64>()
        / times.len() as f64;
    let std_dev = Duration::from_nanos(variance.sqrt() as u64);

    let passed = p99 < Duration::from_millis(MAX_STARTUP_MS);

    BenchResult {
        iterations,
        min,
        max,
        mean,
        p50,
        p95,
        p99,
        std_dev,
        passed,
    }
}

/// Lint result
#[derive(Debug)]
pub struct LintResult {
    pub issues: Vec<LintIssue>,
}

#[derive(Debug)]
pub struct LintIssue {
    pub severity: LintSeverity,
    pub message: String,
    pub line: Option<usize>,
    pub fix: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

impl LintResult {
    /// Check if lint passed (no errors)
    #[must_use]
    pub fn passed(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| matches!(i.severity, LintSeverity::Error))
    }

    /// Format as string
    #[must_use]
    pub fn format(&self) -> String {
        if self.issues.is_empty() {
            return "✓ 0 issues found".to_string();
        }

        let mut output = String::new();
        for issue in &self.issues {
            let severity = match issue.severity {
                LintSeverity::Error => "error",
                LintSeverity::Warning => "warning",
                LintSeverity::Info => "info",
            };

            let line_info = issue
                .line
                .map_or(String::new(), |l| format!(" (line {})", l));

            output.push_str(&format!("[{}]{}: {}\n", severity, line_info, issue.message));

            if let Some(fix) = &issue.fix {
                output.push_str(&format!("  fix: {}\n", fix));
            }
        }

        let error_count = self
            .issues
            .iter()
            .filter(|i| matches!(i.severity, LintSeverity::Error))
            .count();
        let warning_count = self
            .issues
            .iter()
            .filter(|i| matches!(i.severity, LintSeverity::Warning))
            .count();

        output.push_str(&format!(
            "\n{} errors, {} warnings",
            error_count, warning_count
        ));

        output
    }
}

/// Lint configuration content
pub fn lint_config(content: &str) -> LintResult {
    let mut issues = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        // Check for subprocess calls
        if line.contains("$(") {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                message: "subprocess call $() not allowed at startup".to_string(),
                line: Some(line_num),
                fix: Some("use pre-resolved path instead".to_string()),
            });
        }

        // Check for backticks
        if line.contains('`') && !line.trim_start().starts_with('#') {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                message: "backtick substitution not allowed".to_string(),
                line: Some(line_num),
                fix: Some("use pre-resolved value instead".to_string()),
            });
        }

        // Check for brew --prefix
        if line.contains("brew --prefix") {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                message: "brew --prefix is slow (50-100ms)".to_string(),
                line: Some(line_num),
                fix: Some("run `brew --prefix <formula>` once and hardcode the path".to_string()),
            });
        }

        // Check for eval
        if line.contains("eval ") && !line.trim_start().starts_with('#') {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                message: "eval not allowed for safety".to_string(),
                line: Some(line_num),
                fix: None,
            });
        }

        // Check for slow plugin managers
        if line.contains("oh-my-zsh") || line.contains("source $ZSH/oh-my-zsh.sh") {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                message: "oh-my-zsh is slow (500-2000ms startup)".to_string(),
                line: Some(line_num),
                fix: Some("remove oh-my-zsh, use pzsh plugins instead".to_string()),
            });
        }

        // Check for NVM
        if line.contains("nvm.sh") && !line.trim_start().starts_with('#') {
            issues.push(LintIssue {
                severity: LintSeverity::Warning,
                message: "NVM adds 200-500ms to startup".to_string(),
                line: Some(line_num),
                fix: Some("use fnm or volta instead, or lazy-load NVM".to_string()),
            });
        }

        // Check for conda init
        if line.contains("conda init") || line.contains("conda.sh") {
            issues.push(LintIssue {
                severity: LintSeverity::Warning,
                message: "conda init adds 200-400ms to startup".to_string(),
                line: Some(line_num),
                fix: Some("lazy-load conda or use mamba".to_string()),
            });
        }
    }

    LintResult { issues }
}

/// Profile result
#[derive(Debug)]
pub struct ProfileResult {
    pub parse_time: Duration,
    pub env_time: Duration,
    pub alias_time: Duration,
    pub prompt_time: Duration,
    pub total_time: Duration,
    pub passed: bool,
}

impl ProfileResult {
    /// Format as string
    #[must_use]
    pub fn format(&self) -> String {
        let status = if self.passed { "✓" } else { "✗" };
        format!(
            "Startup Profile\n\
             ├─ parse:  {:>6.3}ms\n\
             ├─ env:    {:>6.3}ms\n\
             ├─ alias:  {:>6.3}ms\n\
             ├─ prompt: {:>6.3}ms\n\
             └─ total:  {:>6.3}ms {}\n",
            self.parse_time.as_secs_f64() * 1000.0,
            self.env_time.as_secs_f64() * 1000.0,
            self.alias_time.as_secs_f64() * 1000.0,
            self.prompt_time.as_secs_f64() * 1000.0,
            self.total_time.as_secs_f64() * 1000.0,
            status,
        )
    }
}

/// Run profile
pub fn run_profile() -> ProfileResult {
    use crate::executor::Executor;
    use crate::parser::Parser;
    use crate::prompt::Prompt;

    let config = CompiledConfig::default();
    let total_start = Instant::now();

    // Parser
    let start = Instant::now();
    let _ = Parser::new(&config);
    let parse_time = start.elapsed();

    // Executor (env + alias)
    let start = Instant::now();
    let _ = Executor::new(&config);
    let env_time = start.elapsed();

    let alias_time = Duration::ZERO; // Included in executor

    // Prompt
    let start = Instant::now();
    let _ = Prompt::new(&config);
    let prompt_time = start.elapsed();

    let total_time = total_start.elapsed();
    let passed = total_time < Duration::from_millis(MAX_STARTUP_MS);

    ProfileResult {
        parse_time,
        env_time,
        alias_time,
        prompt_time,
        total_time,
        passed,
    }
}

/// Generate initial configuration
#[must_use]
pub fn generate_init_config(shell: &str) -> String {
    format!(
        r#"# pzsh configuration
# Performance-first shell framework with sub-10ms startup

[pzsh]
version = "0.1.0"
shell = "{shell}"

[performance]
startup_budget_ms = 10
prompt_budget_ms = 2
lazy_load = true

[prompt]
format = "{{user}}@{{host}} {{cwd}} {{git}} {{char}} "
git_async = true
git_cache_ms = 1000

[aliases]
# Add your aliases here (no subprocess calls!)
ll = "ls -la"
gs = "git status"
gp = "git push"

[env]
# Add your environment variables here (pre-resolved paths only!)
EDITOR = "vim"
# GOROOT = "/usr/local/opt/go/libexec"  # Example: hardcoded, not $(brew --prefix)

[plugins]
enabled = ["git"]
lazy = []

[keybindings]
# ctrl-r = "history-search"
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_passes_under_10ms() {
        let result = run_bench(10, false);
        assert!(result.passed, "Benchmark should pass under 10ms");
    }

    #[test]
    fn test_lint_detects_subprocess() {
        let content = r#"
export GOROOT="$(brew --prefix golang)/libexec"
"#;
        let result = lint_config(content);
        assert!(!result.passed());
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_lint_detects_backticks() {
        let content = r#"
export DATE=`date`
"#;
        let result = lint_config(content);
        assert!(!result.passed());
    }

    #[test]
    fn test_lint_detects_oh_my_zsh() {
        let content = r#"
source $ZSH/oh-my-zsh.sh
"#;
        let result = lint_config(content);
        assert!(!result.passed());
    }

    #[test]
    fn test_lint_clean_config() {
        let content = r#"
export EDITOR="vim"
alias ll="ls -la"
"#;
        let result = lint_config(content);
        assert!(result.passed());
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_profile_under_10ms() {
        let result = run_profile();
        assert!(result.passed, "Profile should pass under 10ms");
    }

    #[test]
    fn test_generate_init_config() {
        let config = generate_init_config("zsh");
        assert!(config.contains("shell = \"zsh\""));
        assert!(config.contains("startup_budget_ms = 10"));
    }

    #[test]
    fn test_lint_detects_nvm() {
        let content = r#"
source ~/.nvm/nvm.sh
"#;
        let result = lint_config(content);
        assert!(!result.issues.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|i| i.message.contains("NVM")));
    }

    #[test]
    fn test_lint_detects_conda() {
        let content = r#"
source ~/miniconda3/etc/profile.d/conda.sh
"#;
        let result = lint_config(content);
        assert!(!result.issues.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|i| i.message.contains("conda")));
    }

    #[test]
    fn test_lint_detects_eval() {
        let content = r#"
eval "$(pyenv init -)"
"#;
        let result = lint_config(content);
        assert!(!result.passed());
        assert!(result
            .issues
            .iter()
            .any(|i| i.message.contains("eval")));
    }

    #[test]
    fn test_lint_ignores_comments() {
        let content = r#"
# eval "$(something)"
# `backticks`
# source nvm.sh
"#;
        let result = lint_config(content);
        // Comments should be ignored for backticks and eval, but not for other patterns
        let eval_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.message.contains("eval"))
            .collect();
        let backtick_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.message.contains("backtick"))
            .collect();
        assert!(eval_issues.is_empty(), "eval in comments should be ignored");
        assert!(
            backtick_issues.is_empty(),
            "backticks in comments should be ignored"
        );
    }

    #[test]
    fn test_bench_result_format() {
        let result = BenchResult {
            iterations: 100,
            min: Duration::from_micros(100),
            max: Duration::from_micros(500),
            mean: Duration::from_micros(200),
            p50: Duration::from_micros(180),
            p95: Duration::from_micros(400),
            p99: Duration::from_micros(450),
            std_dev: Duration::from_micros(50),
            passed: true,
        };
        let formatted = result.format();
        assert!(formatted.contains("100 iterations"));
        assert!(formatted.contains("Budget: 10ms"));
        assert!(formatted.contains("✓"));
    }

    #[test]
    fn test_bench_result_format_failed() {
        let result = BenchResult {
            iterations: 10,
            min: Duration::from_millis(5),
            max: Duration::from_millis(15),
            mean: Duration::from_millis(12),
            p50: Duration::from_millis(11),
            p95: Duration::from_millis(14),
            p99: Duration::from_millis(15),
            std_dev: Duration::from_millis(2),
            passed: false,
        };
        let formatted = result.format();
        assert!(formatted.contains("✗"));
    }

    #[test]
    fn test_lint_result_format_with_issues() {
        let result = LintResult {
            issues: vec![
                LintIssue {
                    severity: LintSeverity::Error,
                    message: "test error".to_string(),
                    line: Some(10),
                    fix: Some("fix it".to_string()),
                },
                LintIssue {
                    severity: LintSeverity::Warning,
                    message: "test warning".to_string(),
                    line: None,
                    fix: None,
                },
                LintIssue {
                    severity: LintSeverity::Info,
                    message: "test info".to_string(),
                    line: Some(5),
                    fix: None,
                },
            ],
        };
        let formatted = result.format();
        assert!(formatted.contains("[error]"));
        assert!(formatted.contains("[warning]"));
        assert!(formatted.contains("[info]"));
        assert!(formatted.contains("(line 10)"));
        assert!(formatted.contains("fix: fix it"));
        assert!(formatted.contains("1 errors, 1 warnings"));
    }

    #[test]
    fn test_lint_result_format_empty() {
        let result = LintResult { issues: vec![] };
        let formatted = result.format();
        assert!(formatted.contains("0 issues found"));
    }

    #[test]
    fn test_profile_result_format() {
        let result = ProfileResult {
            parse_time: Duration::from_micros(500),
            env_time: Duration::from_micros(100),
            alias_time: Duration::from_micros(50),
            prompt_time: Duration::from_micros(200),
            total_time: Duration::from_micros(850),
            passed: true,
        };
        let formatted = result.format();
        assert!(formatted.contains("parse:"));
        assert!(formatted.contains("env:"));
        assert!(formatted.contains("total:"));
        assert!(formatted.contains("✓"));
    }

    #[test]
    fn test_profile_result_format_failed() {
        let result = ProfileResult {
            parse_time: Duration::from_millis(5),
            env_time: Duration::from_millis(3),
            alias_time: Duration::from_millis(2),
            prompt_time: Duration::from_millis(4),
            total_time: Duration::from_millis(14),
            passed: false,
        };
        let formatted = result.format();
        assert!(formatted.contains("✗"));
    }

    #[test]
    fn test_generate_init_config_bash() {
        let config = generate_init_config("bash");
        assert!(config.contains("shell = \"bash\""));
    }
}
