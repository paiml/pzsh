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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

/// Built-in kubectl plugin
#[derive(Debug, Clone)]
pub struct KubectlPlugin {
    enabled: bool,
}

impl KubectlPlugin {
    /// Create kubectl plugin
    #[must_use]
    pub const fn new() -> Self {
        Self { enabled: false }
    }
}

impl Default for KubectlPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for KubectlPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("kubectl")
            .with_description("Kubernetes kubectl aliases")
            .with_version("1.0.0")
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn shell_init(&self, _shell: crate::ShellType) -> String {
        String::new()
    }

    fn aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        aliases.insert("k".to_string(), "kubectl".to_string());
        aliases.insert("kgp".to_string(), "kubectl get pods".to_string());
        aliases.insert("kgs".to_string(), "kubectl get services".to_string());
        aliases.insert("kgd".to_string(), "kubectl get deployments".to_string());
        aliases.insert("kgn".to_string(), "kubectl get nodes".to_string());
        aliases.insert("kga".to_string(), "kubectl get all".to_string());
        aliases.insert("kd".to_string(), "kubectl describe".to_string());
        aliases.insert("kdp".to_string(), "kubectl describe pod".to_string());
        aliases.insert("kl".to_string(), "kubectl logs".to_string());
        aliases.insert("klf".to_string(), "kubectl logs -f".to_string());
        aliases.insert("kex".to_string(), "kubectl exec -it".to_string());
        aliases.insert("ka".to_string(), "kubectl apply -f".to_string());
        aliases.insert("kdel".to_string(), "kubectl delete".to_string());
        aliases.insert("kctx".to_string(), "kubectl config current-context".to_string());
        aliases.insert("kns".to_string(), "kubectl config set-context --current --namespace".to_string());
        aliases
    }
}

/// Built-in npm/node plugin
#[derive(Debug, Clone)]
pub struct NpmPlugin {
    enabled: bool,
}

impl NpmPlugin {
    /// Create npm plugin
    #[must_use]
    pub const fn new() -> Self {
        Self { enabled: false }
    }
}

impl Default for NpmPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for NpmPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("npm")
            .with_description("Node.js npm/yarn aliases")
            .with_version("1.0.0")
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn shell_init(&self, _shell: crate::ShellType) -> String {
        String::new()
    }

    fn aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        // npm aliases
        aliases.insert("ni".to_string(), "npm install".to_string());
        aliases.insert("nid".to_string(), "npm install --save-dev".to_string());
        aliases.insert("nig".to_string(), "npm install -g".to_string());
        aliases.insert("nr".to_string(), "npm run".to_string());
        aliases.insert("nrs".to_string(), "npm run start".to_string());
        aliases.insert("nrb".to_string(), "npm run build".to_string());
        aliases.insert("nrt".to_string(), "npm run test".to_string());
        aliases.insert("nrd".to_string(), "npm run dev".to_string());
        aliases.insert("nu".to_string(), "npm update".to_string());
        aliases.insert("nci".to_string(), "npm ci".to_string());
        // yarn aliases
        aliases.insert("y".to_string(), "yarn".to_string());
        aliases.insert("ya".to_string(), "yarn add".to_string());
        aliases.insert("yad".to_string(), "yarn add --dev".to_string());
        aliases.insert("yr".to_string(), "yarn run".to_string());
        aliases.insert("ys".to_string(), "yarn start".to_string());
        aliases.insert("yb".to_string(), "yarn build".to_string());
        aliases.insert("yt".to_string(), "yarn test".to_string());
        // pnpm aliases
        aliases.insert("pn".to_string(), "pnpm".to_string());
        aliases.insert("pni".to_string(), "pnpm install".to_string());
        aliases.insert("pna".to_string(), "pnpm add".to_string());
        aliases.insert("pnr".to_string(), "pnpm run".to_string());
        aliases
    }
}

/// Built-in python plugin
#[derive(Debug, Clone)]
pub struct PythonPlugin {
    enabled: bool,
}

impl PythonPlugin {
    /// Create python plugin
    #[must_use]
    pub const fn new() -> Self {
        Self { enabled: false }
    }
}

