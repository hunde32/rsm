use crate::error::RsmError;
use std::fs;
use std::os::unix::fs::symlink as unix_symlink;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

fn expand_tilde(path: &Path) -> PathBuf {
    if !path.starts_with("~") {
        return path.to_path_buf();
    }
    let mut new_path = directories::BaseDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .unwrap_or_default();

    if path.components().count() > 1 {
        new_path.push(path.strip_prefix("~").unwrap());
    }
    new_path
}

pub fn process_link(
    target: &Path,
    source: &Path,
    force: bool,
    dry_run: bool,
) -> Result<(), RsmError> {
    let expanded_target = expand_tilde(target);
    let expanded_source = expand_tilde(source);

    if !expanded_source.exists() {
        warn!("Source file does not exist: {}", expanded_source.display());
        return Err(RsmError::SourceMissing(expanded_source));
    }

    if expanded_target.exists() || expanded_target.is_symlink() {
        if force {
            debug!(
                "Force flag enabled. Removing existing target: {}",
                expanded_target.display()
            );
            if !dry_run {
                if expanded_target.is_dir() && !expanded_target.is_symlink() {
                    fs::remove_dir_all(&expanded_target)?;
                } else {
                    fs::remove_file(&expanded_target)?;
                }
            }
        } else {
            return Err(RsmError::TargetExists(expanded_target));
        }
    }

    if let Some(parent) = expanded_target.parent() {
        if !parent.exists() {
            debug!(
                "Creating parent directories for: {}",
                expanded_target.display()
            );
            if !dry_run {
                fs::create_dir_all(parent)?;
            }
        }
    }

    debug!(
        "Creating symlink: {} -> {}",
        expanded_target.display(),
        expanded_source.display()
    );
    if !dry_run {
        let abs_source = fs::canonicalize(&expanded_source)
            .map_err(|_| RsmError::PathResolution(expanded_source.clone()))?;
        unix_symlink(abs_source, &expanded_target)?;
    }

    Ok(())
}
