# Colors

pzsh provides a high-performance color system with O(1) rendering, supporting 16-color ANSI, 256-color palette, and true color (24-bit RGB).

## Enabling Colors

Colors are enabled by default in `~/.pzshrc`:

```toml
[prompt]
colors = true
```

## Color Palette

### 16-Color ANSI

Standard terminal colors with both normal and bright variants:

| Color | Normal | Bright |
|-------|--------|--------|
| Black | 0 | 8 |
| Red | 1 | 9 |
| Green | 2 | 10 |
| Yellow | 3 | 11 |
| Blue | 4 | 12 |
| Magenta | 5 | 13 |
| Cyan | 6 | 14 |
| White | 7 | 15 |

### 256-Color Palette

Access the full 256-color palette using palette indices:

```rust
ColorSpec::Palette(196)  // Bright red
ColorSpec::Palette(46)   // Bright green
```

### True Color (24-bit RGB)

Full RGB color support for modern terminals:

```rust
ColorSpec::Rgb(255, 128, 64)  // Orange
```

## Style Modifiers

Combine colors with text styles:

- **Bold**: `style.bold()`
- **Dim**: `style.dim()`
- **Italic**: `style.italic()`
- **Underline**: `style.underline()`

## Default Theme

pzsh includes oh-my-zsh compatible default styles:

| Element | Style |
|---------|-------|
| User | Green + Bold |
| Host | Blue + Bold |
| CWD | Cyan |
| Git (clean) | Green |
| Git (dirty) | Yellow |
| Error | Red + Bold |
| Success | Green |

## Environment Detection

pzsh automatically detects terminal capabilities:

- `NO_COLOR`: Disables all colors (https://no-color.org/)
- `CLICOLOR_FORCE`: Forces colors even without TTY
- `COLORTERM=truecolor`: Enables 24-bit color

## Performance

Color rendering is O(1) with pre-computed ANSI escape sequences. Style application adds no measurable overhead to prompt generation.
