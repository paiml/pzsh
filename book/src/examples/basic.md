# Basic Configuration

A simple pzsh configuration for everyday use.

## Configuration File

Create `~/.pzshrc`:

```toml
# pzsh configuration
[pzsh]
version = "0.2.0"
shell = "zsh"

[performance]
startup_budget_ms = 10
lazy_load = true

[prompt]
colors = true

[aliases]
ll = "ls -la"
la = "ls -A"
l = "ls -CF"

[env]
EDITOR = "vim"

[plugins]
enabled = ["git"]
```

## Activation

Add to `~/.zshrc`:

```bash
eval "$(pzsh compile)"
```

## What You Get

- Colored prompt with git status
- Common aliases (ll, la, l)
- Git plugin aliases (g, gs, ga, gc, gp, etc.)
- 50k history with deduplication
- Fast completion system

## Running Examples

```bash
# Color system
cargo run --example color

# Shell initialization
cargo run --example shell_init

# Theme preview
cargo run --example theme

# Completion demo
cargo run --example completion

# Plugin system
cargo run --example plugin

# Zsh features
cargo run --example zsh_features
```

## Verify Performance

```bash
pzsh status
# pzsh v0.2.0
# Startup: 0.00ms (budget: 10ms) ✓
```
