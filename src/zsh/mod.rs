//! Zsh-specific integration for pzsh
//!
//! Provides zsh completion definitions, widgets, and advanced features.

use ahash::AHashMap;

/// Zsh completion generator
///
/// Generates zsh completion definitions using compdef/compadd.
#[derive(Debug, Default)]
pub struct ZshCompletion {
    /// Custom completions for specific commands
    command_completions: AHashMap<String, Vec<CompletionSpec>>,
}

/// Completion specification for a command
#[derive(Debug, Clone)]
pub struct CompletionSpec {
    /// Pattern to match (e.g., "-*" for flags)
    pub pattern: String,
    /// Description
    pub description: String,
    /// Completion values
    pub values: Vec<String>,
    /// Is a flag/option
    pub is_flag: bool,
}

impl CompletionSpec {
    /// Create a flag completion
    #[must_use]
    pub fn flag(name: &str, description: &str) -> Self {
        Self {
            pattern: name.to_string(),
            description: description.to_string(),
            values: vec![],
            is_flag: true,
        }
    }

    /// Create a value completion
    #[must_use]
    pub fn value(pattern: &str, values: Vec<String>) -> Self {
        Self {
            pattern: pattern.to_string(),
            description: String::new(),
            values,
            is_flag: false,
        }
    }
}

impl ZshCompletion {
    /// Create new zsh completion generator
    #[must_use]
    pub fn new() -> Self {
        let mut zc = Self {
            command_completions: AHashMap::new(),
        };

        // Register built-in completions
        zc.register_git_completions();
        zc.register_docker_completions();

        zc
    }

    /// Register completions for a command
    pub fn register(&mut self, command: &str, specs: Vec<CompletionSpec>) {
        self.command_completions.insert(command.to_string(), specs);
    }

    /// Generate zsh completion function for a command
    #[must_use]
    pub fn generate_completion_function(&self, command: &str) -> Option<String> {
        let specs = self.command_completions.get(command)?;

        let mut output = String::new();
        output.push_str(&format!("#compdef {command}\n\n"));
        output.push_str(&format!("_{command}() {{\n"));
        output.push_str("  local curcontext=\"$curcontext\" state line\n");
        output.push_str("  typeset -A opt_args\n\n");

        // Generate _arguments call
        output.push_str("  _arguments -C \\\n");

        for (i, spec) in specs.iter().enumerate() {
            let comma = if i < specs.len() - 1 { " \\" } else { "" };
            if spec.is_flag {
                output.push_str(&format!(
                    "    '{}[{}]'{}\n",
                    spec.pattern, spec.description, comma
                ));
            } else if !spec.values.is_empty() {
                let values = spec.values.join(" ");
                output.push_str(&format!("    '*:{}:(({values}))'{comma}\n", spec.pattern));
            }
        }

        output.push_str("}\n\n");
        output.push_str(&format!("_{command} \"$@\"\n"));

        Some(output)
    }

    /// Generate all completion functions
    #[must_use]
    pub fn generate_all(&self) -> String {
        let mut output = String::new();
        output.push_str("# pzsh zsh completions\n\n");

        for command in self.command_completions.keys() {
            if let Some(func) = self.generate_completion_function(command) {
                output.push_str(&func);
                output.push('\n');
            }
        }

        output
    }

    /// Register git completions
    fn register_git_completions(&mut self) {
        let specs = vec![
            CompletionSpec::flag("-v", "Show version"),
            CompletionSpec::flag("--help", "Show help"),
            CompletionSpec::value(
                "command",
                vec![
                    "add".to_string(),
                    "branch".to_string(),
                    "checkout".to_string(),
                    "clone".to_string(),
                    "commit".to_string(),
                    "diff".to_string(),
                    "fetch".to_string(),
                    "init".to_string(),
                    "log".to_string(),
                    "merge".to_string(),
                    "pull".to_string(),
                    "push".to_string(),
                    "rebase".to_string(),
                    "reset".to_string(),
                    "restore".to_string(),
                    "stash".to_string(),
                    "status".to_string(),
                    "switch".to_string(),
                    "tag".to_string(),
                ],
            ),
        ];
        self.register("git", specs);
    }

