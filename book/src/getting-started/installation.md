# Installation

## From Cargo (Recommended)

```bash
cargo install pzsh
```

## From Source

```bash
git clone https://github.com/paiml/pzsh
cd pzsh
cargo install --path .
```

## Verify Installation

```bash
pzsh --version
pzsh status
```

Expected output:
```
pzsh v0.2.1
────────────────────────────
Startup: 0.01ms (budget: 10ms) ✓
```

## Initialize Configuration

For zsh (default):
```bash
pzsh init
```

For bash:
```bash
pzsh init --shell bash
```

This creates `~/.pzshrc` with a minimal configuration.

## Add to Shell

Add to your `~/.zshrc`:

```bash
# pzsh shell framework
eval "$(pzsh compile)"
```

Or for bash, add to `~/.bashrc`:

```bash
# pzsh shell framework
eval "$(pzsh compile)"
```

## Shell Support

pzsh supports both **zsh** and **bash** with near-complete feature parity:

| Feature | zsh | bash |
|---------|-----|------|
| Plugins (9) | Full | Full |
| Keybindings | 18 | 22 |
| Completion | oh-my-zsh style | oh-my-zsh style |
| Auto-suggestions | Yes | No (zsh-only) |
| Syntax highlighting | Yes | No (zsh-only) |
| Startup time | <10ms | <10ms |