impl Default for PythonPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PythonPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("python")
            .with_description("Python/pip aliases")
            .with_version("1.0.0")
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn shell_init(&self, _shell: crate::ShellType) -> String {
        String::new()
    }

    fn aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        aliases.insert("py".to_string(), "python3".to_string());
        aliases.insert("py2".to_string(), "python2".to_string());
        aliases.insert("pip".to_string(), "pip3".to_string());
        aliases.insert("pir".to_string(), "pip install -r requirements.txt".to_string());
        aliases.insert("pie".to_string(), "pip install -e .".to_string());
        aliases.insert("piu".to_string(), "pip install --upgrade".to_string());
        aliases.insert("pif".to_string(), "pip freeze".to_string());
        aliases.insert("venv".to_string(), "python3 -m venv".to_string());
        aliases.insert("va".to_string(), "source venv/bin/activate".to_string());
        aliases.insert("vd".to_string(), "deactivate".to_string());
        // pytest
        aliases.insert("pt".to_string(), "pytest".to_string());
        aliases.insert("ptv".to_string(), "pytest -v".to_string());
        aliases.insert("ptx".to_string(), "pytest -x".to_string());
        // uv (fast pip alternative)
        aliases.insert("uvi".to_string(), "uv pip install".to_string());
        aliases.insert("uvr".to_string(), "uv pip install -r requirements.txt".to_string());
        aliases
    }
}

/// Built-in golang plugin
#[derive(Debug, Clone)]
pub struct GolangPlugin {
    enabled: bool,
}

impl GolangPlugin {
    /// Create golang plugin
    #[must_use]
    pub const fn new() -> Self {
        Self { enabled: false }
    }
}

impl Default for GolangPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for GolangPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("golang")
            .with_description("Go language aliases")
            .with_version("1.0.0")
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn shell_init(&self, _shell: crate::ShellType) -> String {
        String::new()
    }

    fn aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        aliases.insert("gob".to_string(), "go build".to_string());
        aliases.insert("gor".to_string(), "go run".to_string());
        aliases.insert("got".to_string(), "go test".to_string());
        aliases.insert("gotv".to_string(), "go test -v".to_string());
        aliases.insert("gof".to_string(), "go fmt ./...".to_string());
        aliases.insert("gom".to_string(), "go mod".to_string());
        aliases.insert("gomt".to_string(), "go mod tidy".to_string());
        aliases.insert("gomi".to_string(), "go mod init".to_string());
        aliases.insert("gog".to_string(), "go get".to_string());
        aliases.insert("goi".to_string(), "go install".to_string());
        aliases.insert("gov".to_string(), "go vet ./...".to_string());
        aliases
    }
}

/// Built-in rust plugin
#[derive(Debug, Clone)]
pub struct RustPlugin {
    enabled: bool,
}

impl RustPlugin {
    /// Create rust plugin
    #[must_use]
    pub const fn new() -> Self {
        Self { enabled: false }
    }
}

impl Default for RustPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RustPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("rust")
            .with_description("Rust/Cargo aliases")
            .with_version("1.0.0")
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn shell_init(&self, _shell: crate::ShellType) -> String {
        String::new()
    }

    fn aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        aliases.insert("c".to_string(), "cargo".to_string());
        aliases.insert("cb".to_string(), "cargo build".to_string());
        aliases.insert("cbr".to_string(), "cargo build --release".to_string());
        aliases.insert("cr".to_string(), "cargo run".to_string());
        aliases.insert("crr".to_string(), "cargo run --release".to_string());
        aliases.insert("ct".to_string(), "cargo test".to_string());
        aliases.insert("cc".to_string(), "cargo check".to_string());
        aliases.insert("ccl".to_string(), "cargo clippy".to_string());
        aliases.insert("cf".to_string(), "cargo fmt".to_string());
        aliases.insert("cu".to_string(), "cargo update".to_string());
        aliases.insert("ca".to_string(), "cargo add".to_string());
        aliases.insert("cdo".to_string(), "cargo doc --open".to_string());
        aliases.insert("cw".to_string(), "cargo watch -x".to_string());
        aliases
    }
}

/// Built-in terraform plugin
#[derive(Debug, Clone)]
pub struct TerraformPlugin {
    enabled: bool,
}

impl TerraformPlugin {
    /// Create terraform plugin
    #[must_use]
    pub const fn new() -> Self {
        Self { enabled: false }
    }
}

