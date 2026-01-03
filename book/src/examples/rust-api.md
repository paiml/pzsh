# Rust API Examples

Use pzsh as a library in your Rust projects.

## Add Dependency

```toml
[dependencies]
pzsh = "0.2"
```

## Parse Configuration

```rust
use pzsh::config::{Config, CompiledConfig};

fn main() {
    // Parse from TOML string
    let toml = r#"
        [pzsh]
        version = "0.2.0"
        shell = "zsh"

        [aliases]
        ll = "ls -la"
        gs = "git status"
    "#;

    let config: Config = toml::from_str(toml).unwrap();
    let compiled = config.compile();

    println!("Aliases: {:?}", compiled.aliases);
}
```

## Generate Shell Init

```rust
use pzsh::shell::generate_init;
use pzsh::config::CompiledConfig;
use pzsh::ShellType;

fn main() {
    let mut config = CompiledConfig::default();
    config.colors_enabled = true;
    config.aliases.insert("ll".into(), "ls -la".into());

    // Generate zsh init script
    let script = generate_init(ShellType::Zsh, config);
    println!("{}", script);
}
```

## Use Plugins

```rust
use pzsh::plugin::{PluginManager, Plugin};
use pzsh::ShellType;

fn main() {
    let mut plugins = PluginManager::new();

    // Load built-in plugins
    plugins.load("git").unwrap();
    plugins.load("docker").unwrap();

    // Get all aliases
    for (name, expansion) in plugins.all_aliases() {
        println!("alias {}='{}'", name, expansion);
    }

    // Generate shell init code
    let init = plugins.shell_init(ShellType::Zsh);
    println!("{}", init);
}
```

## Theme System

```rust
use pzsh::theme::{Theme, ThemeRegistry};
use pzsh::ShellType;

fn main() {
    let registry = ThemeRegistry::new();

    // List available themes
    for name in registry.list() {
        println!("Theme: {}", name);
    }

    // Get a theme
    if let Some(theme) = registry.get("robbyrussell") {
        let prompt = theme.render_prompt(ShellType::Zsh);
        println!("PROMPT='{}'", prompt);
    }
}
```

## Completion System

```rust
use pzsh::completion::{
    CompletionEngine, CompletionContext,
    HistoryCompleter, PathCompleter
};

fn main() {
    let mut engine = CompletionEngine::new();

    // Add completion providers
    engine.add_provider(Box::new(PathCompleter::new()));
    engine.add_provider(Box::new(HistoryCompleter::new(vec![
        "git status".into(),
        "git commit -m".into(),
        "cargo build".into(),
    ])));

    // Get completions
    let ctx = CompletionContext::new("git st", 6);
    let completions = engine.complete(&ctx);

    for item in completions {
        println!("{} ({:?})", item.text, item.kind);
    }
}
```

## Prompt Rendering

```rust
use pzsh::prompt::{PromptBuilder, PromptSegment};

fn main() {
    let prompt = PromptBuilder::new()
        .segment(PromptSegment::Username)
        .literal("@")
        .segment(PromptSegment::Hostname)
        .literal(" ")
        .segment(PromptSegment::Directory)
        .literal(" ")
        .segment(PromptSegment::GitBranch)
        .literal(" $ ")
        .build();

    println!("PROMPT='{}'", prompt.render_zsh());
}
```

## Run Examples

All examples are in the `examples/` directory:

```bash
# Basic configuration
cargo run --example basic_config

# Color system demo
cargo run --example color

# Theme preview
cargo run --example theme

# Plugin system
cargo run --example plugin

# Completion demo
cargo run --example completion

# Shell initialization
cargo run --example shell_init

# Zsh features
cargo run --example zsh_features

# Benchmarking
cargo run --example benchmark

# Config linting
cargo run --example lint_config

# Parser demo
cargo run --example parser

# Prompt rendering
cargo run --example prompt
```
