//! Plugin module for pzsh
//!
//! Lightweight plugin system inspired by oh-my-zsh.
//! Plugins are loaded lazily to maintain O(1) startup.

use ahash::AHashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Plugin loading budget (per plugin)
pub const PLUGIN_BUDGET_MS: u64 = 5;

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin registered but not loaded
    Registered,
    /// Plugin is being loaded
    Loading,
    /// Plugin loaded successfully
    Loaded,
    /// Plugin failed to load
    Failed,
    /// Plugin disabled by user
    Disabled,
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin description
    pub description: String,
    /// Plugin version
    pub version: String,
    /// Dependencies (other plugin names)
    pub dependencies: Vec<String>,
    /// Whether plugin supports lazy loading
    pub lazy_loadable: bool,
}

impl PluginInfo {
    /// Create plugin info
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            version: "1.0.0".to_string(),
            dependencies: Vec::new(),
            lazy_loadable: true,
        }
    }

    /// Set description
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set version
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Add dependency
    #[must_use]
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }
}

/// Plugin definition trait
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn info(&self) -> PluginInfo;

    /// Initialize plugin (called once on load)
    fn init(&mut self) -> Result<(), PluginError>;

    /// Generate shell initialization code
    fn shell_init(&self, shell: crate::ShellType) -> String;

    /// Get aliases provided by this plugin
    fn aliases(&self) -> AHashMap<String, String> {
        AHashMap::new()
    }

    /// Get environment variables set by this plugin
    fn env_vars(&self) -> AHashMap<String, String> {
        AHashMap::new()
    }

    /// Get completions provided by this plugin
    fn completions(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Plugin errors
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin not found: {0}")]
    NotFound(String),

    #[error("plugin load failed: {0}")]
    LoadFailed(String),

    #[error("plugin budget exceeded: {0}ms")]
    BudgetExceeded(u64),

    #[error("dependency not met: {0}")]
    DependencyNotMet(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Built-in git plugin
pub struct GitPlugin {
    enabled: bool,
}

impl GitPlugin {
    /// Create git plugin
    #[must_use]
    pub const fn new() -> Self {
        Self { enabled: false }
    }
}

impl Default for GitPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for GitPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("git")
            .with_description("Git aliases and integration")
            .with_version("1.0.0")
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn shell_init(&self, shell: crate::ShellType) -> String {
        if !self.enabled {
            return String::new();
        }

        match shell {
            crate::ShellType::Zsh => {
                // Zsh-specific git integration
                r"
# pzsh git plugin
autoload -Uz vcs_info
precmd_functions+=( vcs_info )
zstyle ':vcs_info:*' enable git
zstyle ':vcs_info:git:*' formats '%b'
"
                .to_string()
            }
            crate::ShellType::Bash => {
                // Bash-specific git integration
                r"
# pzsh git plugin
__pzsh_git_branch() {
    git branch 2>/dev/null | grep '^\*' | sed 's/^\* //'
}
"
                .to_string()
            }
        }
    }

    fn aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        aliases.insert("g".to_string(), "git".to_string());
        aliases.insert("ga".to_string(), "git add".to_string());
        aliases.insert("gaa".to_string(), "git add --all".to_string());
        aliases.insert("gb".to_string(), "git branch".to_string());
        aliases.insert("gc".to_string(), "git commit".to_string());
        aliases.insert("gcm".to_string(), "git commit -m".to_string());
        aliases.insert("gco".to_string(), "git checkout".to_string());
        aliases.insert("gd".to_string(), "git diff".to_string());
        aliases.insert("gf".to_string(), "git fetch".to_string());
        aliases.insert("gl".to_string(), "git pull".to_string());
        aliases.insert("gp".to_string(), "git push".to_string());
        aliases.insert("gs".to_string(), "git status".to_string());
        aliases.insert("gst".to_string(), "git stash".to_string());
        aliases.insert("glog".to_string(), "git log --oneline --graph".to_string());
        aliases
    }
}

/// Built-in docker plugin
pub struct DockerPlugin {
    enabled: bool,
}

impl DockerPlugin {
    /// Create docker plugin
    #[must_use]
    pub const fn new() -> Self {
        Self { enabled: false }
    }
}