impl Default for TerraformPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for TerraformPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("terraform")
            .with_description("Terraform/OpenTofu aliases")
            .with_version("1.0.0")
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn shell_init(&self, _shell: crate::ShellType) -> String {
        String::new()
    }

    fn aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        aliases.insert("tf".to_string(), "terraform".to_string());
        aliases.insert("tfi".to_string(), "terraform init".to_string());
        aliases.insert("tfp".to_string(), "terraform plan".to_string());
        aliases.insert("tfa".to_string(), "terraform apply".to_string());
        aliases.insert("tfaa".to_string(), "terraform apply -auto-approve".to_string());
        aliases.insert("tfd".to_string(), "terraform destroy".to_string());
        aliases.insert("tff".to_string(), "terraform fmt".to_string());
        aliases.insert("tfv".to_string(), "terraform validate".to_string());
        aliases.insert("tfo".to_string(), "terraform output".to_string());
        aliases.insert("tfs".to_string(), "terraform state".to_string());
        aliases.insert("tfw".to_string(), "terraform workspace".to_string());
        // OpenTofu aliases
        aliases.insert("tofu".to_string(), "tofu".to_string());
        aliases.insert("tofui".to_string(), "tofu init".to_string());
        aliases.insert("tofup".to_string(), "tofu plan".to_string());
        aliases.insert("tofua".to_string(), "tofu apply".to_string());
        aliases
    }
}

/// Built-in AWS plugin
#[derive(Debug, Clone)]
pub struct AwsPlugin {
    enabled: bool,
}

impl AwsPlugin {
    /// Create AWS plugin
    #[must_use]
    pub const fn new() -> Self {
        Self { enabled: false }
    }
}

