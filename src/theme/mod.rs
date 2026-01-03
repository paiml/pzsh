//! Theme module for pzsh
//!
//! Provides oh-my-zsh-style theming with multiple built-in themes.

use crate::color::{Color, Style};

/// Theme trait for customizing shell appearance
pub trait Theme: Send + Sync {
    /// Theme name
    fn name(&self) -> &str;

    /// User segment style
    fn user_style(&self) -> Style;

    /// Host segment style
    fn host_style(&self) -> Style;

    /// Directory segment style
    fn cwd_style(&self) -> Style;

    /// Git clean branch style
    fn git_clean_style(&self) -> Style;

    /// Git dirty branch style
    fn git_dirty_style(&self) -> Style;

    /// Prompt character style (normal user)
    fn prompt_char_style(&self) -> Style;

    /// Prompt character style (root)
    fn prompt_root_style(&self) -> Style;

    /// Error/command failed style
    fn error_style(&self) -> Style;

    /// Success style
    fn success_style(&self) -> Style;

    /// Generate zsh prompt string
    fn zsh_prompt(&self) -> String;

    /// Generate bash prompt string
    fn bash_prompt(&self) -> String;
}

/// Robbyrussell theme (oh-my-zsh default)
#[derive(Debug, Default)]
pub struct RobbyRussellTheme;

impl Theme for RobbyRussellTheme {
    fn name(&self) -> &str {
        "robbyrussell"
    }

    fn user_style(&self) -> Style {
        Style::new() // Not shown in this theme
    }

    fn host_style(&self) -> Style {
        Style::new() // Not shown in this theme
    }

    fn cwd_style(&self) -> Style {
        Style::new().fg_ansi(Color::Cyan).bold()
    }

    fn git_clean_style(&self) -> Style {
        Style::new().fg_ansi(Color::Blue)
    }

    fn git_dirty_style(&self) -> Style {
        Style::new().fg_ansi(Color::Yellow)
    }

    fn prompt_char_style(&self) -> Style {
        Style::new().fg_ansi(Color::Green).bold()
    }

    fn prompt_root_style(&self) -> Style {
        Style::new().fg_ansi(Color::Red).bold()
    }

    fn error_style(&self) -> Style {
        Style::new().fg_ansi(Color::Red).bold()
    }

    fn success_style(&self) -> Style {
        Style::new().fg_ansi(Color::Green)
    }

    fn zsh_prompt(&self) -> String {
        r#"PROMPT='%(?:%F{green}➜:%F{red}➜) %F{cyan}%c%f $(__pzsh_git_info) '"#.to_string()
    }

    fn bash_prompt(&self) -> String {
        r#"PS1='\[\033[32m\]➜ \[\033[36m\]\W\[\033[0m\] $(__pzsh_git_info) '"#.to_string()
    }
}

/// Agnoster theme (powerline-style)
#[derive(Debug, Default)]
pub struct AgnosterTheme;

impl Theme for AgnosterTheme {
    fn name(&self) -> &str {
        "agnoster"
    }

    fn user_style(&self) -> Style {
        Style::new().fg_ansi(Color::Black).bg_ansi(Color::Blue).bold()
    }

    fn host_style(&self) -> Style {
        Style::new().fg_ansi(Color::Black).bg_ansi(Color::Blue).bold()
    }

    fn cwd_style(&self) -> Style {
        Style::new().fg_ansi(Color::White).bg_ansi(Color::Blue)
    }

    fn git_clean_style(&self) -> Style {
        Style::new().fg_ansi(Color::Black).bg_ansi(Color::Green)
    }

    fn git_dirty_style(&self) -> Style {
        Style::new().fg_ansi(Color::Black).bg_ansi(Color::Yellow)
    }

    fn prompt_char_style(&self) -> Style {
        Style::new()
    }

    fn prompt_root_style(&self) -> Style {
        Style::new().fg_ansi(Color::Yellow)
    }

    fn error_style(&self) -> Style {
        Style::new().fg_ansi(Color::White).bg_ansi(Color::Red)
    }

    fn success_style(&self) -> Style {
        Style::new().fg_ansi(Color::Black).bg_ansi(Color::Green)
    }