impl Default for DockerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for DockerPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("docker")
            .with_description("Docker aliases and completions")
            .with_version("1.0.0")
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn shell_init(&self, _shell: crate::ShellType) -> String {
        String::new() // Docker doesn't need shell init
    }

    fn aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        aliases.insert("d".to_string(), "docker".to_string());
        aliases.insert("dc".to_string(), "docker compose".to_string());
        aliases.insert("dcu".to_string(), "docker compose up".to_string());
        aliases.insert("dcd".to_string(), "docker compose down".to_string());
        aliases.insert("dps".to_string(), "docker ps".to_string());
        aliases.insert("di".to_string(), "docker images".to_string());
        aliases.insert("drm".to_string(), "docker rm".to_string());
        aliases.insert("drmi".to_string(), "docker rmi".to_string());
        aliases.insert("dex".to_string(), "docker exec -it".to_string());
        aliases
    }
}

/// Plugin registry and loader
pub struct PluginManager {
    /// Registered plugins
    plugins: AHashMap<String, Box<dyn Plugin>>,
    /// Plugin states
    states: AHashMap<String, PluginState>,
    /// Plugin load order
    load_order: Vec<String>,
    /// Plugin directory
    plugin_dir: Option<PathBuf>,
}

impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("plugins", &self.plugins.keys().collect::<Vec<_>>())
            .field("states", &self.states)
            .field("load_order", &self.load_order)
            .field("plugin_dir", &self.plugin_dir)
            .finish()
    }
}

