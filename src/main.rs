mod config;
mod env;
mod error;
mod symlink;
mod ui;

use clap::{Parser, Subcommand};
use colored::*;
use config::Config;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// RSM (Rusty Symlink Manager)
/// A high-performance, modular system utility for managing symbolic links.
#[derive(Parser)]
#[command(name = "RSM")]
#[command(
    version,
    about = "Rusty Symlink Manager",
    long_about = "A high-performance, modular system utility written in Rust for managing symbolic links via a centralized configuration."
)]
struct Cli {
    /// Path to a specific config file (Defaults to XDG paths or ./rsm.toml)
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Force overwrite of existing files/symlinks
    #[arg(short, long, global = true)]
    force: bool,

    // Make the subcommand OPTIONAL so 'clap' doesn't auto-fail when running just `rsm`
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes a new default rsm.toml template
    Init,

    /// Synchronizes symlinks based on your configuration
    Sync {
        /// Filter and apply only the links matching this tag (e.g., "ui", "work")
        #[arg(long)]
        tag: Option<String>,

        /// Preview the sync process without making any actual filesystem changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Validates your config against the current file system
    Check,
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .without_time()
        .init();
}

fn main() -> Result<(), error::RsmError> {
    let cli = Cli::parse();

    init_tracing();
    ui::print_banner();

    let current_env = env::Environment::current();

    // Match on Some(command) or None
    match &cli.command {
        Some(Commands::Init) => {
            let target_path = cli.config.unwrap_or_else(|| PathBuf::from("rsm.toml"));
            config::Config::init_template(&target_path)?;
            info!(
                "{} Initialized template at {}",
                "✔".green(),
                target_path.display()
            );
        }

        Some(Commands::Sync { tag, dry_run }) => {
            let config_path = Config::resolve_path(cli.config.as_ref())?;
            let config = Config::load(&config_path)?;

            if *dry_run {
                warn!(
                    "{} Executing in DRY RUN mode. No changes will be made.",
                    "⚠".yellow()
                );
            }

            info!(
                "Detected OS: {}, Hostname: {}",
                current_env.os.cyan(),
                current_env.hostname.cyan()
            );

            let links_to_process: Vec<_> = config
                .links
                .into_iter()
                .filter(|link| {
                    if let Some(os) = &link.os {
                        if os != &current_env.os {
                            return false;
                        }
                    }

                    if let Some(target_tag) = tag {
                        match &link.tags {
                            Some(tags) => {
                                if !tags.contains(target_tag) {
                                    return false;
                                }
                            }
                            None => return false,
                        }
                    }
                    true
                })
                .collect();

            if links_to_process.is_empty() {
                info!("No links matched the current environment criteria.");
                return Ok(());
            }

            let pb = ui::create_progress_bar(links_to_process.len() as u64);

            for link in links_to_process {
                pb.set_message(format!(
                    "Syncing {:?}",
                    link.target.file_name().unwrap_or_default()
                ));

                match symlink::process_link(&link.target, &link.source, cli.force, *dry_run) {
                    Ok(_) => {}
                    Err(crate::error::RsmError::SourceMissing(p)) => {
                        pb.suspend(|| warn!("Skipping: Source missing at {}", p.display()));
                    }
                    Err(e) => {
                        pb.suspend(|| {
                            error!(
                                "{} Failed to link {}: {}",
                                "✖".red(),
                                link.target.display(),
                                e
                            )
                        });
                    }
                }
                pb.inc(1);
            }
            pb.finish_with_message("Sync complete.");
        }

        Some(Commands::Check) => {
            let config_path = Config::resolve_path(cli.config.as_ref())?;
            let config = Config::load(&config_path)?;

            info!(
                "{} Checking configuration at {}...",
                "ℹ".blue(),
                config_path.display()
            );
            for link in config.links.iter() {
                let target_str = link.target.to_string_lossy().replace(
                    "~",
                    &directories::BaseDirs::new()
                        .unwrap()
                        .home_dir()
                        .to_string_lossy(),
                );
                let target_path = PathBuf::from(target_str);

                if target_path.is_symlink() {
                    println!(
                        "{} {} -> {}",
                        "✔".green(),
                        link.target.display(),
                        link.source.display()
                    );
                } else if target_path.exists() {
                    println!(
                        "{} {} is a regular file/dir (Conflict)",
                        "✖".red(),
                        link.target.display()
                    );
                } else {
                    println!("{} {} is missing", "⚠".yellow(), link.target.display());
                }
            }
        }

        // Handle the case where the user just types `rsm`
        None => {
            println!(
                "Welcome to RSM! Run {} to see available commands.",
                "rsm --help".yellow().bold()
            );
        }
    }

    Ok(())
}