    fn zsh_prompt(&self) -> String {
        // Powerline-style with special characters
        r#"PROMPT='%K{blue}%F{black} %n@%m %k%F{blue}%K{cyan}%F{black} %~ %k%F{cyan}$(__pzsh_git_segment)%k%f '"#.to_string()
    }

    fn bash_prompt(&self) -> String {
        r#"PS1='\[\033[44m\]\[\033[30m\] \u@\h \[\033[0m\]\[\033[34m\]\[\033[46m\]\[\033[30m\] \w \[\033[0m\]\[\033[36m\] '"#.to_string()
    }
}

/// Simple/minimal theme
#[derive(Debug, Default)]
pub struct SimpleTheme;

impl Theme for SimpleTheme {
    fn name(&self) -> &str {
        "simple"
    }

    fn user_style(&self) -> Style {
        Style::new().fg_ansi(Color::Green)
    }

    fn host_style(&self) -> Style {
        Style::new().fg_ansi(Color::Blue)
    }

    fn cwd_style(&self) -> Style {
        Style::new().fg_ansi(Color::Cyan)
    }

    fn git_clean_style(&self) -> Style {
        Style::new().fg_ansi(Color::Green)
    }

    fn git_dirty_style(&self) -> Style {
        Style::new().fg_ansi(Color::Yellow)
    }

    fn prompt_char_style(&self) -> Style {
        Style::new().fg_ansi(Color::White)
    }

    fn prompt_root_style(&self) -> Style {
        Style::new().fg_ansi(Color::Red)
    }

    fn error_style(&self) -> Style {
        Style::new().fg_ansi(Color::Red)
    }

    fn success_style(&self) -> Style {
        Style::new().fg_ansi(Color::Green)
    }

    fn zsh_prompt(&self) -> String {
        r#"PROMPT='%F{green}%n%f@%F{blue}%m%f %F{cyan}%~%f $(__pzsh_git_info) %# '"#.to_string()
    }

    fn bash_prompt(&self) -> String {
        r#"PS1='\[\033[32m\]\u\[\033[0m\]@\[\033[34m\]\h\[\033[0m\] \[\033[36m\]\w\[\033[0m\] \$ '"#.to_string()
    }
}

/// Pure theme (async, minimal, fast)
#[derive(Debug, Default)]
pub struct PureTheme;

impl Theme for PureTheme {
    fn name(&self) -> &str {
        "pure"
    }

    fn user_style(&self) -> Style {
        Style::new().fg_ansi(Color::Magenta)
    }

    fn host_style(&self) -> Style {
        Style::new().fg_ansi(Color::Yellow)
    }

    fn cwd_style(&self) -> Style {
        Style::new().fg_ansi(Color::Blue).bold()
    }

    fn git_clean_style(&self) -> Style {
        Style::new().fg_ansi(Color::BrightBlack)
    }

    fn git_dirty_style(&self) -> Style {
        Style::new().fg_ansi(Color::Cyan)
    }

    fn prompt_char_style(&self) -> Style {
        Style::new().fg_ansi(Color::Magenta)
    }

    fn prompt_root_style(&self) -> Style {
        Style::new().fg_ansi(Color::Red)
    }

    fn error_style(&self) -> Style {
        Style::new().fg_ansi(Color::Red)
    }

    fn success_style(&self) -> Style {
        Style::new().fg_ansi(Color::Magenta)
    }

    fn zsh_prompt(&self) -> String {
        // Pure-style two-line prompt
        r#"PROMPT='
%F{blue}%~%f $(__pzsh_git_info)
%(?:%F{magenta}❯:%F{red}❯)%f '"#.to_string()
    }

    fn bash_prompt(&self) -> String {
        r#"PS1='\n\[\033[34m\]\w\[\033[0m\] $(__pzsh_git_info)\n\[\033[35m\]❯\[\033[0m\] '"#.to_string()
    }
}

/// Spaceship theme (feature-rich)
#[derive(Debug, Default)]
pub struct SpaceshipTheme;

impl Theme for SpaceshipTheme {
    fn name(&self) -> &str {
        "spaceship"
    }

