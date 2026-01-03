# Themes

pzsh includes several oh-my-zsh compatible themes with O(1) rendering.

## Available Themes

### robbyrussell (Default)

The classic oh-my-zsh default theme:

```
➜ ~/src/pzsh git:(main) ✗
```

Features:
- Arrow prompt character
- Git branch with dirty indicator
- Colored directory

### agnoster

Powerline-style theme with segments:

```
user@host | ~/src/pzsh | main ✗ |
```

Features:
- User and host segment
- Current directory
- Git status with symbols

### simple

Minimal theme for fast rendering:

```
user@host:~/src/pzsh$
```

Features:
- No git integration (fastest)
- Traditional prompt format

### pure

Clean, async-ready theme:

```
~/src/pzsh main*
❯
```

Features:
- Two-line prompt
- Git info on first line
- Minimal second line

### spaceship

Feature-rich theme:

```
~/src/pzsh on main [!] via rust v1.75
❯
```

Features:
- Language version detection
- Extensive git status
- Package version display

## Theme Selection

Themes are selected in `~/.pzshrc`:

```toml
[prompt]
theme = "robbyrussell"
```

## Custom Themes

Create custom themes by implementing the `Theme` trait:

```rust
pub trait Theme: Send + Sync {
    fn name(&self) -> &str;
    fn zsh_prompt(&self) -> String;
    fn bash_prompt(&self) -> String;
    fn user_style(&self) -> Style;
    fn host_style(&self) -> Style;
    fn cwd_style(&self) -> Style;
    fn git_clean_style(&self) -> Style;
    fn git_dirty_style(&self) -> Style;
}
```

## Performance

All themes render in O(1) time with pre-computed escape sequences. Git status is cached for 1 second by default.
