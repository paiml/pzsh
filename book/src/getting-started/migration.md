# Migration from oh-my-zsh

pzsh provides oh-my-zsh compatible features with sub-10ms startup. This guide helps you migrate.

## Quick Migration

1. Install pzsh:
   ```bash
   cargo install pzsh
   ```

2. Initialize configuration:
   ```bash
   pzsh init --shell zsh
   ```

3. Add to `~/.zshrc`:
   ```bash
   eval "$(pzsh compile)"
   ```

4. Remove oh-my-zsh:
   ```bash
   # Comment out or remove:
   # source $ZSH/oh-my-zsh.sh
   ```

## Feature Mapping

| oh-my-zsh | pzsh | Status |
|-----------|------|--------|
| Colored prompt | `[prompt] colors = true` | ✓ |
| Git branch display | Built-in prompt | ✓ |
| Git dirty indicator | Built-in prompt | ✓ |
| git plugin aliases | `plugins = ["git"]` | ✓ |
| docker plugin | `plugins = ["docker"]` | ✓ |
| Themes | robbyrussell, agnoster, etc. | ✓ |
| Auto-suggestions | Built-in widget | ✓ |
| Syntax highlighting | Built-in widget | ✓ |
| History search | Built-in widget | ✓ |
| Directory jump (z) | Built-in z command | ✓ |
| Completion system | Native zsh + extensions | ✓ |

## Alias Migration

Copy your aliases to `~/.pzshrc`:

**oh-my-zsh:**
```zsh
alias ll="ls -la"
alias gst="git status"
```

**pzsh:**
```toml
[aliases]
ll = "ls -la"
gst = "git status"
```

## Plugin Migration

### Git Plugin

oh-my-zsh git aliases are built-in:

```toml
[plugins]
enabled = ["git"]
```

Provides: `g`, `gs`, `ga`, `gc`, `gp`, `gl`, `gd`, `gco`, `gb`, `glog`, and more.

### Docker Plugin

```toml
[plugins]
enabled = ["docker"]
```

Provides: `d`, `dps`, `dimg`, `drun`, `dexec`, and more.

## Environment Variables

**oh-my-zsh:**
```zsh
export EDITOR="vim"
export GOPATH="$HOME/go"
```

**pzsh:**
```toml
[env]
EDITOR = "vim"
GOPATH = "$HOME/go"
```

## Startup Comparison

| Configuration | oh-my-zsh | pzsh |
|--------------|-----------|------|
| Minimal | ~200ms | <1ms |
| Git plugin | ~300ms | <1ms |
| Full config | 500-2000ms | <10ms |

## Troubleshooting

### Slow startup after migration

Check for remaining oh-my-zsh artifacts:
```bash
pzsh lint ~/.zshrc
```

Common issues:
- Leftover `source $ZSH/oh-my-zsh.sh`
- NVM without lazy loading
- Conda init blocks

### Missing aliases

Verify plugins are enabled:
```bash
pzsh compile | grep "alias g="
```

### No colors

Check terminal support:
```bash
echo $TERM
echo $COLORTERM
```

Ensure `colors = true` in config.
