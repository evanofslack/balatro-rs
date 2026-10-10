use std::path::PathBuf;

use crate::error::Error;

const BUNDLE_ID: &str = "com.playstack.balatroarcade";

/// The platform/distribution the save files come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOsArcade,
}

/// All relevant paths for a given profile on a given platform.
#[derive(Debug, Clone)]
pub struct Paths {
    pub platform: Platform,
    pub profile: u8,
    /// The binary plist that holds meta.jkr and profile.jkr blobs.
    pub plist: PathBuf,
    /// Directory where save.jkr and settings.jkr live.
    pub game_dir: PathBuf,
}

impl Paths {
    pub fn new(platform: Platform, profile: u8) -> Result<Self, Error> {
        if !(1..=3).contains(&profile) {
            return Err(Error::InvalidProfile(profile));
        }
        match platform {
            Platform::MacOsArcade => Self::macos_arcade(profile),
        }
    }

    fn macos_arcade(profile: u8) -> Result<Self, Error> {
        let home = dirs::home_dir().ok_or(Error::NoHomeDir)?;
        let container = home
            .join("Library")
            .join("Containers")
            .join(BUNDLE_ID)
            .join("Data");

        let plist = container
            .join("Library")
            .join("Preferences")
            .join(format!("{BUNDLE_ID}.plist"));

        let game_dir = container
            .join("Library")
            .join("Application Support")
            .join("game");

        Ok(Self {
            platform: Platform::MacOsArcade,
            profile,
            plist,
            game_dir,
        })
    }

    /// Path to save.jkr for this profile.
    pub fn save_jkr(&self) -> PathBuf {
        self.game_dir
            .join(self.profile.to_string())
            .join("save.jkr")
    }

    /// Path to settings.jkr (shared across all profiles).
    pub fn settings_jkr(&self) -> PathBuf {
        self.game_dir.join("settings.jkr")
    }

    /// Override the plist path (e.g. from a CLI flag).
    pub fn with_plist(mut self, plist: Option<PathBuf>) -> Self {
        if let Some(p) = plist {
            self.plist = p;
        }
        self
    }

    /// Override the game directory path (e.g. from a CLI flag).
    pub fn with_game_dir(mut self, game_dir: Option<PathBuf>) -> Self {
        if let Some(d) = game_dir {
            self.game_dir = d;
        }
        self
    }

    /// Plist key for the meta.jkr blob.
    pub fn meta_key(&self) -> String {
        format!("{}__meta.jkr.data", self.profile)
    }

    /// Plist key for the profile.jkr blob.
    pub fn profile_key(&self) -> String {
        format!("{}__profile.jkr.data", self.profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_out_of_range_profile() {
        assert!(matches!(
            Paths::new(Platform::MacOsArcade, 0),
            Err(Error::InvalidProfile(0))
        ));
        assert!(matches!(
            Paths::new(Platform::MacOsArcade, 4),
            Err(Error::InvalidProfile(4))
        ));
    }

    #[test]
    fn plist_keys_and_disk_paths() {
        let p = Paths::new(Platform::MacOsArcade, 2)
            .unwrap()
            .with_game_dir(Some(PathBuf::from("/g")));
        assert_eq!(p.meta_key(), "2__meta.jkr.data");
        assert_eq!(p.profile_key(), "2__profile.jkr.data");
        assert_eq!(p.save_jkr(), PathBuf::from("/g/2/save.jkr"));
        assert_eq!(p.settings_jkr(), PathBuf::from("/g/settings.jkr"));
    }
}
