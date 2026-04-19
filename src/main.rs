use anyhow::{anyhow, Context, Result};
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Select};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use update_informer::Check;

#[derive(Serialize, Deserialize, Default)]
struct Config {
    default: Option<String>,
    versions: HashMap<String, PathBuf>,
}

impl Config {
    fn load() -> Result<Self> {
        let config_path = get_config_path()?;
        if !config_path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(config_path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}

fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "godot-manager", "godot-manager")
        .context("Could not determine config directory")?;
    Ok(proj_dirs.config_dir().join("config.toml"))
}

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser)]
#[command(
    name = "godot",
    bin_name = "godot",
    display_name = "godot",
    about = "Godot Version Manager - Manage and run multiple Godot versions seamlessly",
    version = "1.0.0",
    styles = get_styles(),
    help_template = "{before-help}{name} {version}\n{about-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}",
    override_usage = "godot [mgr] [COMMAND] | godot [GODOT_ARGS]",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    args: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(
        alias = "m",
        override_usage = "godot mgr <COMMAND>",
        disable_help_subcommand = true
    )]
    Mgr {
        #[command(subcommand)]
        subcommand: MgrCommands,
    },
}

#[derive(Subcommand)]
enum MgrCommands {
    /// Add a new Godot version
    Add { name: String, path: PathBuf },
    /// List all registered versions
    List,
    /// Set a default Godot version
    Default {
        /// Name of the version to set as default (optional, shows menu if empty)
        name: Option<String>,
    },
    /// Remove a registered version
    Remove { name: String },
}

fn main() -> Result<()> {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let repo = "TadeoOrtiz/godot-manager";

    let informer = update_informer::new(update_informer::registry::GitHub, repo, version);
    if let Some(new_version) = informer.check_version().ok().flatten() {
        println!("\n\x1b[1;33m┌────────────────────────────────────────────────────────┐\x1b[0m");
        println!("\x1b[1;33m│\x1b[0m  🚀 A new version of \x1b[1;36m{}\x1b[0m is available!         \x1b[1;33m│\x1b[0m", name);
        println!("\x1b[1;33m│\x1b[0m  \x1b[1;30m{}\x1b[0m → \x1b[1;32m{}\x1b[0m                                \x1b[1;33m│\x1b[0m", version, new_version);
        println!("\x1b[1;33m│\x1b[0m                                                        \x1b[1;33m│\x1b[0m");
        println!("\x1b[1;33m│\x1b[0m  Run: \x1b[1;36mcargo install --git https://github.com/{}\x1b[0m \x1b[1;33m│\x1b[0m", repo);
        println!("\x1b[1;33m└────────────────────────────────────────────────────────┘\x1b[0m\n");
    }

    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Some(Commands::Mgr { subcommand }) => match subcommand {
            MgrCommands::Add { name, path } => {
                let absolute_path = fs::canonicalize(&path)
                    .with_context(|| format!("Could not find path: {:?}", path))?;
                config.versions.insert(name.clone(), absolute_path);
                config.save()?;
                println!("\x1b[32m✔\x1b[0m Added version: \x1b[1;36m{}\x1b[0m", name);
            }
            MgrCommands::List => {
                if config.versions.is_empty() {
                    println!("\x1b[33m⚠ No versions registered.\x1b[0m");
                } else {
                    println!("\x1b[1;33mRegistered Godot versions:\x1b[0m");
                    let mut names: Vec<_> = config.versions.keys().collect();
                    names.sort();
                    for name in names {
                        let path = &config.versions[name];
                        let is_default = config.default.as_ref() == Some(name);
                        if is_default {
                            println!(
                                "  \x1b[32m* \x1b[1m{}\x1b[0m \x1b[32m(default)\x1b[0m -> \x1b[3m{:?}\x1b[0m",
                                name, path
                            );
                        } else {
                            println!("    \x1b[1m{}\x1b[0m -> \x1b[3m{:?}\x1b[0m", name, path);
                        }
                    }
                }
            }
            MgrCommands::Default { name } => {
                let selected_name = if let Some(n) = name {
                    n
                } else {
                    if config.versions.is_empty() {
                        return Err(anyhow!("\x1b[33mNo versions available to set as default.\x1b[0m"));
                    }
                    let mut names: Vec<_> = config.versions.keys().cloned().collect();
                    names.sort();
                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select a version to set as default")
                        .items(&names)
                        .default(0)
                        .interact()?;
                    names[selection].clone()
                };

                if config.versions.contains_key(&selected_name) {
                    config.default = Some(selected_name.clone());
                    config.save()?;
                    println!(
                        "\x1b[32m✔\x1b[0m Default version set to: \x1b[1;36m{}\x1b[0m",
                        selected_name
                    );
                } else {
                    return Err(anyhow!(
                        "\x1b[31mError: Version '{}' not found\x1b[0m",
                        selected_name
                    ));
                }
            }
            MgrCommands::Remove { name } => {
                if config.versions.remove(&name).is_some() {
                    if config.default.as_ref() == Some(&name) {
                        config.default = None;
                    }
                    config.save()?;
                    println!(
                        "\x1b[32m✔\x1b[0m Removed version: \x1b[1;31m{}\x1b[0m",
                        name
                    );
                } else {
                    return Err(anyhow!(
                        "\x1b[31mError: Version '{}' not found\x1b[0m",
                        name
                    ));
                }
            }
        },
        None => {
            if let Some(first_arg) = cli.args.first() {
                if first_arg == "mrg" || first_arg == "manager" {
                    println!("\x1b[31mError: Unknown command '{}'. Did you mean '\x1b[1;32mmgr\x1b[31m'?\x1b[0m", first_arg);
                    println!("Run '\x1b[1;33mgodot mgr --help\x1b[0m' for management commands.");
                    return Ok(());
                }
            }
            run_godot(&config, cli.args)?;
        }
    }

    Ok(())
}

fn run_godot(config: &Config, args: Vec<String>) -> Result<()> {
    if config.versions.is_empty() {
        return Err(anyhow!("\x1b[33mNo Godot versions registered. Use '\x1b[1mgodot mgr add <name> <path>\x1b[0m\x1b[33m' first.\x1b[0m"));
    }

    let version_name = if let Some(ref default) = config.default {
        default.clone()
    } else {
        let mut names: Vec<_> = config.versions.keys().cloned().collect();
        names.sort();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select Godot version")
            .items(&names)
            .default(0)
            .interact()?;

        names[selection].clone()
    };

    let path = config
        .versions
        .get(&version_name)
        .ok_or_else(|| anyhow!("Version '{}' not found", version_name))?;

    println!(
        "\x1b[34m🚀 Running Godot \x1b[1;36m{}\x1b[34m...\x1b[0m",
        version_name
    );

    let mut child = Command::new(path)
        .args(args)
        .spawn()
        .with_context(|| format!("Failed to spawn Godot at {:?}", path))?;

    let status = child.wait()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
