//! Lint configuration example - detect slow patterns
//!
//! Run with: cargo run --example lint_config

use pzsh::cli::lint_config;

fn main() {
    println!("pzsh Configuration Linter");
    println!("═════════════════════════");
    println!();

    // Example 1: Clean configuration
    let clean_config = r#"
export EDITOR="vim"
export GOROOT="/usr/local/opt/go/libexec"
alias ll="ls -la"
alias gs="git status"
"#;

    println!("Example 1: Clean configuration");
    println!("──────────────────────────────");
    let result = lint_config(clean_config);
    println!("{}", result.format());
    println!();

    // Example 2: Configuration with slow patterns
    let slow_config = r#"
# This has multiple issues!
export GOROOT="$(brew --prefix golang)/libexec"
export DATE=`date`
eval "$(pyenv init -)"
source $ZSH/oh-my-zsh.sh
source ~/.nvm/nvm.sh
source ~/miniconda3/etc/profile.d/conda.sh
"#;

    println!("Example 2: Configuration with slow patterns");
    println!("────────────────────────────────────────────");
    let result = lint_config(slow_config);
    println!("{}", result.format());
    println!();

    // Summary
    println!("Key takeaways:");
    println!("  • Avoid $(...) and backticks at startup");
    println!("  • Pre-resolve paths like brew --prefix");
    println!("  • Replace oh-my-zsh with pzsh plugins");
    println!("  • Lazy-load NVM, conda, pyenv");
}