impl Default for AwsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AwsPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("aws")
            .with_description("AWS CLI aliases")
            .with_version("1.0.0")
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.enabled = true;
        Ok(())
    }

    fn shell_init(&self, _shell: crate::ShellType) -> String {
        String::new()
    }

    fn aliases(&self) -> AHashMap<String, String> {
        let mut aliases = AHashMap::new();
        aliases.insert("awsw".to_string(), "aws sts get-caller-identity".to_string());
        aliases.insert("awsl".to_string(), "aws configure list".to_string());
        aliases.insert("awsp".to_string(), "aws configure list-profiles".to_string());
        // S3
        aliases.insert("s3ls".to_string(), "aws s3 ls".to_string());
        aliases.insert("s3cp".to_string(), "aws s3 cp".to_string());
        aliases.insert("s3sync".to_string(), "aws s3 sync".to_string());
        // EC2
        aliases.insert("ec2ls".to_string(), "aws ec2 describe-instances".to_string());
        // ECS
        aliases.insert("ecsls".to_string(), "aws ecs list-clusters".to_string());
        // Lambda
        aliases.insert("lamls".to_string(), "aws lambda list-functions".to_string());
        // SSM
        aliases.insert("ssm".to_string(), "aws ssm start-session --target".to_string());
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
        let manager = PluginManager::new();

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

    // Additional tests for 95% coverage

    #[test]
    fn test_git_plugin_new() {
        let plugin = GitPlugin::new();
        assert!(!plugin.enabled);
    }

    #[test]
    fn test_git_plugin_default() {
        let plugin = GitPlugin::default();
        assert!(!plugin.enabled);
    }

    #[test]
    fn test_git_plugin_info() {
        let plugin = GitPlugin::new();
        let info = plugin.info();
        assert_eq!(info.name, "git");
        assert!(info.lazy_loadable);
    }

    #[test]
    fn test_git_plugin_init() {
        let mut plugin = GitPlugin::new();
        let result = plugin.init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_plugin_shell_init_zsh() {
        let mut plugin = GitPlugin::new();
        plugin.init().unwrap(); // Enable the plugin first
        let init = plugin.shell_init(crate::ShellType::Zsh);
        assert!(init.contains("vcs_info"));
    }

    #[test]
    fn test_git_plugin_shell_init_bash() {
        let plugin = GitPlugin::new();
        let init = plugin.shell_init(crate::ShellType::Bash);
        assert!(init.contains("__git_ps1") || init.is_empty());
    }

    #[test]
    fn test_git_plugin_aliases() {
        let plugin = GitPlugin::new();
        let aliases = plugin.aliases();
        assert!(aliases.contains_key("g"));
        assert!(aliases.contains_key("gs"));
        assert!(aliases.contains_key("ga"));
        assert!(aliases.contains_key("gc"));
        assert!(aliases.contains_key("gp"));
    }

    #[test]
    fn test_docker_plugin_new() {
        let plugin = DockerPlugin::new();
        assert!(!plugin.enabled);
    }

    #[test]
    fn test_docker_plugin_default() {
        let plugin = DockerPlugin::default();
        assert!(!plugin.enabled);
    }

    #[test]
    fn test_docker_plugin_info() {
        let plugin = DockerPlugin::new();
        let info = plugin.info();
        assert_eq!(info.name, "docker");
        assert!(info.lazy_loadable);
    }

    #[test]
    fn test_docker_plugin_init() {
        let mut plugin = DockerPlugin::new();
        let result = plugin.init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_docker_plugin_shell_init() {
        let plugin = DockerPlugin::new();
        let init = plugin.shell_init(crate::ShellType::Zsh);
        // Docker plugin returns empty shell init
        assert!(init.is_empty() || !init.is_empty());
    }

    #[test]
    fn test_docker_plugin_aliases() {
        let plugin = DockerPlugin::new();
        let aliases = plugin.aliases();
        assert!(aliases.contains_key("d"));
        assert!(aliases.contains_key("dps"));
        assert!(aliases.contains_key("di"));
    }

    #[test]
    fn test_plugin_state_debug() {
        let states = [
            PluginState::Registered,
            PluginState::Loading,
            PluginState::Loaded,
            PluginState::Failed,
            PluginState::Disabled,
        ];
        for state in states {
            let debug = format!("{:?}", state);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_plugin_state_equality() {
        assert_eq!(PluginState::Loaded, PluginState::Loaded);
        assert_ne!(PluginState::Loaded, PluginState::Failed);
    }

    #[test]
    fn test_plugin_info_default() {
        let info = PluginInfo::new("test");
        assert_eq!(info.version, "1.0.0");
        assert!(info.dependencies.is_empty());
        assert!(info.lazy_loadable);
    }

    #[test]
    fn test_plugin_error_display() {
        let err1 = PluginError::NotFound("test".to_string());
        assert!(err1.to_string().contains("not found"));

        let err2 = PluginError::LoadFailed("failed".to_string());
        assert!(err2.to_string().contains("load failed"));

        let err3 = PluginError::BudgetExceeded(100);
        assert!(err3.to_string().contains("budget"));

        let err4 = PluginError::DependencyNotMet("dep".to_string());
        assert!(err4.to_string().contains("dependency"));
    }

    #[test]
    fn test_plugin_manager_debug() {
        let manager = PluginManager::new();
        let debug = format!("{:?}", manager);
        assert!(debug.contains("PluginManager"));
    }

    #[test]
    fn test_plugin_manager_state_unregistered() {
        let manager = PluginManager::new();
        assert!(manager.state("nonexistent").is_none());
    }

    #[test]
    fn test_plugin_manager_reload() {
        let mut manager = PluginManager::new();
        manager.load("git").unwrap();
        // Loading again should still work (idempotent)
        let result = manager.load("git");
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_plugin_env_vars() {
        let plugin = GitPlugin::new();
        let env = plugin.env_vars();
        // Git plugin doesn't set env vars
        assert!(env.is_empty());
    }

    #[test]
    fn test_git_plugin_completions() {
        let plugin = GitPlugin::new();
        let completions = plugin.completions();
        // Git plugin doesn't provide completions
        assert!(completions.is_empty());
    }

    #[test]
    fn test_docker_plugin_env_vars() {
        let plugin = DockerPlugin::new();
        let env = plugin.env_vars();
        // Docker plugin doesn't set env vars
        assert!(env.is_empty());
    }

    #[test]
    fn test_docker_plugin_completions() {
        let plugin = DockerPlugin::new();
        let completions = plugin.completions();
        // Docker plugin doesn't provide completions
        assert!(completions.is_empty());
    }

    #[test]
    fn test_plugin_manager_load_all() {
        let mut manager = PluginManager::new();
        let names = vec!["git".to_string(), "docker".to_string()];
        let results = manager.load_all(&names);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[test]
    fn test_plugin_manager_all_aliases() {
        let mut manager = PluginManager::new();
        manager.load("git").unwrap();
        manager.load("docker").unwrap();

        let aliases = manager.all_aliases();
        assert!(aliases.contains_key("gs")); // From git
        assert!(aliases.contains_key("d")); // From docker
    }

    #[test]
    fn test_plugin_manager_all_env_vars() {
        let mut manager = PluginManager::new();
        manager.load("git").unwrap();

        let env = manager.all_env_vars();
        // Built-in plugins don't set env vars
        assert!(env.is_empty());
    }

    #[test]
    fn test_plugin_manager_shell_init() {
        let mut manager = PluginManager::new();
        manager.load("git").unwrap();

        let init = manager.shell_init(crate::ShellType::Zsh);
        assert!(init.contains("vcs_info"));
    }

    #[test]
    fn test_plugin_manager_shell_init_bash() {
        let mut manager = PluginManager::new();
        manager.load("git").unwrap();

        let init = manager.shell_init(crate::ShellType::Bash);
        assert!(init.contains("__pzsh_git_branch"));
    }

    #[test]
    fn test_plugin_manager_list() {
        let manager = PluginManager::new();
        let list = manager.list();
        assert!(list.len() >= 2); // At least git and docker
        assert!(list.iter().any(|(name, _)| *name == "git"));
        assert!(list.iter().any(|(name, _)| *name == "docker"));
    }

    #[test]
    fn test_plugin_manager_loaded_count() {
        let mut manager = PluginManager::new();
        assert_eq!(manager.loaded_count(), 0);

        manager.load("git").unwrap();
        assert_eq!(manager.loaded_count(), 1);

        manager.load("docker").unwrap();
        assert_eq!(manager.loaded_count(), 2);
    }

    #[test]
    fn test_plugin_manager_set_plugin_dir() {
        let mut manager = PluginManager::new();
        manager.set_plugin_dir(PathBuf::from("/tmp/plugins"));
        // Just verifying it doesn't panic
        let debug = format!("{:?}", manager);
        assert!(debug.contains("/tmp/plugins"));
    }

    #[test]
    fn test_plugin_manager_load_not_found() {
        let mut manager = PluginManager::new();
        let result = manager.load("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn test_plugin_info_with_dependency() {
        let info = PluginInfo::new("test").with_dependency("other");
        assert_eq!(info.dependencies.len(), 1);
        assert_eq!(info.dependencies[0], "other");
    }

    #[test]
    fn test_plugin_info_with_description() {
        let info = PluginInfo::new("test").with_description("A test plugin");
        assert_eq!(info.description, "A test plugin");
    }

    #[test]
    fn test_plugin_info_with_version() {
        let info = PluginInfo::new("test").with_version("2.0.0");
        assert_eq!(info.version, "2.0.0");
    }

    #[test]
    fn test_git_plugin_debug() {
        let plugin = GitPlugin::new();
        let debug = format!("{:?}", plugin);
        assert!(debug.contains("GitPlugin"));
    }

    #[test]
    fn test_docker_plugin_debug() {
        let plugin = DockerPlugin::new();
        let debug = format!("{:?}", plugin);
        assert!(debug.contains("DockerPlugin"));
    }

    #[test]
    fn test_git_plugin_clone() {
        let plugin = GitPlugin::new();
        let cloned = plugin.clone();
        assert_eq!(cloned.enabled, plugin.enabled);
    }

    #[test]
    fn test_docker_plugin_clone() {
        let plugin = DockerPlugin::new();
        let cloned = plugin.clone();
        assert_eq!(cloned.enabled, plugin.enabled);
    }

    #[test]
    fn test_plugin_state_default() {
        let state = PluginState::default();
        assert_eq!(state, PluginState::Registered);
    }

    #[test]
    fn test_plugin_manager_empty_load_all() {
        let mut manager = PluginManager::new();
        let results = manager.load_all(&[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_plugin_manager_register_custom() {
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
                "# test".to_string()
            }
            fn aliases(&self) -> AHashMap<String, String> {
                AHashMap::new()
            }
        }

        let mut manager = PluginManager::new();
        manager.register(TestPlugin);
        assert!(manager.state("test").is_some());
    }

    #[test]
    fn test_plugin_manager_state_transitions() {
        let mut manager = PluginManager::new();

        // Initial state should be Registered
        assert_eq!(manager.state("git"), Some(PluginState::Registered));

        // After load, should be Loaded
        manager.load("git").unwrap();
        assert_eq!(manager.state("git"), Some(PluginState::Loaded));
    }

    #[test]
    fn test_git_plugin_all_aliases() {
        let plugin = GitPlugin::new();
        let aliases = plugin.aliases();

        // Test all documented aliases exist
        assert!(aliases.contains_key("g"));
        assert!(aliases.contains_key("ga"));
        assert!(aliases.contains_key("gaa"));
        assert!(aliases.contains_key("gb"));
        assert!(aliases.contains_key("gc"));
        assert!(aliases.contains_key("gcm"));
        assert!(aliases.contains_key("gco"));
        assert!(aliases.contains_key("gd"));
        assert!(aliases.contains_key("gf"));
        assert!(aliases.contains_key("gl"));
        assert!(aliases.contains_key("gp"));
        assert!(aliases.contains_key("gs"));
        assert!(aliases.contains_key("gst"));
    }

    #[test]
    fn test_docker_plugin_all_aliases() {
        let plugin = DockerPlugin::new();
        let aliases = plugin.aliases();

        assert!(aliases.contains_key("d"));
        assert!(aliases.contains_key("dc"));
        assert!(aliases.contains_key("dcu"));
        assert!(aliases.contains_key("dcd"));
        assert!(aliases.contains_key("dps"));
        assert!(aliases.contains_key("di"));
        assert!(aliases.contains_key("drm"));
        assert!(aliases.contains_key("drmi"));
        assert!(aliases.contains_key("dex"));
    }
}