    /// Register docker completions
    fn register_docker_completions(&mut self) {
        let specs = vec![
            CompletionSpec::flag("-v", "Show version"),
            CompletionSpec::flag("--help", "Show help"),
            CompletionSpec::value(
                "command",
                vec![
                    "build".to_string(),
                    "compose".to_string(),
                    "container".to_string(),
                    "exec".to_string(),
                    "image".to_string(),
                    "images".to_string(),
                    "logs".to_string(),
                    "network".to_string(),
                    "ps".to_string(),
                    "pull".to_string(),
                    "push".to_string(),
                    "rm".to_string(),
                    "rmi".to_string(),
                    "run".to_string(),
                    "start".to_string(),
                    "stop".to_string(),
                    "volume".to_string(),
                ],
            ),
        ];
        self.register("docker", specs);
    }
}

/// Zsh widget for auto-suggestions
#[derive(Debug)]
pub struct AutoSuggestWidget {
    /// History entries for suggestions
    history: Vec<String>,
    /// Current suggestion
    current_suggestion: Option<String>,
}

impl AutoSuggestWidget {
    /// Create new auto-suggest widget
    #[must_use]
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current_suggestion: None,
        }
    }

    /// Load history from file
    pub fn load_history(&mut self, entries: Vec<String>) {
        self.history = entries;
    }

    /// Get suggestion for current input
    #[must_use]
    pub fn suggest(&mut self, input: &str) -> Option<&str> {
        if input.is_empty() {
            self.current_suggestion = None;
            return None;
        }

        // Find most recent matching history entry
        for entry in self.history.iter().rev() {
            if entry.starts_with(input) && entry != input {
                self.current_suggestion = Some(entry.clone());
                return self.current_suggestion.as_deref();
            }
        }

        self.current_suggestion = None;
        None
    }

    /// Generate zsh widget code
    #[must_use]
    pub fn generate_widget_code() -> String {
        r#"# pzsh auto-suggestions widget
# Similar to zsh-autosuggestions

# Suggestion color (gray)
typeset -g PZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=8'

# Auto-suggest from history
_pzsh_autosuggest() {
    local suggestion
    suggestion=$(fc -ln -1000 | grep -m1 "^${BUFFER}")

    if [[ -n "$suggestion" && "$suggestion" != "$BUFFER" ]]; then
        local postfix="${suggestion#$BUFFER}"
        POSTDISPLAY="$postfix"
        region_highlight=("${#BUFFER} $((${#BUFFER} + ${#postfix})) $PZSH_AUTOSUGGEST_HIGHLIGHT_STYLE")
    else
        POSTDISPLAY=""
    fi
}

# Accept suggestion
_pzsh_autosuggest_accept() {
    if [[ -n "$POSTDISPLAY" ]]; then
        BUFFER="$BUFFER$POSTDISPLAY"
        CURSOR=${#BUFFER}
        POSTDISPLAY=""
    fi
    zle redisplay
}

# Clear suggestion
_pzsh_autosuggest_clear() {
    POSTDISPLAY=""
    zle redisplay
}

# Register widgets
zle -N _pzsh_autosuggest
zle -N _pzsh_autosuggest_accept
zle -N _pzsh_autosuggest_clear

# Hook into line editing
autoload -Uz add-zle-hook-widget
add-zle-hook-widget line-pre-redraw _pzsh_autosuggest

# Key bindings
bindkey '^[[C' _pzsh_autosuggest_accept  # Right arrow
bindkey '^ ' _pzsh_autosuggest_accept     # Ctrl+Space
"#.to_string()
    }
}

impl Default for AutoSuggestWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Zsh syntax highlighting generator
#[derive(Debug)]
pub struct SyntaxHighlighter {
    /// Command color
    pub command_color: String,
    /// Alias color
    pub alias_color: String,
    /// Builtin color
    pub builtin_color: String,
    /// Error color (unknown command)
    pub error_color: String,
    /// Path color
    pub path_color: String,
    /// String color
    pub string_color: String,
    /// Comment color
    pub comment_color: String,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self {
            command_color: "fg=green,bold".to_string(),
            alias_color: "fg=cyan,bold".to_string(),
            builtin_color: "fg=yellow,bold".to_string(),
            error_color: "fg=red,bold".to_string(),
            path_color: "fg=blue,underline".to_string(),
            string_color: "fg=yellow".to_string(),
            comment_color: "fg=8".to_string(), // Gray
        }
    }
}

