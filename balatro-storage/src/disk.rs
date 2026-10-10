use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::paths::Paths;

/// Copies save.jkr from the game container to `dest`.
/// Returns None if save.jkr does not exist (no active run).
pub fn copy_save_out(paths: &Paths, dest: &Path) -> Result<Option<PathBuf>, Error> {
    let src = paths.save_jkr();
    if !src.exists() {
        return Ok(None);
    }
    std::fs::copy(&src, dest).map_err(|e| Error::FileCopy {
        src: src.clone(),
        dst: dest.to_path_buf(),
        source: e,
    })?;
    Ok(Some(dest.to_path_buf()))
}

/// Copies settings.jkr from the game container to `dest`.
/// Returns None if settings.jkr does not exist.
pub fn copy_settings_out(paths: &Paths, dest: &Path) -> Result<Option<PathBuf>, Error> {
    let src = paths.settings_jkr();
    if !src.exists() {
        return Ok(None);
    }
    std::fs::copy(&src, dest).map_err(|e| Error::FileCopy {
        src: src.clone(),
        dst: dest.to_path_buf(),
        source: e,
    })?;
    Ok(Some(dest.to_path_buf()))
}

/// Copies save.jkr from `src` into the game container for the given profile.
pub fn copy_save_in(paths: &Paths, src: &Path) -> Result<(), Error> {
    let dest = paths.save_jkr();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::FileCopy {
            src: src.to_path_buf(),
            dst: dest.clone(),
            source: e,
        })?;
    }
    std::fs::copy(src, &dest).map_err(|e| Error::FileCopy {
        src: src.to_path_buf(),
        dst: dest,
        source: e,
    })?;
    Ok(())
}

/// Copies settings.jkr from `src` into the game container.
pub fn copy_settings_in(paths: &Paths, src: &Path) -> Result<(), Error> {
    let dest = paths.settings_jkr();
    std::fs::copy(src, &dest).map_err(|e| Error::FileCopy {
        src: src.to_path_buf(),
        dst: dest,
        source: e,
    })?;
    Ok(())
}
