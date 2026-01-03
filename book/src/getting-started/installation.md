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
pzsh v0.1.0
────────────────────────────
Startup: 0.03ms (budget: 10ms) ✓
```

## Initialize Configuration

```bash
pzsh init --shell zsh
```

This creates `~/.pzshrc` with a minimal configuration.

## Add to Shell

Add to your `~/.zshrc`:

```bash
# pzsh initialization
eval "$(pzsh init)"
```

Or for bash, add to `~/.bashrc`:

```bash
# pzsh initialization
eval "$(pzsh init)"
```