    fn user_style(&self) -> Style {
        Style::new().fg_ansi(Color::Yellow)
    }

    fn host_style(&self) -> Style {
        Style::new().fg_ansi(Color::Green)
    }

    fn cwd_style(&self) -> Style {
        Style::new().fg_ansi(Color::Cyan).bold()
    }

    fn git_clean_style(&self) -> Style {
        Style::new().fg_ansi(Color::Green)
    }

    fn git_dirty_style(&self) -> Style {
        Style::new().fg_ansi(Color::Red)
    }

    fn prompt_char_style(&self) -> Style {
        Style::new().fg_ansi(Color::Green)
    }

    fn prompt_root_style(&self) -> Style {
        Style::new().fg_ansi(Color::Red)
    }

    fn error_style(&self) -> Style {
        Style::new().fg_ansi(Color::Red)
    }

    fn success_style(&self) -> Style {
        Style::new().fg_ansi(Color::Green)
    }

    fn zsh_prompt(&self) -> String {
        r#"PROMPT='
%F{cyan}%~%f $(__pzsh_git_info)
%(?:%F{green}❯:%F{red}❯)%f '"#.to_string()
    }

    fn bash_prompt(&self) -> String {
        r#"PS1='\n\[\033[36m\]\w\[\033[0m\] $(__pzsh_git_info)\n\[\033[32m\]❯\[\033[0m\] '"#.to_string()
    }
}

/// Theme registry
pub struct ThemeRegistry {
    themes: ahash::AHashMap<String, Box<dyn Theme>>,
    current: String,
}

impl std::fmt::Debug for ThemeRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThemeRegistry")
            .field("themes", &self.themes.keys().collect::<Vec<_>>())
            .field("current", &self.current)
            .finish()
    }
}

