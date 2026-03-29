use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RsmError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration Error: {0}")]
    Config(String),

    #[error("Target already exists at {0}. Use --force to overwrite.")]
    TargetExists(PathBuf),

    #[error("Source does not exist: {0}")]
    SourceMissing(PathBuf),

    #[error("Failed to parse TOML: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Failed to resolve absolute path for: {0}")]
    PathResolution(PathBuf),

    #[error("Directory traversal error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Invalid glob pattern: {0}")]
    PatternError(#[from] glob::PatternError),
}
