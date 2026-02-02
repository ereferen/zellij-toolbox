//! toolbox CLI - Display development tool versions

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use toolbox_core::{Config, ToolDetector};

#[derive(Parser)]
#[command(name = "toolbox")]
#[command(about = "Display development tool versions and system info")]
#[command(version)]
struct Cli {
    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Working directory (for asdf/mise directory-specific versions)
    #[arg(short = 'd', long)]
    dir: Option<String>,

    /// Output format
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,

    /// Compact output
    #[arg(long)]
    compact: bool,

    /// Hide icons
    #[arg(long)]
    no_icons: bool,

    /// Powerline style output (colored segments with separators)
    #[arg(long)]
    powerline: bool,

    /// Single line output (only with --powerline)
    #[arg(long)]
    single_line: bool,

    /// Color mode: auto, always, never
    #[arg(long, default_value = "auto")]
    color: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    JsonPretty,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize configuration file
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },
    /// Show current configuration
    ShowConfig,
    /// List available tools
    ListTools,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(ref command) = cli.command {
        return handle_command(command, &cli);
    }

    // Load config
    let config = if let Some(ref config_path) = cli.config {
        Config::load_from_path(config_path)?
    } else {
        Config::load()?
    };

    // Create detector
    let mut detector = ToolDetector::new(config);
    if let Some(ref dir) = cli.dir {
        detector = detector.with_working_dir(dir.clone());
    }

    // Detect all tools
    let info = detector.detect_all();

    // Parse color mode
    let color_mode: toolbox_core::color::ColorMode = cli
        .color
        .parse()
        .unwrap_or(toolbox_core::color::ColorMode::Auto);
    let use_color = toolbox_core::color::should_use_color(color_mode);

    // Output
    match cli.format {
        OutputFormat::Text => {
            let compact = cli.compact || detector.config().display.compact;
            let show_icons = !cli.no_icons && detector.config().display.show_icons;

            if cli.powerline {
                println!(
                    "{}",
                    info.format_powerline(compact, show_icons, use_color, cli.single_line)
                );
            } else {
                println!("{}", info.format_display(compact, show_icons));
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&info)?);
        }
        OutputFormat::JsonPretty => {
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
    }

    Ok(())
}

fn handle_command(command: &Commands, cli: &Cli) -> Result<()> {
    match command {
        Commands::Init { force } => {
            let force = *force;
            let config_path = if let Some(ref path) = cli.config {
                path.clone()
            } else {
                Config::config_path()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine config path"))?
            };

            if config_path.exists() && !force {
                eprintln!("Config file already exists at: {}", config_path.display());
                eprintln!("Use --force to overwrite");
                return Ok(());
            }

            let config = Config::default();
            config.save_to_path(&config_path)?;
            println!("Created config file at: {}", config_path.display());
        }

        Commands::ShowConfig => {
            let config = if let Some(ref config_path) = cli.config {
                Config::load_from_path(config_path)?
            } else {
                Config::load()?
            };

            let toml_str = toml::to_string_pretty(&config)?;
            println!("{}", toml_str);
        }

        Commands::ListTools => {
            let config = Config::default();
            println!("Available tools:\n");
            for tool in &config.effective_tools() {
                let status = if tool.enabled { "enabled" } else { "disabled" };
                let icon = tool.icon.as_deref().unwrap_or(" ");
                println!("  {} {} ({}) - {}", icon, tool.name, status, tool.command);
            }
            println!("\nEdit your config file to enable/disable tools or add custom ones.");
            if let Some(path) = Config::config_path() {
                println!("Config path: {}", path.display());
            }
        }
    }

    Ok(())
}
