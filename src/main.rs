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

#[derive(Parser)]
#[command(name = "RSM")]
#[command(about = "Rusty Symlink Manager", long_about = None)]
struct Cli {
    /// Path to a specific rsm.toml file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Force overwrite of existing files/symlinks
    #[arg(short, long, global = true)]
    force: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Sync {
        #[arg(long)]
        tag: Option<String>,

        #[arg(long)]
        dry_run: bool,
    },
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
    init_tracing();
    ui::print_banner();

    let cli = Cli::parse();
    let current_env = env::Environment::current();

    match &cli.command {
        Commands::Init => {
            let target_path = cli.config.unwrap_or_else(|| PathBuf::from("rsm.toml"));
            config::Config::init_template(&target_path)?;
            info!(
                "{} Initialized template at {}",
                "✔".green(),
                target_path.display()
            );
        }

        Commands::Sync { tag, dry_run } => {
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

        Commands::Check => {
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
    }

    Ok(())
}
