mod config;
mod env;
mod error;
mod symlink;
mod ui;

use clap::{Parser, Subcommand};
use colored::*;
use config::Config;
use rayon::prelude::*;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// RSM (Rusty Symlink Manager)
#[derive(Parser)]
#[command(
    name = "RSM",
    version,
    about = "A high-performance, modular system utility written in Rust for managing symbolic links."
)]
struct Cli {
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[arg(short, long, global = true)]
    force: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Sync {
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        prune: bool,
    },
    Check,
    Info,
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

        Some(Commands::Info) => {
            println!("{}", "System Information".bold().underline());
            println!("{:<10} {}", "OS:".cyan(), current_env.os);
            println!("{:<10} {}", "Hostname:".cyan(), current_env.hostname);
            println!("{:<10} {}", "Arch:".cyan(), current_env.arch);
        }

        Some(Commands::Sync {
            tag,
            dry_run,
            prune,
        }) => {
            let config_path = Config::resolve_path(cli.config.as_ref())?;
            let config = Config::load(&config_path)?;
            let is_dry_run = *dry_run;

            if is_dry_run {
                warn!(
                    "{} Executing in DRY RUN mode. No changes will be made.",
                    "⚠".yellow()
                );
            }

            let mut valid_links = Vec::new();
            for link in config.links {
                let mut skip = false;

                if let Some(os) = &link.os {
                    if os != &current_env.os {
                        warn!(
                            "Skipping {} (OS mismatch: requires {})",
                            link.target.display(),
                            os
                        );
                        skip = true;
                    }
                }
                if !skip {
                    if let Some(host) = &link.host {
                        if host != &current_env.hostname {
                            warn!(
                                "Skipping {} (Host mismatch: requires {})",
                                link.target.display(),
                                host
                            );
                            skip = true;
                        }
                    }
                }
                if !skip {
                    if let Some(target_tag) = tag {
                        if let Some(tags) = &link.tags {
                            if !tags.contains(target_tag) {
                                skip = true;
                            }
                        } else {
                            skip = true;
                        }
                    }
                }

                if !skip {
                    valid_links.push(link);
                }
            }

            if valid_links.is_empty() {
                info!("No links matched the current environment criteria.");
                return Ok(());
            }

            let global_ignores = config.global_ignores.unwrap_or_default();
            let mut all_tasks = Vec::new();

            for link in &valid_links {
                if *prune {
                    symlink::prune_dead_links(&link.target, &link.source, is_dry_run)?;
                }

                let local_ignores = link.ignore.clone().unwrap_or_default();
                match symlink::resolve_tasks(
                    &link.source,
                    &link.target,
                    link.recursive,
                    &global_ignores,
                    &local_ignores,
                ) {
                    Ok(tasks) => all_tasks.extend(tasks),
                    Err(e) => warn!("Skipping entry due to error: {}", e),
                }
            }

            if all_tasks.is_empty() {
                info!("No files found to sync.");
                return Ok(());
            }

            let pb = ui::create_progress_bar(all_tasks.len() as u64);

            all_tasks.par_iter().for_each(|task| {
                pb.set_message(format!(
                    "Syncing {:?}",
                    task.target.file_name().unwrap_or_default()
                ));

                if let Err(e) = symlink::create_link(task, cli.force, is_dry_run) {
                    pb.suspend(|| {
                        error!(
                            "{} Failed to link {}: {}",
                            "✖".red(),
                            task.target.display(),
                            e
                        );
                    });
                }
                pb.inc(1);
            });

            pb.finish_with_message(format!("{} Sync complete.", "✔".green()));
        }

        Some(Commands::Check) => {
            let config_path = Config::resolve_path(cli.config.as_ref())?;
            let config = Config::load(&config_path)?;

            info!(
                "{} Checking configuration at {}...",
                "ℹ".blue(),
                config_path.display()
            );

            let global_ignores = config.global_ignores.unwrap_or_default();

            for link in config.links.iter() {
                if let Some(os) = &link.os {
                    if os != &current_env.os {
                        println!(
                            "{} [Skipped] {} (OS mismatch: requires {})",
                            "⏸".cyan(),
                            link.target.display(),
                            os
                        );
                        continue;
                    }
                }
                if let Some(host) = &link.host {
                    if host != &current_env.hostname {
                        println!(
                            "{} [Skipped] {} (Host mismatch: requires {})",
                            "⏸".cyan(),
                            link.target.display(),
                            host
                        );
                        continue;
                    }
                }

                let local_ignores = link.ignore.clone().unwrap_or_default();

                match symlink::resolve_tasks(
                    &link.source,
                    &link.target,
                    link.recursive,
                    &global_ignores,
                    &local_ignores,
                ) {
                    Ok(tasks) => {
                        for task in tasks {
                            if task.target.is_symlink() {
                                println!(
                                    "{} {} -> {}",
                                    "✔".green(),
                                    task.target.display(),
                                    task.source.display()
                                );
                            } else if task.target.exists() {
                                println!(
                                    "{} {} is a regular file/dir (Conflict)",
                                    "✖".red(),
                                    task.target.display()
                                );
                            } else {
                                println!("{} {} is missing", "⚠".yellow(), task.target.display());
                            }
                        }
                    }
                    Err(e) => {
                        println!(
                            "{} Failed to resolve {}: {}",
                            "✖".red(),
                            link.target.display(),
                            e
                        );
                    }
                }
            }
        }

        None => {
            println!(
                "Welcome to RSM! Run {} to see available commands.",
                "rsm --help".yellow().bold()
            );
        }
    }
    Ok(())
}
