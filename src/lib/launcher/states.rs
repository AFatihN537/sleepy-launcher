use gtk4::{self as gtk, prelude::*};
use libadwaita::{self as adw, prelude::*};

use std::io::{Error, ErrorKind};

use anime_game_core::prelude::*;

use crate::lib::config;

#[derive(Debug, Clone)]
pub enum LauncherState {
    Launch,
    PatchAvailable(Patch),

    // Always contains `VersionDiff::Diff`
    VoiceUpdateAvailable(VersionDiff),

    /// Always contains `VersionDiff::Outdated`
    VoiceOutdated(VersionDiff),

    /// Always contains `VersionDiff::NotInstalled`
    VoiceNotInstalled(VersionDiff),

    // Always contains `VersionDiff::Diff`
    GameUpdateAvailable(VersionDiff),

    /// Always contains `VersionDiff::Outdated`
    GameOutdated(VersionDiff),

    /// Always contains `VersionDiff::NotInstalled`
    GameNotInstalled(VersionDiff)
}

impl Default for LauncherState {
    fn default() -> Self {
        Self::Launch
    }
}

impl LauncherState {
    pub fn get(status_page: Option<adw::StatusPage>) -> std::io::Result<Self> {
        let config = config::get()?;
        let game = Game::new(&config.game.path);

        if let Some(status_page) = &status_page {
            status_page.set_description(Some("Updating game info..."));
        }

        let diff = game.try_get_diff()?;

        Ok(match diff {
            VersionDiff::Latest(_) => {
                if let Some(status_page) = &status_page {
                    status_page.set_description(Some("Updating voice info..."));
                }

                for voice_package in &config.game.voices {
                    let mut voice_package = VoicePackage::with_locale(match VoiceLocale::from_str(voice_package) {
                        Some(locale) => locale,
                        None => return Err(Error::new(ErrorKind::Other, format!("Incorrect voice locale \"{}\" specified in the config", voice_package)))
                    })?;

                    if let Some(status_page) = &status_page {
                        status_page.set_description(Some(format!("Updating voice info ({})...", voice_package.locale().to_name()).as_str()));
                    }

                    // Replace voice package struct with the one constructed in the game's folder
                    // so it'll properly calculate its difference instead of saying "not installed"
                    if voice_package.is_installed_in(&config.game.path) {
                        voice_package = match VoicePackage::new(get_voice_package_path(&config.game.path, voice_package.locale())) {
                            Some(locale) => locale,
                            None => return Err(Error::new(ErrorKind::Other, format!("Failed to load {} voice package", voice_package.locale().to_name())))
                        };
                    }

                    let diff = voice_package.try_get_diff()?;

                    match diff {
                        VersionDiff::Latest(_) => continue,
                        VersionDiff::Diff { .. } => return Ok(Self::VoiceUpdateAvailable(diff)),
                        VersionDiff::Outdated { .. } => return Ok(Self::VoiceOutdated(diff)),
                        VersionDiff::NotInstalled { .. } => return Ok(Self::VoiceNotInstalled(diff))
                    }
                }

                if let Some(status_page) = &status_page {
                    status_page.set_description(Some("Updating patch info..."));
                }

                let patch = Patch::try_fetch(config.patch.servers.clone())?;

                if patch.is_applied(&config.game.path)? {
                    Self::Launch
                }

                else {
                    Self::PatchAvailable(patch)
                }
            },
            VersionDiff::Diff { .. } => Self::GameUpdateAvailable(diff),
            VersionDiff::Outdated { .. } => Self::GameOutdated(diff),
            VersionDiff::NotInstalled { .. } => Self::GameNotInstalled(diff)
        })
    }
}