impl SyntaxHighlighter {
    /// Create new syntax highlighter
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate zsh syntax highlighting code
    #[must_use]
    pub fn generate_highlight_code(&self) -> String {
        format!(
            r#"# pzsh syntax highlighting
# Similar to zsh-syntax-highlighting (simplified)

typeset -gA PZSH_HIGHLIGHT_STYLES
PZSH_HIGHLIGHT_STYLES[command]='{}'
PZSH_HIGHLIGHT_STYLES[alias]='{}'
PZSH_HIGHLIGHT_STYLES[builtin]='{}'
PZSH_HIGHLIGHT_STYLES[unknown]='{}'
PZSH_HIGHLIGHT_STYLES[path]='{}'
PZSH_HIGHLIGHT_STYLES[single-quoted-argument]='{}'
PZSH_HIGHLIGHT_STYLES[double-quoted-argument]='{}'
PZSH_HIGHLIGHT_STYLES[comment]='{}'

# Highlight function
_pzsh_highlight() {{
    region_highlight=()
    local word start=0
    local -a words
    words=(${{(z)BUFFER}})

    [[ -z "${{words[1]}}" ]] && return

    local cmd="${{words[1]}}"
    local style

    if (( $+commands[$cmd] )); then
        style=$PZSH_HIGHLIGHT_STYLES[command]
    elif (( $+aliases[$cmd] )); then
        style=$PZSH_HIGHLIGHT_STYLES[alias]
    elif (( $+builtins[$cmd] )); then
        style=$PZSH_HIGHLIGHT_STYLES[builtin]
    else
        style=$PZSH_HIGHLIGHT_STYLES[unknown]
    fi

    local end=${{#cmd}}
    region_highlight+=("0 $end $style")
}}

# Hook into line editing
autoload -Uz add-zle-hook-widget
add-zle-hook-widget line-pre-redraw _pzsh_highlight
"#,
            self.command_color,
            self.alias_color,
            self.builtin_color,
            self.error_color,
            self.path_color,
            self.string_color,
            self.string_color,
            self.comment_color
        )
    }
}

/// History substring search widget
#[derive(Debug, Default)]
pub struct HistorySearch;

impl HistorySearch {
    /// Generate zsh history search widget code
    #[must_use]
    pub fn generate_widget_code() -> String {
        r#"# pzsh history substring search
# Similar to zsh-history-substring-search

typeset -g PZSH_HISTORY_SUBSTRING_SEARCH_HIGHLIGHT_FOUND='bg=magenta,fg=white,bold'
typeset -g PZSH_HISTORY_SUBSTRING_SEARCH_HIGHLIGHT_NOT_FOUND='bg=red,fg=white,bold'

_pzsh_history_search_up() {
    local search_term="$BUFFER"
    local -a matches

    # Find matches
    matches=(${(f)"$(fc -ln -1000 | grep -F "$search_term" | tac)"})

    if [[ ${#matches} -gt 0 ]]; then
        BUFFER="${matches[1]}"
        CURSOR=${#BUFFER}
        region_highlight=("0 ${#BUFFER} $PZSH_HISTORY_SUBSTRING_SEARCH_HIGHLIGHT_FOUND")
    else
        region_highlight=("0 ${#BUFFER} $PZSH_HISTORY_SUBSTRING_SEARCH_HIGHLIGHT_NOT_FOUND")
    fi
}

_pzsh_history_search_down() {
    # Similar but search forward
    _pzsh_history_search_up
}

zle -N _pzsh_history_search_up
zle -N _pzsh_history_search_down

bindkey '^[[A' _pzsh_history_search_up    # Up arrow
bindkey '^[[B' _pzsh_history_search_down  # Down arrow
"#
        .to_string()
    }
}

/// Directory jump (z-like) functionality
#[derive(Debug, Default)]
pub struct DirectoryJump {
    /// Frecency database (path -> score)
    frecency: AHashMap<String, f64>,
}

impl DirectoryJump {
    /// Create new directory jump
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a directory visit
    pub fn record(&mut self, path: &str) {
        let entry = self.frecency.entry(path.to_string()).or_insert(0.0);
        *entry += 1.0;
    }

    /// Find best match for query
    #[must_use]
    pub fn find(&self, query: &str) -> Option<&str> {
        let query_lower = query.to_lowercase();

        self.frecency
            .iter()
            .filter(|(path, _)| {
                let path_lower = path.to_lowercase();
                path_lower.contains(&query_lower)
            })
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(path, _)| path.as_str())
    }

    /// Generate zsh z command code
    #[must_use]
    pub fn generate_z_command() -> String {
        r#"# pzsh directory jump (z-like)
# Similar to z, autojump, zoxide

typeset -g PZSH_Z_DATA="${XDG_DATA_HOME:-$HOME/.local/share}/pzsh/z_data"

# Ensure data directory exists
[[ -d "${PZSH_Z_DATA%/*}" ]] || mkdir -p "${PZSH_Z_DATA%/*}"

# Record directory on chpwd
_pzsh_z_record() {
    local pwd="${PWD:A}"
    [[ "$pwd" == "$HOME" ]] && return

    # Append to data file
    echo "$pwd|$(date +%s)" >> "$PZSH_Z_DATA"
}

# Jump to directory
z() {
    local query="$1"

    if [[ -z "$query" ]]; then
        cd ~ && return
    fi

    local match
    match=$(awk -F'|' -v q="$query" '
        tolower($1) ~ tolower(q) { print $1; exit }
    ' "$PZSH_Z_DATA" 2>/dev/null)

    if [[ -n "$match" && -d "$match" ]]; then
        cd "$match"
    else
        echo "z: no match for: $query" >&2
        return 1
    fi
}

# Hook into chpwd
autoload -Uz add-zsh-hook
add-zsh-hook chpwd _pzsh_z_record
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== COMPLETION TESTS ====================

    #[test]
    fn test_zsh_completion_new() {
        let zc = ZshCompletion::new();
        assert!(zc.command_completions.contains_key("git"));
        assert!(zc.command_completions.contains_key("docker"));
    }

    #[test]
    fn test_completion_spec_flag() {
        let spec = CompletionSpec::flag("-v", "verbose");
        assert!(spec.is_flag);
        assert_eq!(spec.pattern, "-v");
        assert_eq!(spec.description, "verbose");
    }

    #[test]
    fn test_generate_git_completion() {
        let zc = ZshCompletion::new();
        let output = zc.generate_completion_function("git").unwrap();

        assert!(output.contains("#compdef git"));
        assert!(output.contains("_git()"));
        assert!(output.contains("_arguments"));
        assert!(output.contains("commit"));
        assert!(output.contains("push"));
    }

    #[test]
    fn test_generate_docker_completion() {
        let zc = ZshCompletion::new();
        let output = zc.generate_completion_function("docker").unwrap();

        assert!(output.contains("#compdef docker"));
        assert!(output.contains("ps"));
        assert!(output.contains("build"));
    }

    #[test]
    fn test_generate_all_completions() {
        let zc = ZshCompletion::new();
        let output = zc.generate_all();

        assert!(output.contains("#compdef git"));
        assert!(output.contains("#compdef docker"));
    }

    #[test]
    fn test_unknown_command_completion() {
        let zc = ZshCompletion::new();
        let output = zc.generate_completion_function("nonexistent");
        assert!(output.is_none());
    }

    // ==================== AUTO-SUGGEST TESTS ====================

    #[test]
    fn test_auto_suggest_new() {
        let widget = AutoSuggestWidget::new();
        assert!(widget.history.is_empty());
        assert!(widget.current_suggestion.is_none());
    }

    #[test]
    fn test_auto_suggest_with_history() {
        let mut widget = AutoSuggestWidget::new();
        widget.load_history(vec![
            "git status".to_string(),
            "git push".to_string(),
            "git commit -m 'test'".to_string(),
        ]);

        // Should suggest most recent match
        let suggestion = widget.suggest("git c");
        assert_eq!(suggestion, Some("git commit -m 'test'"));
    }

    #[test]
    fn test_auto_suggest_no_match() {
        let mut widget = AutoSuggestWidget::new();
        widget.load_history(vec!["git status".to_string()]);

        let suggestion = widget.suggest("docker");
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_auto_suggest_empty_input() {
        let mut widget = AutoSuggestWidget::new();
        widget.load_history(vec!["git status".to_string()]);

        let suggestion = widget.suggest("");
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_auto_suggest_exact_match_no_suggest() {
        let mut widget = AutoSuggestWidget::new();
        widget.load_history(vec!["git status".to_string()]);

        // Exact match should not suggest itself
        let suggestion = widget.suggest("git status");
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_auto_suggest_widget_code() {
        let code = AutoSuggestWidget::generate_widget_code();

        assert!(code.contains("_pzsh_autosuggest"));
        assert!(code.contains("_pzsh_autosuggest_accept"));
        assert!(code.contains("zle -N"));
        assert!(code.contains("bindkey"));
    }

    // ==================== SYNTAX HIGHLIGHT TESTS ====================

    #[test]
    fn test_syntax_highlighter_default() {
        let hl = SyntaxHighlighter::default();
        assert!(hl.command_color.contains("green"));
        assert!(hl.error_color.contains("red"));
    }

    #[test]
    fn test_syntax_highlight_code() {
        let hl = SyntaxHighlighter::new();
        let code = hl.generate_highlight_code();

        assert!(code.contains("PZSH_HIGHLIGHT_STYLES"));
        assert!(code.contains("[command]"));
        assert!(code.contains("[alias]"));
        assert!(code.contains("_pzsh_highlight"));
        assert!(code.contains("region_highlight"));
    }

    // ==================== HISTORY SEARCH TESTS ====================

    #[test]
    fn test_history_search_widget_code() {
        let code = HistorySearch::generate_widget_code();

        assert!(code.contains("_pzsh_history_search_up"));
        assert!(code.contains("_pzsh_history_search_down"));
        assert!(code.contains("HIGHLIGHT_FOUND"));
        assert!(code.contains("bindkey"));
    }

    // ==================== DIRECTORY JUMP TESTS ====================

    #[test]
    fn test_directory_jump_new() {
        let dj = DirectoryJump::new();
        assert!(dj.frecency.is_empty());
    }

    #[test]
    fn test_directory_jump_record() {
        let mut dj = DirectoryJump::new();
        dj.record("/home/user/projects");
        dj.record("/home/user/projects");
        dj.record("/tmp");

        assert_eq!(dj.frecency.get("/home/user/projects"), Some(&2.0));
        assert_eq!(dj.frecency.get("/tmp"), Some(&1.0));
    }

    #[test]
    fn test_directory_jump_find() {
        let mut dj = DirectoryJump::new();
        dj.record("/home/user/projects");
        dj.record("/home/user/documents");

        let result = dj.find("proj");
        assert_eq!(result, Some("/home/user/projects"));
    }

    #[test]
    fn test_directory_jump_find_case_insensitive() {
        let mut dj = DirectoryJump::new();
        dj.record("/home/user/Projects");

        let result = dj.find("projects");
        assert_eq!(result, Some("/home/user/Projects"));
    }

    #[test]
    fn test_directory_jump_find_no_match() {
        let dj = DirectoryJump::new();
        let result = dj.find("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_directory_jump_z_command() {
        let code = DirectoryJump::generate_z_command();

        assert!(code.contains("z()"));
        assert!(code.contains("PZSH_Z_DATA"));
        assert!(code.contains("_pzsh_z_record"));
        assert!(code.contains("add-zsh-hook"));
    }

    // ==================== PERFORMANCE TESTS ====================

    #[test]
    fn test_completion_generation_fast() {
        let zc = ZshCompletion::new();

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = zc.generate_all();
        }
        let elapsed = start.elapsed();

        assert!(
            elapsed < std::time::Duration::from_millis(100),
            "Completion generation too slow: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_auto_suggest_fast() {
        let mut widget = AutoSuggestWidget::new();

        // Load 1000 history entries
        let history: Vec<String> = (0..1000)
            .map(|i| format!("command-{i} --arg value"))
            .collect();
        widget.load_history(history);

        let start = std::time::Instant::now();
        for i in 0..1000 {
            let _ = widget.suggest(&format!("command-{i}"));
        }
        let elapsed = start.elapsed();

        assert!(
            elapsed < std::time::Duration::from_millis(100),
            "Auto-suggest too slow: {:?}",
            elapsed
        );
    }
}
