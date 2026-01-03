//! Plugin system example
//!
//! Run with: cargo run --example plugin

use pzsh::plugin::{Plugin, PluginManager, GitPlugin, DockerPlugin, PluginInfo};
use pzsh::ShellType;

fn main() {
    println!("=== pzsh Plugin System ===\n");
    println!("oh-my-zsh compatible plugins with O(1) loading\n");

    // Git plugin
    println!("=== Git Plugin ===");
    let mut git = GitPlugin::new();
    let _ = git.init();
    let info = git.info();
    println!("Name: {}", info.name);
    println!("Description: {}", info.description);
    println!("Version: {}", info.version);
    println!("Aliases ({}):", git.aliases().len());
    for (alias, expansion) in git.aliases().iter().take(10) {
        println!("  {} -> {}", alias, expansion);
    }
    println!("  ... and {} more", git.aliases().len().saturating_sub(10));
    println!();

    // Docker plugin
    println!("=== Docker Plugin ===");
    let mut docker = DockerPlugin::new();
    let _ = docker.init();
    let info = docker.info();
    println!("Name: {}", info.name);
    println!("Description: {}", info.description);
    println!("Aliases ({}):", docker.aliases().len());
    for (alias, expansion) in docker.aliases() {
        println!("  {} -> {}", alias, expansion);
    }
    println!();

    // Plugin manager
    println!("=== Plugin Manager ===");
    let mut manager = PluginManager::new();

    // Load plugins
    match manager.load("git") {
        Ok(duration) => println!("Loaded git plugin in {:?}", duration),
        Err(e) => println!("Failed to load git: {}", e),
    }
    match manager.load("docker") {
        Ok(duration) => println!("Loaded docker plugin in {:?}", duration),
        Err(e) => println!("Failed to load docker: {}", e),
    }
    println!();

    // All aliases from loaded plugins
    println!("All plugin aliases:");
    let all_aliases = manager.all_aliases();
    for (name, expansion) in all_aliases.iter().take(15) {
        println!("  {} -> {}", name, expansion);
    }
    println!("  ... total: {} aliases", all_aliases.len());
    println!();

    // Shell initialization code
    println!("=== Shell Init Code (ZSH) ===");
    let init_code = manager.shell_init(ShellType::Zsh);
    for line in init_code.lines() {
        println!("  {}", line);
    }
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

    // Available plugins
    println!("=== Available Plugins ===");
    println!("Built-in plugins:");
    println!("  - git: Git aliases and integration");
    println!("  - docker: Docker aliases");
    println!();
    println!("Plugin load time: O(1) - no external scripts");
}
