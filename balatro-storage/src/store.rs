use std::path::{Path, PathBuf};

use crate::disk;
use crate::error::Error;
use crate::paths::{Paths, Platform};
use crate::plist_store;

/// Optional path overrides supplied by the caller (e.g. CLI flags).
#[derive(Debug, Default, Clone)]
pub struct PathOverrides {
    /// Override the binary plist path.
    pub plist: Option<PathBuf>,
    /// Override the game support directory.
    pub game_dir: Option<PathBuf>,
}

impl PathOverrides {
    pub fn apply(&self, paths: Paths) -> Paths {
        paths
            .with_plist(self.plist.clone())
            .with_game_dir(self.game_dir.clone())
    }
}

/// Files exported for a single profile.
#[derive(Debug)]
pub struct ExportResult {
    pub profile: u8,
    pub meta_jkr: PathBuf,
    pub profile_jkr: PathBuf,
    /// None if no active run was in progress.
    pub save_jkr: Option<PathBuf>,
    /// None if settings.jkr was not present.
    pub settings_jkr: Option<PathBuf>,
}

/// Summary of what was imported for a single profile.
#[derive(Debug)]
pub struct ImportResult {
    pub profile: u8,
    pub imported_meta: bool,
    pub imported_profile: bool,
    pub imported_save: bool,
    pub imported_settings: bool,
}

/// What exists in the plist and on disk for a given profile.
#[derive(Debug)]
pub struct ProfileState {
    pub profile: u8,
    pub has_meta: bool,
    pub has_profile: bool,
    pub has_save: bool,
    pub has_settings: bool,
}

/// Export all save files for `profile` into `out_dir`.
///
/// Writes:
/// - `{out_dir}/{profile}-meta.jkr`
/// - `{out_dir}/{profile}-profile.jkr`
/// - `{out_dir}/{profile}-save.jkr`    (only if an active run exists)
/// - `{out_dir}/settings.jkr`
pub fn export(
    platform: Platform,
    profile: u8,
    out_dir: &Path,
    overrides: &PathOverrides,
) -> Result<ExportResult, Error> {
    std::fs::create_dir_all(out_dir).map_err(|e| Error::FileWrite {
        path: out_dir.to_path_buf(),
        source: e,
    })?;

    let paths = overrides.apply(Paths::new(platform, profile)?);

    // Extract meta + profile from plist
    let (meta_bytes, profile_bytes) = plist_store::extract(&paths)?;

    let meta_dest = out_dir.join(format!("{profile}-meta.jkr"));
    let profile_dest = out_dir.join(format!("{profile}-profile.jkr"));

    std::fs::write(&meta_dest, &meta_bytes).map_err(|e| Error::FileWrite {
        path: meta_dest.clone(),
        source: e,
    })?;
    std::fs::write(&profile_dest, &profile_bytes).map_err(|e| Error::FileWrite {
        path: profile_dest.clone(),
        source: e,
    })?;

    // Copy disk-resident files
    let save_dest = out_dir.join(format!("{profile}-save.jkr"));
    let save_jkr = disk::copy_save_out(&paths, &save_dest)?;

    let settings_dest = out_dir.join("settings.jkr");
    let settings_jkr = disk::copy_settings_out(&paths, &settings_dest)?;

    Ok(ExportResult {
        profile,
        meta_jkr: meta_dest,
        profile_jkr: profile_dest,
        save_jkr,
        settings_jkr,
    })
}

/// Import all save files for `profile` from `from_dir` into the game.
///
/// Expects:
/// - `{from_dir}/{profile}-meta.jkr`
/// - `{from_dir}/{profile}-profile.jkr`
/// - `{from_dir}/{profile}-save.jkr`   (optional)
/// - `{from_dir}/settings.jkr`          (optional)
pub fn import(
    platform: Platform,
    profile: u8,
    from_dir: &Path,
    overrides: &PathOverrides,
) -> Result<ImportResult, Error> {
    guard_game_not_running()?;

    let paths = overrides.apply(Paths::new(platform, profile)?);

    // Inject meta + profile into plist
    let meta_src = from_dir.join(format!("{profile}-meta.jkr"));
    let profile_src = from_dir.join(format!("{profile}-profile.jkr"));

    let meta_bytes = std::fs::read(&meta_src).map_err(|e| Error::FileRead {
        path: meta_src,
        source: e,
    })?;
    let profile_bytes = std::fs::read(&profile_src).map_err(|e| Error::FileRead {
        path: profile_src,
        source: e,
    })?;

    plist_store::inject(&paths, meta_bytes, profile_bytes)?;

    // Copy disk-resident files if present
    let save_src = from_dir.join(format!("{profile}-save.jkr"));
    let imported_save = if save_src.exists() {
        disk::copy_save_in(&paths, &save_src)?;
        true
    } else {
        false
    };

    let settings_src = from_dir.join("settings.jkr");
    let imported_settings = if settings_src.exists() {
        disk::copy_settings_in(&paths, &settings_src)?;
        true
    } else {
        false
    };

    Ok(ImportResult {
        profile,
        imported_meta: true,
        imported_profile: true,
        imported_save,
        imported_settings,
    })
}

/// Returns the state of all three profiles on the given platform.
pub fn list(platform: Platform, overrides: &PathOverrides) -> Result<Vec<ProfileState>, Error> {
    let mut states = Vec::new();

    for profile in 1u8..=3 {
        let paths = match Paths::new(platform, profile) {
            Ok(p) => overrides.apply(p),
            Err(_) => continue,
        };

        let present = plist_store::present_profiles(&paths.plist).unwrap_or_default();

        let has_meta = present.contains(&profile);
        let has_profile_data = has_meta;
        let has_save = paths.save_jkr().exists();
        let has_settings = paths.settings_jkr().exists();

        states.push(ProfileState {
            profile,
            has_meta,
            has_profile: has_profile_data,
            has_save,
            has_settings,
        });
    }

    Ok(states)
}

fn guard_game_not_running() -> Result<(), Error> {
    let running = std::process::Command::new("pgrep")
        .args(["-xq", "Balatro"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if running {
        Err(Error::GameRunning)
    } else {
        Ok(())
    }
}