impl ThemeRegistry {
    /// Create new theme registry with built-in themes
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            themes: ahash::AHashMap::new(),
            current: "robbyrussell".to_string(),
        };

        // Register built-in themes
        registry.register(RobbyRussellTheme);
        registry.register(AgnosterTheme);
        registry.register(SimpleTheme);
        registry.register(PureTheme);
        registry.register(SpaceshipTheme);

        registry
    }

    /// Register a theme
    pub fn register(&mut self, theme: impl Theme + 'static) {
        let name = theme.name().to_string();
        self.themes.insert(name, Box::new(theme));
    }

    /// Get current theme
    #[must_use]
    pub fn current(&self) -> Option<&dyn Theme> {
        self.themes.get(&self.current).map(|t| t.as_ref())
    }

    /// Set current theme by name
    pub fn set_current(&mut self, name: &str) -> bool {
        if self.themes.contains_key(name) {
            self.current = name.to_string();
            true
        } else {
            false
        }
    }

    /// Get a theme by name
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn Theme> {
        self.themes.get(name).map(|t| t.as_ref())
    }

    /// List all available themes
    #[must_use]
    pub fn list(&self) -> Vec<&str> {
        self.themes.keys().map(|s| s.as_str()).collect()
    }

    /// Get theme count
    #[must_use]
    pub fn count(&self) -> usize {
        self.themes.len()
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== THEME TRAIT TESTS ====================

    #[test]
    fn test_robbyrussell_theme() {
        let theme = RobbyRussellTheme;
        assert_eq!(theme.name(), "robbyrussell");

        let prompt = theme.zsh_prompt();
        assert!(prompt.contains("PROMPT="));
        assert!(prompt.contains("cyan"));
        assert!(prompt.contains("➜"));
    }

    #[test]
    fn test_agnoster_theme() {
        let theme = AgnosterTheme;
        assert_eq!(theme.name(), "agnoster");

        let prompt = theme.zsh_prompt();
        assert!(prompt.contains("%K{blue}")); // Background color
    }

    #[test]
    fn test_simple_theme() {
        let theme = SimpleTheme;
        assert_eq!(theme.name(), "simple");

        let prompt = theme.zsh_prompt();
        assert!(prompt.contains("%n")); // Username
        assert!(prompt.contains("%m")); // Hostname
    }

    #[test]
    fn test_pure_theme() {
        let theme = PureTheme;
        assert_eq!(theme.name(), "pure");

        let prompt = theme.zsh_prompt();
        assert!(prompt.contains("❯"));
        assert!(prompt.contains('\n')); // Two-line prompt
    }

    #[test]
    fn test_spaceship_theme() {
        let theme = SpaceshipTheme;
        assert_eq!(theme.name(), "spaceship");

        let prompt = theme.zsh_prompt();
        assert!(prompt.contains("❯"));
    }

    // ==================== THEME REGISTRY TESTS ====================

    #[test]
    fn test_registry_new() {
        let registry = ThemeRegistry::new();
        assert!(registry.count() >= 5); // At least 5 built-in themes
    }

    #[test]
    fn test_registry_list_themes() {
        let registry = ThemeRegistry::new();
        let themes = registry.list();

        assert!(themes.contains(&"robbyrussell"));
        assert!(themes.contains(&"agnoster"));
        assert!(themes.contains(&"simple"));
        assert!(themes.contains(&"pure"));
        assert!(themes.contains(&"spaceship"));
    }

    #[test]
    fn test_registry_get_theme() {
        let registry = ThemeRegistry::new();

        let theme = registry.get("robbyrussell").unwrap();
        assert_eq!(theme.name(), "robbyrussell");
    }

    #[test]
    fn test_registry_get_unknown() {
        let registry = ThemeRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_current_theme() {
        let registry = ThemeRegistry::new();
        let current = registry.current().unwrap();
        assert_eq!(current.name(), "robbyrussell"); // Default
    }

    #[test]
    fn test_registry_set_current() {
        let mut registry = ThemeRegistry::new();

        assert!(registry.set_current("pure"));
        assert_eq!(registry.current().unwrap().name(), "pure");
    }

    #[test]
    fn test_registry_set_current_unknown() {
        let mut registry = ThemeRegistry::new();
        assert!(!registry.set_current("nonexistent"));
    }

    // ==================== STYLE TESTS ====================

    #[test]
    fn test_theme_styles() {
        let theme = SimpleTheme;

        let user_style = theme.user_style();
        assert!(user_style.fg.is_some());

        let error_style = theme.error_style();
        assert!(error_style.fg.is_some());
    }

    #[test]
    fn test_all_themes_have_prompts() {
        let registry = ThemeRegistry::new();

        for name in registry.list() {
            let theme = registry.get(name).unwrap();
            let zsh = theme.zsh_prompt();
            let bash = theme.bash_prompt();

            assert!(!zsh.is_empty(), "Theme {name} has empty zsh prompt");
            assert!(!bash.is_empty(), "Theme {name} has empty bash prompt");
        }
    }

    // ==================== PERFORMANCE TESTS ====================

    #[test]
    fn test_registry_lookup_fast() {
        let registry = ThemeRegistry::new();

        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = registry.get("robbyrussell");
        }
        let elapsed = start.elapsed();

        // Allow 100ms for coverage builds which have instrumentation overhead
        assert!(
            elapsed < std::time::Duration::from_millis(100),
            "Registry lookup too slow: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_prompt_generation_fast() {
        let theme = SpaceshipTheme;

        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = theme.zsh_prompt();
        }
        let elapsed = start.elapsed();

        assert!(
            elapsed < std::time::Duration::from_millis(10),
            "Prompt generation too slow: {:?}",
            elapsed
        );
    }

    // ==================== CUSTOM THEME TEST ====================

    struct CustomTheme;

    impl Theme for CustomTheme {
        fn name(&self) -> &str {
            "custom"
        }

        fn user_style(&self) -> Style {
            Style::new().fg_ansi(Color::Magenta)
        }

        fn host_style(&self) -> Style {
            Style::new().fg_ansi(Color::Cyan)
        }

        fn cwd_style(&self) -> Style {
            Style::new().fg_ansi(Color::Yellow)
        }

        fn git_clean_style(&self) -> Style {
            Style::new().fg_ansi(Color::Green)
        }

        fn git_dirty_style(&self) -> Style {
            Style::new().fg_ansi(Color::Red)
        }

        fn prompt_char_style(&self) -> Style {
            Style::new().fg_ansi(Color::White)
        }

        fn prompt_root_style(&self) -> Style {
            Style::new().fg_ansi(Color::Red)
        }

        fn error_style(&self) -> Style {
            Style::new().fg_ansi(Color::Red)
        }

        fn success_style(&self) -> Style {
            Style::new().fg_ansi(Color::Green)
        }

        fn zsh_prompt(&self) -> String {
            "PROMPT='custom> '".to_string()
        }

        fn bash_prompt(&self) -> String {
            "PS1='custom> '".to_string()
        }
    }

    #[test]
    fn test_custom_theme_registration() {
        let mut registry = ThemeRegistry::new();
        registry.register(CustomTheme);

        assert!(registry.get("custom").is_some());
        assert_eq!(registry.get("custom").unwrap().name(), "custom");
    }

    // ==================== BASH PROMPT TESTS ====================

    #[test]
    fn test_robbyrussell_bash_prompt() {
        let theme = RobbyRussellTheme;
        let prompt = theme.bash_prompt();
        assert!(prompt.contains("PS1="));
        assert!(prompt.contains("\\033["));
    }

    #[test]
    fn test_agnoster_bash_prompt() {
        let theme = AgnosterTheme;
        let prompt = theme.bash_prompt();
        assert!(prompt.contains("PS1="));
    }

    #[test]
    fn test_pure_bash_prompt() {
        let theme = PureTheme;
        let prompt = theme.bash_prompt();
        assert!(prompt.contains("\\n")); // Two-line prompt
    }

    // ==================== STYLE COVERAGE TESTS ====================

    #[test]
    fn test_robbyrussell_all_styles() {
        let theme = RobbyRussellTheme;
        let _ = theme.user_style();
        let _ = theme.host_style();
        let _ = theme.cwd_style();
        let _ = theme.git_clean_style();
        let _ = theme.git_dirty_style();
        let _ = theme.prompt_char_style();
        let _ = theme.prompt_root_style();
        let _ = theme.error_style();
        let _ = theme.success_style();
    }

    #[test]
    fn test_agnoster_all_styles() {
        let theme = AgnosterTheme;
        let user = theme.user_style();
        assert!(user.bg.is_some()); // Agnoster uses backgrounds
        let _ = theme.host_style();
        let _ = theme.cwd_style();
        let _ = theme.git_clean_style();
        let _ = theme.git_dirty_style();
        let _ = theme.prompt_char_style();
        let _ = theme.prompt_root_style();
        let _ = theme.error_style();
        let _ = theme.success_style();
    }

    #[test]
    fn test_pure_all_styles() {
        let theme = PureTheme;
        let _ = theme.user_style();
        let _ = theme.host_style();
        let _ = theme.cwd_style();
        let _ = theme.git_clean_style();
        let _ = theme.git_dirty_style();
        let _ = theme.prompt_char_style();
        let _ = theme.prompt_root_style();
        let _ = theme.error_style();
        let _ = theme.success_style();
    }

    #[test]
    fn test_spaceship_all_styles() {
        let theme = SpaceshipTheme;
        let _ = theme.user_style();
        let _ = theme.host_style();
        let _ = theme.cwd_style();
        let _ = theme.git_clean_style();
        let _ = theme.git_dirty_style();
        let _ = theme.prompt_char_style();
        let _ = theme.prompt_root_style();
        let _ = theme.error_style();
        let _ = theme.success_style();
    }

    #[test]
    fn test_simple_all_styles() {
        let theme = SimpleTheme;
        let _ = theme.user_style();
        let _ = theme.host_style();
        let _ = theme.cwd_style();
        let _ = theme.git_clean_style();
        let _ = theme.git_dirty_style();
        let _ = theme.prompt_char_style();
        let _ = theme.prompt_root_style();
        let _ = theme.error_style();
        let _ = theme.success_style();
    }

    #[test]
    fn test_registry_debug() {
        let registry = ThemeRegistry::new();
        let debug = format!("{:?}", registry);
        assert!(debug.contains("ThemeRegistry"));
        assert!(debug.contains("current"));
    }

    #[test]
    fn test_registry_default() {
        let registry = ThemeRegistry::default();
        assert!(registry.count() >= 5);
    }
}
