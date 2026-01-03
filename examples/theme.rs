//! Theme system example
//!
//! Run with: cargo run --example theme

use pzsh::theme::{
    Theme, ThemeRegistry,
    RobbyRussellTheme, AgnosterTheme, SimpleTheme, PureTheme, SpaceshipTheme,
};

fn main() {
    println!("=== pzsh Theme System ===\n");
    println!("oh-my-zsh compatible themes with O(1) rendering\n");

    // Available themes
    let themes: Vec<Box<dyn Theme>> = vec![
        Box::new(RobbyRussellTheme),
        Box::new(AgnosterTheme),
        Box::new(SimpleTheme),
        Box::new(PureTheme),
        Box::new(SpaceshipTheme),
    ];

    for theme in &themes {
        println!("=== {} ===", theme.name());
        println!();

        // Show zsh prompt
        println!("ZSH Prompt:");
        let zsh_prompt = theme.zsh_prompt();
        for line in zsh_prompt.lines().take(5) {
            println!("  {}", line);
        }
        if zsh_prompt.lines().count() > 5 {
            println!("  ...");
        }
        println!();
    }

    // Theme registry
    println!("=== Theme Registry ===");
    let registry = ThemeRegistry::new();

    println!("Available themes:");
    for name in registry.list() {
        println!("  - {}", name);
    }
    println!();

    // Get theme by name
    if let Some(theme) = registry.get("robbyrussell") {
        println!("Selected theme: {}", theme.name());
        println!("Prompt preview: PROMPT='{}'",
            theme.zsh_prompt().lines().next().unwrap_or(""));
    }
    println!();

    // Theme styles
    println!("=== Theme Style Attributes ===");
    let theme = RobbyRussellTheme;

    println!("User style:     {:?}", theme.user_style().to_ansi());
    println!("Host style:     {:?}", theme.host_style().to_ansi());
    println!("CWD style:      {:?}", theme.cwd_style().to_ansi());
    println!("Git clean:      {:?}", theme.git_clean_style().to_ansi());
    println!("Git dirty:      {:?}", theme.git_dirty_style().to_ansi());
}
