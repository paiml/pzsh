//! Plugin module for pzsh
//!
//! Lightweight plugin system inspired by oh-my-zsh.
//! Plugins are loaded lazily to maintain O(1) startup.

mod builtins;
pub use builtins::*;

use ahash::AHashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Plugin loading budget (per plugin)
pub const PLUGIN_BUDGET_MS: u64 = 5;

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PluginState {
    /// Plugin registered but not loaded
    #[default]
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
        contract_pre_startup_budget!(input);
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
        manager.register(KubectlPlugin::new());
        manager.register(NpmPlugin::new());
        manager.register(PythonPlugin::new());
        manager.register(GolangPlugin::new());
        manager.register(RustPlugin::new());
        manager.register(TerraformPlugin::new());
        manager.register(AwsPlugin::new());

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
        contract_pre_config_validation!(name);
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
                let state = self
                    .states
                    .get(name)
                    .copied()
                    .unwrap_or(PluginState::Registered);
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

    // ==================== PluginState TESTS ====================

    #[test]
    fn test_plugin_state_default() {
        assert_eq!(PluginState::default(), PluginState::Registered);
    }

    #[test]
    fn test_plugin_state_debug_and_equality() {
        let states = [
            PluginState::Registered,
            PluginState::Loading,
            PluginState::Loaded,
            PluginState::Failed,
            PluginState::Disabled,
        ];
        for state in states {
            assert!(!format!("{:?}", state).is_empty());
        }
        assert_eq!(PluginState::Loaded, PluginState::Loaded);
        assert_ne!(PluginState::Loaded, PluginState::Failed);
    }

    // ==================== PluginInfo TESTS ====================

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
        assert!(info.lazy_loadable);
    }

    #[test]
    fn test_plugin_info_defaults() {
        let info = PluginInfo::new("test");
        assert_eq!(info.version, "1.0.0");
        assert!(info.dependencies.is_empty());
        assert!(info.lazy_loadable);
    }

    // ==================== PluginError TESTS ====================

    #[test]
    fn test_plugin_error_display() {
        assert!(
            PluginError::NotFound("x".into())
                .to_string()
                .contains("not found")
        );
        assert!(
            PluginError::LoadFailed("x".into())
                .to_string()
                .contains("load failed")
        );
        assert!(
            PluginError::BudgetExceeded(100)
                .to_string()
                .contains("budget")
        );
        assert!(
            PluginError::DependencyNotMet("x".into())
                .to_string()
                .contains("dependency")
        );
    }

    // ==================== PluginManager TESTS ====================

    #[test]
    fn test_manager_new_has_all_builtins() {
        let manager = PluginManager::new();
        let list = manager.list();
        assert!(list.len() >= 9);
        for name in [
            "git",
            "docker",
            "kubectl",
            "npm",
            "python",
            "golang",
            "rust",
            "terraform",
            "aws",
        ] {
            assert!(
                list.iter().any(|(n, _)| *n == name),
                "missing plugin: {name}"
            );
        }
    }

    #[test]
    fn test_manager_load_and_state_transitions() {
        let mut manager = PluginManager::new();
        assert_eq!(manager.state("git"), Some(PluginState::Registered));
        assert!(manager.load("git").is_ok());
        assert_eq!(manager.state("git"), Some(PluginState::Loaded));
        // Reload is idempotent
        assert!(manager.load("git").is_ok());
    }

    #[test]
    fn test_manager_load_not_found() {
        let mut manager = PluginManager::new();
        assert!(matches!(
            manager.load("nonexistent"),
            Err(PluginError::NotFound(_))
        ));
        assert!(manager.state("nonexistent").is_none());
    }

    #[test]
    fn test_manager_load_all() {
        let mut manager = PluginManager::new();
        let results = manager.load_all(&["git".into(), "docker".into()]);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
        assert_eq!(manager.loaded_count(), 2);

        // Empty load_all
        assert!(manager.load_all(&[]).is_empty());
    }

    #[test]
    fn test_manager_all_aliases() {
        let mut manager = PluginManager::new();
        manager.load("git").unwrap();
        manager.load("docker").unwrap();
        let aliases = manager.all_aliases();
        assert!(aliases.contains_key("gs"));
        assert!(aliases.contains_key("dps"));
    }

    #[test]
    fn test_manager_all_env_vars() {
        let mut manager = PluginManager::new();
        manager.load("git").unwrap();
        assert!(manager.all_env_vars().is_empty());
    }

    #[test]
    fn test_manager_shell_init() {
        let mut manager = PluginManager::new();
        manager.load("git").unwrap();
        assert!(
            manager
                .shell_init(crate::ShellType::Zsh)
                .contains("vcs_info")
        );
        assert!(
            manager
                .shell_init(crate::ShellType::Bash)
                .contains("__pzsh_git_branch")
        );
    }

    #[test]
    fn test_manager_debug_and_plugin_dir() {
        let mut manager = PluginManager::new();
        assert!(format!("{:?}", manager).contains("PluginManager"));
        manager.set_plugin_dir(PathBuf::from("/tmp/plugins"));
        assert!(format!("{:?}", manager).contains("/tmp/plugins"));
    }

    #[test]
    fn test_manager_register_custom() {
        #[derive(Clone, Debug)]
        struct TestPlugin;
        impl Plugin for TestPlugin {
            fn info(&self) -> PluginInfo {
                PluginInfo::new("test")
            }
            fn init(&mut self) -> Result<(), PluginError> {
                Ok(())
            }
            fn shell_init(&self, _: crate::ShellType) -> String {
                "# test".into()
            }
        }

        let mut manager = PluginManager::new();
        manager.register(TestPlugin);
        assert!(manager.state("test").is_some());
    }

    #[test]
    fn test_manager_load_performance() {
        let mut manager = PluginManager::new();
        let start = Instant::now();
        manager.load("git").unwrap();
        manager.load("docker").unwrap();
        assert!(start.elapsed() < Duration::from_millis(10));
    }
}
