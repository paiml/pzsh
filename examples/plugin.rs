//! Plugin system example
//!
//! Run with: cargo run --example plugin

use pzsh::ShellType;
use pzsh::plugin::{
    AwsPlugin, DockerPlugin, GitPlugin, GolangPlugin, KubectlPlugin, NpmPlugin, Plugin,
    PluginInfo, PluginManager, PythonPlugin, RustPlugin, TerraformPlugin,
};

fn main() {
    println!("=== pzsh Plugin System ===\n");
    println!("oh-my-zsh compatible plugins with O(1) loading\n");

    // Show all available plugins
    println!("=== Available Plugins (9) ===\n");

    let plugins: Vec<Box<dyn Plugin>> = vec![
        Box::new(GitPlugin::new()),
        Box::new(DockerPlugin::new()),
        Box::new(KubectlPlugin::new()),
        Box::new(NpmPlugin::new()),
        Box::new(PythonPlugin::new()),
        Box::new(GolangPlugin::new()),
        Box::new(RustPlugin::new()),
        Box::new(TerraformPlugin::new()),
        Box::new(AwsPlugin::new()),
    ];

    for plugin in &plugins {
        let info = plugin.info();
        let aliases = plugin.aliases();
        println!(
            "  {:12} - {} ({} aliases)",
            info.name,
            info.description,
            aliases.len()
        );
    }
    println!();

    // Plugin manager - load all plugins
    println!("=== Plugin Manager ===");
    let mut manager = PluginManager::new();

    let plugin_names = [
        "git",
        "docker",
        "kubectl",
        "npm",
        "python",
        "golang",
        "rust",
        "terraform",
        "aws",
    ];

    let mut total_time = std::time::Duration::ZERO;
    for name in plugin_names {
        match manager.load(name) {
            Ok(duration) => {
                total_time += duration;
                println!("  ✓ Loaded {} in {:?}", name, duration);
            }
            Err(e) => println!("  ✗ Failed to load {}: {}", name, e),
        }
    }
    println!("\nTotal load time: {:?}", total_time);
    println!("Loaded plugins: {}", manager.loaded_count());
    println!();

    // All aliases from loaded plugins
    println!("=== Combined Aliases ===");
    let all_aliases = manager.all_aliases();
    println!("Total aliases: {}", all_aliases.len());

    // Show sample aliases by category
    println!("\nSample aliases:");
    let samples = [
        ("gs", "git status"),
        ("dps", "docker ps"),
        ("k", "kubectl"),
        ("ni", "npm install"),
        ("py", "python3"),
        ("gob", "go build"),
        ("cb", "cargo build"),
        ("tf", "terraform"),
        ("awsw", "aws sts get-caller-identity"),
    ];
    for (alias, expected) in samples {
        if let Some(expansion) = all_aliases.get(alias) {
            println!("  {} -> {}", alias, expansion);
        } else {
            println!("  {} -> (expected: {})", alias, expected);
        }
    }
    println!();

    // Shell initialization code
    println!("=== Shell Init Code (ZSH) ===");
    let init_code = manager.shell_init(ShellType::Zsh);
    for line in init_code.lines().take(10) {
        println!("  {}", line);
    }
    println!("  ...");
    println!();

    // Plugin info builder
    println!("=== Plugin Info Builder ===");
    let custom_info = PluginInfo::new("custom-plugin")
        .with_description("A custom plugin example")
        .with_version("2.0.0")
        .with_dependency("git");
    println!("Name: {}", custom_info.name);
    println!("Description: {}", custom_info.description);
    println!("Version: {}", custom_info.version);
    println!("Dependencies: {:?}", custom_info.dependencies);
    println!();

    println!("Plugin load time: O(1) - no external scripts");
}
