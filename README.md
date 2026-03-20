# pzsh

[![CI](https://github.com/paiml/pzsh/actions/workflows/ci.yml/badge.svg)](https://github.com/paiml/pzsh/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/pzsh.svg)](https://crates.io/crates/pzsh)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Test Coverage](https://img.shields.io/badge/coverage-97%25-brightgreen.svg)](https://github.com/paiml/pzsh)

Performance-first shell framework with sub-10ms startup. Like oh-my-zsh, but **50-200x faster**.

## ⚡ Demo

<div align="center">

```
┌─────────────────────────────────────────────────────────────┐
│  noah@dev ~/src/pzsh (main)                                 │
│  ❯ pzsh bench                                               │
│                                                             │
│  Startup Benchmark (100 iterations)                         │
│  ────────────────────────────────                           │
│  min:       0.002ms   ████                                  │
│  max:       0.003ms   ████                                  │
│  mean:      0.003ms   ████                                  │
│  p99:       0.003ms   ████                                  │
│  ────────────────────────────────                           │
│  Budget: 10ms ✓ (p99 < 10ms)                                │
└─────────────────────────────────────────────────────────────┘
```

</div>

## Core Invariant

**No shell startup shall exceed 10ms.** This is not a goal—it is a hard constraint enforced at compile time, test time, and runtime.

## 🚀 Performance

| Framework | Startup | vs pzsh |
|-----------|---------|---------|
| **pzsh** | **<1ms** | 1x |
| bare zsh | 5-10ms | 10x |
| zinit | 100-300ms | 300x |
| prezto | 200-500ms | 500x |
| oh-my-zsh | 500-2000ms | 2000x |

### Benchmark Output

```ansi
Startup Benchmark (100 iterations)
────────────────────────────────
min:       0.002ms
max:       0.003ms
mean:      0.003ms
p50:       0.003ms
p95:       0.003ms
p99:       0.003ms
────────────────────────────────
Budget: 10ms ✓ (p99 < 10ms)
```

### Profile Breakdown

```
Startup Profile
├─ parse:   0.007ms
├─ env:     0.000ms
├─ alias:   0.000ms
├─ prompt:  0.005ms
└─ total:   0.013ms ✓
```

## 📦 Installation

```bash
cargo install pzsh
```

### Add to your shell

```bash
# For zsh (~/.zshrc)
eval "$(pzsh init zsh)"

# For bash (~/.bashrc)
eval "$(pzsh init bash)"
```

## 🎨 Features

### oh-my-zsh Compatibility

pzsh provides drop-in replacements for common oh-my-zsh features:

- **Git plugin** - `g`, `ga`, `gc`, `gp`, `gst` aliases
- **Docker plugin** - `d`, `di`, `dps`, `dex` aliases
- **Colored prompts** - Git branch with dirty status
- **Themes** - robbyrussell, agnoster, pure, minimal

### Prompt Preview

```
┌──────────────────────────────────────────────────────────┐
│  robbyrussell theme                                      │
│  ➜ ~/src/pzsh (main*) git status                        │
├──────────────────────────────────────────────────────────┤
│  agnoster theme                                          │
│  noah │ dev │ ~/src/pzsh │ main* │ ❯                    │
├──────────────────────────────────────────────────────────┤
│  pure theme                                              │
│  ~/src/pzsh main*                                        │
│  ❯                                                       │
├──────────────────────────────────────────────────────────┤
│  minimal theme                                           │
│  > ls -la                                                │
└──────────────────────────────────────────────────────────┘
```

### ML-Powered Completions

Optional integration with [aprender-shell](https://crates.io/crates/aprender-shell) for intelligent command predictions:

```bash
# Load your trained model
pzsh completion --model ~/.local/share/pzsh/model.apr

# Predictions trained on your command history
$ git c<TAB>
  commit (0.85)   checkout (0.72)   clone (0.45)
```

## 🔧 Usage

```bash
# Initialize configuration
pzsh init --shell zsh

# Benchmark startup time
pzsh bench

# Lint for slow patterns
pzsh lint ~/.pzshrc

# Profile startup breakdown
pzsh profile

# Compile configuration
pzsh compile

# Check status
pzsh status
```

## ⚙️ Configuration

```toml
# ~/.pzshrc
[pzsh]
version = "0.2.0"
shell = "zsh"

[performance]
startup_budget_ms = 10
lazy_load = true

[prompt]
theme = "robbyrussell"
git_status = true
colors = true

[plugins]
enabled = ["git", "docker"]

[aliases]
ll = "ls -la"
gs = "git status"

[env]
EDITOR = "vim"
GOROOT = "/usr/local/opt/go/libexec"  # Pre-resolved, no $(brew ...)
```

## 🚫 Forbidden Patterns

pzsh enforces O(1) startup by rejecting slow patterns:

```bash
# FORBIDDEN: subprocess calls at startup
export GOROOT="$(brew --prefix golang)/libexec"  # 50-100ms per call

# ALLOWED: pre-resolved paths
export GOROOT="/usr/local/opt/go/libexec"  # 0ms

# FORBIDDEN: oh-my-zsh, NVM, conda init
source $ZSH/oh-my-zsh.sh  # 500-2000ms

# ALLOWED: pzsh lazy loading
eval "$(pzsh init zsh)"  # <1ms
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│              pzsh init zsh              │
├─────────────────────────────────────────┤
│  Parser     │ O(1) LRU cache   │ 2ms   │
│  Executor   │ O(1) hash lookup │ 2ms   │
│  Prompt     │ Async git status │ 2ms   │
│  Config     │ Pre-compiled     │ 0ms   │
├─────────────────────────────────────────┤
│  Total Budget                  │ 10ms  │
└─────────────────────────────────────────┘
```

## 🧪 Testing

```bash
# Run all tests (335 tests, 97% coverage)
cargo test

# Run benchmarks
cargo bench

# Run with coverage
cargo llvm-cov --html
```

## 📖 Examples

```bash
# Color system demo
cargo run --example color

# Theme preview (robbyrussell, agnoster, pure, minimal)
cargo run --example theme

# Plugin system (git, docker aliases)
cargo run --example plugin

# Completion system
cargo run --example completion

# Shell initialization script
cargo run --example shell_init

# Zsh features (autocd, history, keybindings)
cargo run --example zsh_features

# Benchmark performance
cargo run --example benchmark

# Configuration parsing
cargo run --example basic_config

# Lint configuration
cargo run --example lint_config

# Prompt rendering
cargo run --example prompt

# Parser demo
cargo run --example parser
```

## 🔗 Built With

- [aprender-shell](https://crates.io/crates/aprender-shell) v0.3 - ML completions
- [trueno](https://crates.io/crates/trueno) v0.15 - SIMD acceleration

## 📚 Toyota Way

Development follows the [Toyota Production System](docs/specifications/pzsh-spec.md):

- **Andon** - Stop the line on defects
- **Kaizen** - Continuous improvement
- **Genchi Genbutsu** - Go and see for yourself

## Contributing

Contributions welcome. Please run `make lint test` before submitting PRs.

## 📄 License

MIT
