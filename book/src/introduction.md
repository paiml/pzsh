# Introduction

**pzsh** is a performance-first shell framework with a hard constraint: **no shell startup shall exceed 10ms**.

## Why pzsh?

Modern shell frameworks like oh-my-zsh prioritize features over performance, resulting in startup times of 500-2000ms. This violates fundamental principles of developer experience—every time you open a terminal, you wait.

pzsh takes a different approach:

| Framework | Startup Time | vs pzsh |
|-----------|-------------|---------|
| **pzsh** | **<1ms** | 1x |
| bare zsh | 5-10ms | 10x |
| zinit | 100-300ms | 300x |
| prezto | 200-500ms | 500x |
| oh-my-zsh | 500-2000ms | 2000x |

## Core Invariant

The 10ms constraint is not a goal—it is enforced at:

- **Compile time**: Forbidden patterns are rejected
- **Test time**: Every test enforces time budgets
- **Runtime**: Startup is measured and reported

```bash
$ pzsh bench
Startup Benchmark (100 iterations)
────────────────────────────────
mean:      0.032ms
p99:       0.051ms
────────────────────────────────
Budget: 10ms ✓
```

## Philosophy

pzsh follows the **Toyota Production System** principles:

1. **Stop the line on defects** (Andon) - If startup exceeds budget, we fail fast
2. **Continuous improvement** (Kaizen) - Every microsecond matters
3. **Go and see for yourself** (Genchi Genbutsu) - All claims are measured, not estimated

## Built With

- [bashrs](https://github.com/paiml/bashrs) - Rust↔Shell transpiler for safety
- [aprender](https://github.com/paiml/aprender) - ML-powered optimization

## Quick Example

```toml
# ~/.pzshrc
[pzsh]
shell = "zsh"

[aliases]
ll = "ls -la"
gs = "git status"

[env]
EDITOR = "vim"
GOROOT = "/usr/local/opt/go/libexec"  # Pre-resolved, not $(brew --prefix)
```

Ready to get started? See [Installation](./getting-started/installation.md).
