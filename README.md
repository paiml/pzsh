# pzsh

[![Crates.io](https://img.shields.io/crates/v/pzsh.svg)](https://crates.io/crates/pzsh)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Performance-first shell framework with sub-10ms startup. Like oh-my-zsh, but **50-200x faster**.

## Core Invariant

**No shell startup shall exceed 10ms.** This is not a goal—it is a hard constraint enforced at compile time, test time, and runtime.

## Performance

```
$ pzsh bench
Startup Benchmark (100 iterations)
────────────────────────────────
min:       0.030ms
max:       0.051ms
mean:      0.032ms
p99:       0.051ms
────────────────────────────────
Budget: 10ms ✓ (p99 < 10ms)
```

| Framework | Startup | vs pzsh |
|-----------|---------|---------|
| pzsh | **<1ms** | 1x |
| bare zsh | 5-10ms | 10x |
| zinit | 100-300ms | 300x |
| prezto | 200-500ms | 500x |
| oh-my-zsh | 500-2000ms | 2000x |

## Installation

```bash
cargo install pzsh
```

## Usage

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
```

## Configuration

```toml
# ~/.pzshrc
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
GOROOT = "/usr/local/opt/go/libexec"  # Pre-resolved, no $(brew ...)
```

## Forbidden Patterns

pzsh enforces O(1) startup by rejecting slow patterns:

```bash
# FORBIDDEN: subprocess calls at startup
export GOROOT="$(brew --prefix golang)/libexec"  # 50-100ms per call

# ALLOWED: pre-resolved paths
export GOROOT="/usr/local/opt/go/libexec"  # 0ms

# FORBIDDEN: oh-my-zsh, NVM, conda init
source $ZSH/oh-my-zsh.sh  # 500-2000ms
```

## Architecture

- **Parser**: O(1) with LRU caching, 2ms budget
- **Executor**: O(1) hash lookups, 2ms budget
- **Prompt**: Async git status, 2ms budget
- **Config**: Pre-compiled TOML, no runtime parsing

## Testing

```bash
cargo test      # 64 tests, all enforcing time budgets
cargo bench     # Criterion benchmarks
```

## Built With

- [bashrs](https://github.com/paiml/bashrs) - Rust↔Shell transpiler for safety
- [aprender](https://github.com/paiml/aprender) - ML-powered optimization

## Toyota Way

Development follows the [Toyota Production System](docs/specifications/pzsh-spec.md):
- Stop the line on defects (Andon)
- Continuous improvement (Kaizen)
- Go and see for yourself (Genchi Genbutsu)

## License

MIT
