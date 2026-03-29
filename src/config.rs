use crate::error::RsmError;
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub global_ignores: Option<Vec<String>>,
    pub links: Vec<LinkEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LinkEntry {
    pub target: PathBuf,
    pub source: PathBuf,
    #[serde(default)]
    pub recursive: bool,
    pub os: Option<String>,
    pub host: Option<String>,
    pub tags: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
}

impl Config {
    pub fn resolve_path(cli_path: Option<&PathBuf>) -> Result<PathBuf, RsmError> {
        if let Some(path) = cli_path {
            if path.exists() {
                return Ok(path.clone());
            }
            return Err(RsmError::Config(format!(
                "Provided config path does not exist: {}",
                path.display()
            )));
        }

        let local_path = PathBuf::from("rsm.toml");
        if local_path.exists() {
            return Ok(local_path);
        }

        if let Some(proj_dirs) = ProjectDirs::from("", "", "rsm") {
            let config_dir = proj_dirs.config_dir();
            let xdg_path = config_dir.join("rsm.toml");
            if xdg_path.exists() {
                return Ok(xdg_path);
            }
        }

        Err(RsmError::Config(
            "Could not find rsm.toml in current directory or ~/.config/rsm/".into(),
        ))
    }

    pub fn load(path: &Path) -> Result<Self, RsmError> {
        info!("Loading configuration from: {}", path.display());
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn init_template(path: &Path) -> Result<(), RsmError> {
        if path.exists() {
            return Err(RsmError::TargetExists(path.to_path_buf()));
        }

        let template = r#"
# RSM Configuration
global_ignores = [".git", ".DS_Store", "node_modules"]

[[links]]
target = "~/.config/hypr/"
source = "~/dotfiles/hyprland/"
recursive = true
tags = ["wm", "ui"]
os = "linux"
ignore = ["*.bak", "secrets.conf"]

[[links]]
target = "~/.bashrc"
source = "~/dotfiles/bash/bashrc"
tags = ["shell"]
host = "my-work-laptop"
"#;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, template.trim_start())?;
        Ok(())
    }
}