impl PluginManager {
    /// Create new plugin manager
    #[must_use]
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: AHashMap::new(),
            states: AHashMap::new(),
            load_order: Vec::new(),
            plugin_dir: None,
        };

        // Register built-in plugins
        manager.register(GitPlugin::new());
        manager.register(DockerPlugin::new());

        manager
    }

    /// Set plugin directory for external plugins
    pub fn set_plugin_dir(&mut self, dir: PathBuf) {
        self.plugin_dir = Some(dir);
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: impl Plugin + 'static) {
        let info = plugin.info();
        let name = info.name.clone();
        self.plugins.insert(name.clone(), Box::new(plugin));
        self.states.insert(name, PluginState::Registered);
    }

    /// Load a plugin by name
    pub fn load(&mut self, name: &str) -> Result<Duration, PluginError> {
        let start = Instant::now();

        // Check if already loaded
        if self.states.get(name) == Some(&PluginState::Loaded) {
            return Ok(Duration::ZERO);
        }

        // Get plugin
        let plugin = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        // Check dependencies
        let deps = plugin.info().dependencies.clone();
        for dep in &deps {
            if !matches!(self.states.get(dep), Some(PluginState::Loaded)) {
                return Err(PluginError::DependencyNotMet(dep.clone()));
            }
        }

        // Mark as loading
        self.states.insert(name.to_string(), PluginState::Loading);

        // Initialize plugin
        plugin.init().inspect_err(|_| {
            self.states.insert(name.to_string(), PluginState::Failed);
        })?;

        let elapsed = start.elapsed();

        // Check budget
        if elapsed > Duration::from_millis(PLUGIN_BUDGET_MS) {
            self.states.insert(name.to_string(), PluginState::Failed);
            return Err(PluginError::BudgetExceeded(elapsed.as_millis() as u64));
        }

        // Mark as loaded
        self.states.insert(name.to_string(), PluginState::Loaded);
        self.load_order.push(name.to_string());

        Ok(elapsed)
    }

    /// Load multiple plugins
    pub fn load_all(&mut self, names: &[String]) -> Vec<Result<Duration, PluginError>> {
        names.iter().map(|name| self.load(name)).collect()
    }

    /// Get all aliases from loaded plugins
    #[must_use]
    pub fn all_aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        for name in &self.load_order {
            if let Some(plugin) = self.plugins.get(name)
                && matches!(self.states.get(name), Some(PluginState::Loaded))
            {
                aliases.extend(plugin.aliases());
            }
        }
        aliases
    }

    /// Get all env vars from loaded plugins
    #[must_use]
    pub fn all_env_vars(&self) -> AHashMap<String, String> {
        let mut vars = AHashMap::new();
        for name in &self.load_order {
            if let Some(plugin) = self.plugins.get(name)
                && matches!(self.states.get(name), Some(PluginState::Loaded))
            {
                vars.extend(plugin.env_vars());
            }
        }
        vars
    }

    /// Generate shell init code for all loaded plugins
    #[must_use]
    pub fn shell_init(&self, shell: crate::ShellType) -> String {
        let mut init = String::new();
        for name in &self.load_order {
            if let Some(plugin) = self.plugins.get(name)
                && matches!(self.states.get(name), Some(PluginState::Loaded))
            {
                init.push_str(&plugin.shell_init(shell));
            }
        }
        init
    }

    /// Get plugin state
    #[must_use]
    pub fn state(&self, name: &str) -> Option<PluginState> {
        self.states.get(name).copied()
    }

    /// List all registered plugins
    #[must_use]
    pub fn list(&self) -> Vec<(&str, PluginState)> {
        self.plugins
            .keys()
            .map(|name| {
                let state = self.states.get(name).copied().unwrap_or(PluginState::Registered);
                (name.as_str(), state)
            })
            .collect()
    }

    /// Get loaded plugin count
    #[must_use]
    pub fn loaded_count(&self) -> usize {
        self.states
            .values()
            .filter(|&&s| s == PluginState::Loaded)
            .count()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_plugin() {
        let mut plugin = GitPlugin::new();
        assert!(plugin.init().is_ok());

        let info = plugin.info();
        assert_eq!(info.name, "git");

        let aliases = plugin.aliases();
        assert!(aliases.contains_key("gs"));
        assert_eq!(aliases.get("gs"), Some(&"git status".to_string()));
    }

    #[test]
    fn test_docker_plugin() {
        let mut plugin = DockerPlugin::new();
        assert!(plugin.init().is_ok());

        let info = plugin.info();
        assert_eq!(info.name, "docker");

        let aliases = plugin.aliases();
        assert!(aliases.contains_key("dps"));
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();

        // Should have built-in plugins
        let plugins = manager.list();
        assert!(plugins.iter().any(|(name, _)| *name == "git"));
        assert!(plugins.iter().any(|(name, _)| *name == "docker"));
    }

    #[test]
    fn test_plugin_load() {
        let mut manager = PluginManager::new();

        let result = manager.load("git");
        assert!(result.is_ok());
        assert_eq!(manager.state("git"), Some(PluginState::Loaded));
    }

    #[test]
    fn test_plugin_not_found() {
        let mut manager = PluginManager::new();

        let result = manager.load("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn test_plugin_aliases_aggregation() {
        let mut manager = PluginManager::new();
        manager.load("git").unwrap();
        manager.load("docker").unwrap();

        let aliases = manager.all_aliases();
        assert!(aliases.contains_key("gs")); // From git
        assert!(aliases.contains_key("dps")); // From docker
    }

    #[test]
    fn test_plugin_info_builder() {
        let info = PluginInfo::new("test")
            .with_description("A test plugin")
            .with_version("2.0.0")
            .with_dependency("git");

        assert_eq!(info.name, "test");
        assert_eq!(info.description, "A test plugin");
        assert_eq!(info.version, "2.0.0");
        assert_eq!(info.dependencies, vec!["git"]);
    }

    #[test]
    fn test_plugin_load_performance() {
        let mut manager = PluginManager::new();

        let start = Instant::now();
        manager.load("git").unwrap();
        manager.load("docker").unwrap();
        let elapsed = start.elapsed();

        // Both plugins should load under 10ms total
        assert!(
            elapsed < Duration::from_millis(10),
            "Plugin loading too slow: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_shell_init_generation() {
        let mut manager = PluginManager::new();
        manager.load("git").unwrap();

        let init = manager.shell_init(crate::ShellType::Zsh);
        assert!(init.contains("vcs_info")); // Zsh git integration
    }

    #[test]
    fn test_loaded_count() {
        let mut manager = PluginManager::new();
        assert_eq!(manager.loaded_count(), 0);

        manager.load("git").unwrap();
        assert_eq!(manager.loaded_count(), 1);

        manager.load("docker").unwrap();
        assert_eq!(manager.loaded_count(), 2);
    }
}
