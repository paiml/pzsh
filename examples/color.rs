//! Color and styling example
//!
//! Run with: cargo run --example color

use pzsh::color::{Color, ColorSpec, Style, Styled, RESET, themes::DefaultTheme};

fn main() {
    println!("=== pzsh Color System ===\n");

    // Basic ANSI colors (16-color palette)
    println!("16-Color Palette:");
    for color in [
        Color::Black, Color::Red, Color::Green, Color::Yellow,
        Color::Blue, Color::Magenta, Color::Cyan, Color::White,
    ] {
        let style = Style::new().fg_ansi(color);
        let styled = Styled::new(format!("{color:?}"), style);
        print!("  {} ", styled);
    }
    println!("\n");

    // Bright colors
    println!("Bright Colors:");
    for color in [
        Color::BrightBlack, Color::BrightRed, Color::BrightGreen, Color::BrightYellow,
        Color::BrightBlue, Color::BrightMagenta, Color::BrightCyan, Color::BrightWhite,
    ] {
        let style = Style::new().fg_ansi(color);
        let styled = Styled::new(format!("{color:?}"), style);
        print!("  {} ", styled);
    }
    println!("\n");

    // Style modifiers
    println!("Style Modifiers:");
    println!("  {} - bold text", Styled::new("Bold", Style::new().bold()));
    println!("  {} - dim text", Styled::new("Dim", Style::new().dim()));
    println!("  {} - italic text", Styled::new("Italic", Style::new().italic()));
    println!("  {} - underlined text", Styled::new("Underline", Style::new().underline()));
    println!();

    // Combined styles
    println!("Combined Styles:");
    let fancy = Style::new()
        .fg_ansi(Color::Cyan)
        .bold()
        .underline();
    println!("  {}", Styled::new("Bold + Cyan + Underline", fancy));
    println!();

    // 256-color palette
    println!("256-Color Palette (sample):");
    print!("  ");
    for i in (16..232).step_by(6) {
        let style = Style::new().fg(ColorSpec::Palette(i));
        print!("{}", Styled::new("█", style));
    }
    println!("\n");

    // True color (24-bit RGB)
    println!("True Color Gradient:");
    print!("  ");
    for r in (0..=255).step_by(8) {
        let style = Style::new().fg(ColorSpec::Rgb(r as u8, 100, 200));
        print!("{}", Styled::new("█", style));
    }
    println!("\n");

    // Theme styles (oh-my-zsh compatible)
    println!("Default Theme Styles:");
    println!("  {} - user style", Styled::new("noah", DefaultTheme::user()));
    println!("  {} - host style", Styled::new("laptop", DefaultTheme::host()));
    println!("  {} - cwd style", Styled::new("~/src/pzsh", DefaultTheme::cwd()));
    println!("  {} - git clean", Styled::new("(main)", DefaultTheme::git_clean()));
    println!("  {} - git dirty", Styled::new("(main*)", DefaultTheme::git_dirty()));
    println!("  {} - error", Styled::new("Error!", DefaultTheme::error()));
    println!("  {} - success", Styled::new("Success", DefaultTheme::success()));
    println!("  {} - warning", Styled::new("Warning", DefaultTheme::warning()));
    println!();

    // Raw ANSI codes
    println!("Raw ANSI Escape Sequences:");
    let style = Style::new().fg_ansi(Color::Green).bold();
    println!("  Style code: {:?}", style.to_ansi());
    println!("  Reset code: {:?}", RESET);
}
